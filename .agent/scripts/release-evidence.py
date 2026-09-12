#!/usr/bin/env python3
"""Source-bound native packages and release receipts shared by CI providers."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[2]
PLATFORMS = {
    'linux-x86_64': 'x86_64-unknown-linux-gnu',
    'linux-aarch64': 'aarch64-unknown-linux-gnu',
    'macos-x86_64': 'x86_64-apple-darwin',
    'macos-arm64': 'aarch64-apple-darwin',
    'windows-x86_64': 'x86_64-pc-windows-msvc',
    'windows-arm64': 'aarch64-pc-windows-msvc',
}
GATES = ('rust', 'lint', 'linux-deep', 'archive')


def release_platforms():
    """Explicit source policy; available artifacts never choose release scope."""
    platforms = (ROOT / '.agent/release-platforms.txt').read_text().splitlines()
    if (not platforms or len(platforms) != len(set(platforms))
            or any(platform not in PLATFORMS for platform in platforms)
            or 'linux-x86_64' not in platforms):
        fail('Invalid release platform policy; unique known targets including Linux x86_64 are required.')
    return platforms


def fail(message):
    raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_hash(path):
    with Path(path).open('rb') as stream:
        result = hashlib.sha256()
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(block)
        return result.hexdigest()


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def source():
    repository = os.environ.get('CI_REPO') or os.environ.get('GITHUB_REPOSITORY')
    if repository != 'corbet-labs/ccvl':
        fail('Release work requires the public corbet-labs/ccvl repository.')
    commit = os.environ.get('CI_COMMIT_SHA') or os.environ.get('GITHUB_SHA')
    if not commit or not re.fullmatch('[0-9a-f]{40}', commit):
        fail('An exact CI source commit is required.')
    archive_path = os.environ.get('SOURCE_ARCHIVE')
    if archive_path:
        data = Path(archive_path).read_bytes()
        if digest(data) != os.environ.get('SOURCE_SHA256'):
            fail('Source archive checksum mismatch.')
    else:
        if command('git', 'rev-parse', 'HEAD') != commit:
            fail('Checkout differs from the requested source commit.')
        # Generated ignored caches are allowed; changed tracked/public inputs are not.
        if command('git', 'status', '--porcelain', '--untracked-files=normal'):
            fail('Release source checkout must be clean.')
        data = subprocess.check_output(['git', 'archive', commit], cwd=ROOT)
    archive = tarfile.open(fileobj=io.BytesIO(data), mode='r:')
    if archive.pax_headers.get('comment') != commit:
        fail('Source archive is not the requested Git commit.')
    files = {}
    for member in archive:
        relative = PurePosixPath(member.name)
        if relative.is_absolute() or '..' in relative.parts or '.git' in relative.parts:
            fail('Unsafe source archive path.')
        if member.isdir():
            continue
        if not member.isfile() or member.name in files:
            fail('Only unique regular source files are supported.')
        payload = archive.extractfile(member).read()
        if not (ROOT / member.name).is_file() or (ROOT / member.name).is_symlink():
            fail('Missing source input: ' + member.name)
        if os.name != 'nt' and ((ROOT / member.name).stat().st_mode & 0o111) != (member.mode & 0o111):
            fail('Source executable mode changed: ' + member.name)
        if file_hash(ROOT / member.name) != digest(payload):
            fail('Source input changed: ' + member.name)
        files[member.name] = (payload, member.mode)
    required = ('Cargo.toml', '.agent/core/Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.agent/build.rs')
    if not all(path in files for path in (*required, '.agent/release-platforms.txt')):
        fail('Incomplete source archive.')
    fingerprint_paths = list(required) + sorted(
        path for path in files if path.startswith(('.agent/src/', '.agent/core/src/')) and path.endswith('.rs'))
    fingerprint = digest(''.join(f'{path} {digest(files[path][0])}\n' for path in fingerprint_paths).encode())
    # Canonical file content/mode identity permits differently encoded provider archives.
    workspace = digest(''.join(
        f'{path} {mode:o} {digest(payload)}\n'
        for path, (payload, mode) in sorted(files.items())).encode())
    return {
        'schema': 1, 'repository': repository, 'source_commit': commit,
        'source_sha256': workspace, 'source_archive_sha256': digest(data),
        'cargo_lock_sha256': digest(files['Cargo.lock'][0]), 'runtime_id': fingerprint,
    }, files


def identity_equal(actual, expected):
    for field in ('schema', 'repository', 'source_commit', 'source_sha256', 'cargo_lock_sha256', 'runtime_id'):
        if actual.get(field) != expected[field]:
            fail('Release receipt differs in ' + field)
    if not re.fullmatch('[0-9a-f]{64}', actual.get('source_archive_sha256', '')):
        fail('Missing source archive identity.')


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = (json.dumps(value, indent=2, sort_keys=True) + '\n').encode()
    if path.exists() and path.read_bytes() != encoded:
        fail('Refusing to replace differing evidence: ' + str(path))
    path.write_bytes(encoded)


def load(path):
    return json.loads(Path(path).read_text())


def provenance():
    provider = 'crow' if os.environ.get('CI_REPO') else 'gha'
    run = os.environ.get('CI_PIPELINE_NUMBER') or os.environ.get('CI_BUILD_NUMBER') or os.environ.get('GITHUB_RUN_ID')
    if not run:
        fail('A CI run identity is required.')
    return {'provider': provider, 'run': str(run)}


def native(args):
    identity, files = source()
    if args.host != PLATFORMS[args.platform]:
        fail('Rust compiler host does not match the requested native platform.')
    binary = Path(args.binary).resolve()
    if command(str(binary), 'runtime-id') != identity['runtime_id']:
        fail('Compiled runtime differs from the verified source.')
    output = Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    name = 'ccvl-' + args.platform + ('.exe' if args.platform.startswith('windows-') else '')
    exe = 'ccvl.exe' if args.platform.startswith('windows-') else 'ccvl'
    archive_name = 'ccvl-' + args.platform + '.tar.gz'
    binary_path = output / name
    if binary_path.exists() and file_hash(binary_path) != file_hash(binary):
        fail('Output already contains a different native executable.')
    binary_path.write_bytes(binary.read_bytes())
    binary_path.chmod(0o755)
    archive_path = output / archive_name
    if archive_path.exists():
        fail('Package already exists; retain it and use a new attempt directory.')
    with tarfile.open(archive_path, 'w:gz') as archive:
        for path, (payload, mode) in sorted(files.items()):
            entry = tarfile.TarInfo(path)
            entry.size, entry.mode, entry.mtime = len(payload), mode, 0
            archive.addfile(entry, io.BytesIO(payload))
        payload = binary.read_bytes()
        entry = tarfile.TarInfo('.agent/cache/ccvl/bin/' + exe)
        entry.size, entry.mode, entry.mtime = len(payload), 0o755, 0
        archive.addfile(entry, io.BytesIO(payload))
    assets = {name: file_hash(binary_path), archive_name: file_hash(archive_path)}
    for filename, checksum in assets.items():
        (output / (filename + '.sha256')).write_text(f'{checksum}  {filename}\n')
    (output / (name + '.runtime-id')).write_text(identity['runtime_id'] + '\n')
    write_json(output / ('ccvl-' + args.platform + '.receipt.json'), {
        **identity, **provenance(), 'kind': 'native', 'platform': args.platform,
        'rust_host': args.host, 'rustc': args.rustc, 'cargo': args.cargo,
        'checks': ['public-check', 'native-runtime'], 'assets': assets,
    })


def gate(args):
    identity, _ = source()
    receipt = {**identity, **provenance(), 'kind': 'gate', 'gate': args.gate}
    if args.gate in ('linux-deep', 'archive'):
        name = 'ccvl-linux-x86_64.receipt.json'
        native_receipt = load(Path(args.output) / name)
        identity_equal(native_receipt, identity)
        receipt['native_receipt_sha256'] = file_hash(Path(args.output) / name)
    write_json(Path(args.output) / ('gate-' + args.gate + '.json'), receipt)


def check(args):
    identity, _ = source()
    root = Path(args.output)
    receipts = {}
    runtimes = {}
    platforms = release_platforms()
    expected_native = {'ccvl-' + platform + '.receipt.json' for platform in platforms}
    if any(path.name not in expected_native for path in root.glob('ccvl-*.receipt.json')):
        fail('Native evidence outside the explicit release platform policy.')
    for platform in platforms:
        host = PLATFORMS[platform]
        path = root / ('ccvl-' + platform + '.receipt.json')
        receipt = load(path)
        identity_equal(receipt, identity)
        if (receipt.get('kind'), receipt.get('platform'), receipt.get('rust_host')) != ('native', platform, host):
            fail('Missing native platform evidence: ' + platform)
        if receipt.get('checks') != ['public-check', 'native-runtime'] or not receipt.get('rustc') or not receipt.get('cargo'):
            fail('Incomplete native check evidence: ' + platform)
        if receipt.get('provider') not in ('crow', 'gha') or not receipt.get('run'):
            fail('Missing native run provenance: ' + platform)
        name = 'ccvl-' + platform + ('.exe' if platform.startswith('windows-') else '')
        expected_names = {name, 'ccvl-' + platform + '.tar.gz'}
        if set(receipt.get('assets', {})) != expected_names:
            fail('Unexpected native asset list: ' + platform)
        for filename, checksum in receipt['assets'].items():
            if file_hash(root / filename) != checksum:
                fail('Artifact checksum mismatch: ' + filename)
            if (root / (filename + '.sha256')).read_text() != f'{checksum}  {filename}\n':
                fail('Checksum file mismatch: ' + filename)
        if (root / (name + '.runtime-id')).read_text().strip() != identity['runtime_id']:
            fail('Runtime identity mismatch: ' + platform)
        receipts[path.name] = file_hash(path)
        runtimes[platform] = {'binary_sha256': receipt['assets'][name], 'rust_host': host}
    for name in GATES:
        path = root / ('gate-' + name + '.json')
        receipt = load(path)
        identity_equal(receipt, identity)
        if (receipt.get('kind'), receipt.get('gate')) != ('gate', name):
            fail('Missing release gate: ' + name)
        if receipt.get('provider') not in ('crow', 'gha') or not receipt.get('run'):
            fail('Missing gate run provenance: ' + name)
        if name in ('linux-deep', 'archive') and receipt.get('native_receipt_sha256') != receipts['ccvl-linux-x86_64.receipt.json']:
            fail('Linux verification used a different native package.')
        receipts[path.name] = file_hash(path)
    write_json(root / 'manifest.json', {**identity, 'platforms': platforms, 'receipts': receipts})
    write_json(root / 'runtime-manifest.json', {
        'schema': 1, 'runtime_id': identity['runtime_id'],
        'cargo_lock_sha256': identity['cargo_lock_sha256'], 'platforms': runtimes})
    print(identity['source_commit'], identity['runtime_id'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='action', required=True)
    native_parser = commands.add_parser('native')
    native_parser.add_argument('--platform', choices=PLATFORMS, required=True)
    for flag in ('binary', 'host', 'rustc', 'cargo', 'output'):
        native_parser.add_argument('--' + flag, required=True)
    gate_parser = commands.add_parser('gate')
    gate_parser.add_argument('gate', choices=GATES)
    gate_parser.add_argument('--output', required=True)
    checker = commands.add_parser('check')
    checker.add_argument('--output', required=True)
    source_parser = commands.add_parser('source')
    source_parser.add_argument('--output')
    commands.add_parser('platforms', help='Print the explicit supported release targets.')
    args = parser.parse_args()
    if args.action == 'platforms':
        print('\n'.join(release_platforms()))
    elif args.action == 'source':
        identity, _ = source()
        if args.output:
            write_json(args.output, identity)
        else:
            print(json.dumps(identity, sort_keys=True))
    else:
        {'native': native, 'gate': gate, 'check': check}[args.action](args)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, KeyError, tarfile.TarError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
