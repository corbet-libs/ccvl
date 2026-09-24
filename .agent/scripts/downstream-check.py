#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Julian Y. Richard Corbet
# SPDX-License-Identifier: FSL-1.1-ALv2
"""Check one private downstream on Crow using its matching published runtime."""
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import tempfile


PUBLIC_REPO = 'corbet-libs/ccvl'
PUBLIC_URL = 'https://github.com/' + PUBLIC_REPO + '.git'
ROOT = Path(__file__).resolve().parents[2]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def run(label, *args, cwd=None, data=None):
    result = subprocess.run(args, cwd=cwd, input=data, capture_output=True)
    # Gate errors can contain private document text or paths. Keep the command's
    # output inside this process; report only the failing stage to Crow.
    require(result.returncode == 0, label + ' failed; no private content was published.')
    return result.stdout


def source_files(data, commit):
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:') as archive:
        require(archive.pax_headers.get('comment') == commit, 'Source archive commit mismatch.')
        files = {}
        for entry in archive:
            name = PurePosixPath(entry.name)
            require(not name.is_absolute() and '..' not in name.parts and '.git' not in name.parts,
                    'Unsafe source archive path.')
            if entry.isdir():
                continue
            require(entry.isfile() and entry.name not in files,
                    'Private checks require unique regular source files.')
            files[entry.name] = (entry.mode & 0o111, sha256(archive.extractfile(entry).read()))
        return files


def check():
    require(os.environ.get('CI') == 'crow' and os.environ.get('CI_REPO_PRIVATE') == 'true'
            and bool(os.environ.get('CI_REPO')) and os.environ['CI_REPO'] != PUBLIC_REPO
            and not os.environ.get('GITHUB_ACTIONS'),
            'Private downstream checks require a private repository on Crow.')
    require(os.environ.get('CI_PIPELINE_EVENT') == 'manual', 'Private downstream checks require a manual event.')
    target = os.environ.get('CARGO_TARGET_DIR')
    require(target and os.environ.get('CCID_TARGET_LOCK_HELD') == target,
            'Private downstream checks require the verified ccid target lock.')
    require(not any(os.environ.get(key) for key in (
        'GH_TOKEN', 'GITHUB_TOKEN', 'DOWNSTREAM_PUSH_KEY', 'CARGO_REGISTRY_TOKEN',
        'CARGO_REGISTRIES_CRATES_IO_TOKEN', 'NPM_TOKEN', 'NODE_AUTH_TOKEN', 'PYPI_TOKEN', 'JSR_TOKEN',
    )), 'Publication credentials are not accepted by the private check stage.')
    commit = os.environ.get('CI_COMMIT_SHA', '')
    upstream = os.environ.get('UPSTREAM_COMMIT', '')
    require(re.fullmatch('[0-9a-f]{40}', commit) and re.fullmatch('[0-9a-f]{40}', upstream),
            'Exact private and upstream commits are required.')
    for key in ('SOURCE_SHA256', 'DOWNSTREAM_SOURCE_SHA256'):
        require(re.fullmatch('[0-9a-f]{64}', os.environ.get(key, '')), 'Missing source or bundle digest.')
    archive = Path(os.environ['SOURCE_ARCHIVE']).read_bytes()
    require(sha256(archive) == os.environ['SOURCE_SHA256'], 'Source archive checksum mismatch.')
    files = source_files(archive, commit)
    # Check the actual ccid extraction too, not merely its archive header.
    for name, (mode, checksum) in files.items():
        path = ROOT / name
        require(path.is_file() and not path.is_symlink() and path.stat().st_mode & 0o111 == mode
                and sha256(path.read_bytes()) == checksum, 'Extracted source differs from its archive.')
    bundle = Path(os.environ['DOWNSTREAM_SOURCE_BUNDLE']).resolve()
    require(sha256(bundle.read_bytes()) == os.environ['DOWNSTREAM_SOURCE_SHA256'], 'Git bundle checksum mismatch.')
    # No checkout hooks, credential helpers, replacement objects, or private
    # remote configuration are inherited by this job-only Git repository.
    for key in tuple(os.environ):
        if key.startswith('GIT_'):
            os.environ.pop(key)
    os.environ.update(GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL='/dev/null',
                      GIT_TERMINAL_PROMPT='0', GIT_NO_REPLACE_OBJECTS='1')
    require(run('Bundle identity', 'git', 'bundle', 'list-heads', str(bundle)).decode().strip()
            == commit + ' refs/heads/source', 'Git bundle commit mismatch.')
    with tempfile.TemporaryDirectory(prefix='ccvl-private-check-') as temporary:
        scratch = Path(temporary)
        verifier = scratch / 'verify.git'
        run('Empty bundle verifier', 'git', 'init', '--bare', '--template=', '--quiet', str(verifier))
        run('Complete Git history', 'git', 'bundle', 'verify', str(bundle), cwd=verifier)
        checkout = scratch / 'source'
        run('Private checkout', 'git', 'clone', '--quiet', '--no-checkout', '--template=', str(bundle), str(checkout))
        run('Exact private checkout', 'git', 'checkout', '--quiet', '--detach', commit, cwd=checkout)
        # Compare full path/content/executable inventories; two transport files
        # bearing the same commit header cannot substitute different source.
        committed_archive = run('Committed source identity', 'git', 'archive', commit, cwd=checkout)
        require(source_files(committed_archive, commit) == files,
                'Source archive differs from the committed Git bundle tree.')
        run('Upstream ancestry', 'git', 'merge-base', '--is-ancestor', upstream, commit, cwd=checkout)
        policy = json.loads((checkout / 'ccvl-downstream.json').read_text())
        configuration = policy.get('upstream', {})
        remote = configuration.get('remote', '')
        require(configuration.get('url') == PUBLIC_URL and re.fullmatch('[A-Za-z0-9][A-Za-z0-9_.-]*', remote),
                'Private policy must name the public ccvl upstream.')
        # The Git bundle clone has no private origin URL or push credentials.
        run('Remove bundle origin', 'git', 'remote', 'remove', 'origin', cwd=checkout)
        run('Policy upstream', 'git', 'remote', 'add', remote, PUBLIC_URL, cwd=checkout)
        # Use the existing bootstrap identity algorithm from the explicit public
        # ancestor; do not source a downstream-modified helper or allow its test hook.
        helper = scratch / 'runtime-id.sh'
        helper.write_bytes(run('Upstream runtime identity helper', 'git', 'show',
                               upstream + ':.agent/scripts/runtime-id.sh', cwd=checkout))
        os.environ.pop('CCVL_BOOTSTRAP_TESTING', None)
        os.environ.pop('CCVL_BOOTSTRAP_TEST_FINGERPRINT', None)
        fingerprint = run('Workspace runtime identity', 'bash', '-c',
                          'source "$1"; repo_root="$2"; probe() { command -v "$1"; }; source_fingerprint',
                          'private-check', str(helper), str(checkout)).decode().strip()
        require(re.fullmatch('[0-9a-f]{64}', fingerprint), 'Invalid workspace runtime identity.')
        require(os.uname().sysname == 'Linux' and os.uname().machine == 'x86_64',
                'This Crow route requires the existing Linux x86_64 executor.')
        asset = 'ccvl-linux-x86_64'
        base = 'https://github.com/' + PUBLIC_REPO + '/releases/download/runtime-' + fingerprint
        checksum_file = scratch / (asset + '.sha256')
        curl = ('curl', '-q', '--fail', '--location', '--proto', '=https', '--proto-redir', '=https',
                '--tlsv1.2', '--silent', '--show-error')
        run('Matching published runtime checksum unavailable', *curl, '--output', str(checksum_file), base + '/' + asset + '.sha256')
        checksum = re.fullmatch(r'([0-9a-f]{64})  ccvl-linux-x86_64\n?', checksum_file.read_text())
        require(checksum, 'Invalid published runtime checksum.')
        binary = scratch / asset
        existing = os.environ.get('CCVL_CHECK_RUNTIME')
        if existing:
            # An existing artifact is accepted only against the current published
            # checksum and runtime ID; its path alone is not release evidence.
            shutil.copyfile(existing, binary)
        else:
            run('Matching published runtime unavailable', *curl, '--output', str(binary), base + '/' + asset)
        require(sha256(binary.read_bytes()) == checksum[1], 'Published runtime checksum mismatch.')
        binary.chmod(0o700)
        require(run('Published runtime identity', str(binary), 'runtime-id', cwd=checkout).decode().strip() == fingerprint,
                'Published runtime is stale or does not match this source.')
        run('Downstream ownership gate', str(binary), 'downstream-check', '--policy', 'ccvl-downstream.json',
            '--upstream-ref', upstream, cwd=checkout)
        run('Full private document gate', str(binary), 'check', cwd=checkout)
        return {'schema': 1, 'repository': os.environ['CI_REPO'], 'source_commit': commit,
                'upstream_commit': upstream, 'source_archive_sha256': sha256(archive),
                'git_bundle_sha256': os.environ['DOWNSTREAM_SOURCE_SHA256'],
                'runtime_id': fingerprint, 'binary_sha256': checksum[1],
                'checks': ['downstream-check', 'check'], 'merged': False, 'pushed': False}


if __name__ == '__main__':
    try:
        os.umask(0o077)
        print(json.dumps(check(), sort_keys=True))
    except (ValueError, OSError, KeyError, TypeError, tarfile.TarError):
        # Do not print exception payloads: a malformed private policy or missing
        # private path can otherwise leak its contents into an upstream log.
        import sys
        error = sys.exception()
        print(str(error) if type(error) is ValueError else 'Private check inputs are unavailable or invalid.', file=sys.stderr)
        sys.exit(2)
