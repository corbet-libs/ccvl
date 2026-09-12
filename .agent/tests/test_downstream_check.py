#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Julian Y. Richard Corbet
# SPDX-License-Identifier: FSL-1.1-ALv2
"""Offline private-route guards with real Git archives and fixture runtimes."""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


class DownstreamCheck(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.repo = self.base / 'repository'
        self.repo.mkdir()
        self.environment = {k: v for k, v in os.environ.items() if not k.startswith(('GIT_', 'CCVL_'))}
        for key in ('GH_TOKEN', 'GITHUB_TOKEN', 'GITHUB_ACTIONS', 'DOWNSTREAM_PUSH_KEY',
                    'CARGO_REGISTRY_TOKEN', 'CARGO_REGISTRIES_CRATES_IO_TOKEN',
                    'NPM_TOKEN', 'NODE_AUTH_TOKEN', 'PYPI_TOKEN', 'JSR_TOKEN'):
            self.environment.pop(key, None)
        self.environment.update(GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL='/dev/null',
                                GIT_AUTHOR_NAME='fixture', GIT_AUTHOR_EMAIL='fixture@example.invalid',
                                GIT_COMMITTER_NAME='fixture', GIT_COMMITTER_EMAIL='fixture@example.invalid')
        self.git('init', '--quiet', '--template=', '--initial-branch=main')
        for name in ('Cargo.toml', '.agent/core/Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.agent/build.rs', '.agent/src/main.rs', '.agent/core/src/lib.rs'):
            self.write(name, 'runtime fixture\n')
        for name in ('runtime-id.sh', 'downstream-check.py'):
            source = ROOT / '.agent/scripts' / name
            self.write('.agent/scripts/' + name, source.read_bytes())
        self.git('add', '.')
        self.git('commit', '--quiet', '-m', 'public fixture')
        self.upstream = self.git('rev-parse', 'HEAD').decode().strip()
        self.write('ccvl-downstream.json', json.dumps({
            'schema_version': 1,
            'upstream': {'remote': 'upstream', 'url': 'https://github.com/corbet-labs/ccvl.git', 'branch': 'main'},
            'allowed_paths': ['ccvl-downstream.json', 'interview/'],
        }))
        self.write('interview/profile.md', 'PRIVATE_SENTINEL\n')
        self.git('add', '.')
        self.git('commit', '--quiet', '-m', 'private fixture')
        self.commit = self.git('rev-parse', 'HEAD').decode().strip()
        self.git('update-ref', 'refs/heads/source', self.commit)
        self.bundle = self.base / 'source.bundle'
        self.git('bundle', 'create', str(self.bundle), 'refs/heads/source')
        self.archive = self.base / 'source.tar'
        self.archive.write_bytes(self.git('archive', self.commit))
        self.extracted = self.base / 'extracted'
        self.extracted.mkdir()
        with tarfile.open(self.archive) as archive:
            archive.extractall(self.extracted, filter='data')
        self.fingerprint = subprocess.check_output([
            'bash', '-c', 'source "$1"; repo_root="$2"; probe() { command -v "$1"; }; source_fingerprint',
            'fixture', str(self.repo / '.agent/scripts/runtime-id.sh'), str(self.repo),
        ], env=self.environment, text=True).strip()
        self.bin = self.base / 'bin'
        self.bin.mkdir()
        self.assets = self.base / 'published'
        self.assets.mkdir()
        self.trace = self.base / 'trace'
        self.trace.write_text('')
        binary = self.assets / 'ccvl-linux-x86_64'
        binary.write_text('#!' + sys.executable + '\n' + '''import os, pathlib, sys
command = sys.argv[1]
with open(os.environ['TEST_TRACE'], 'a') as log: log.write(command + '\\n')
if command == 'runtime-id':
    print('0' * 64 if os.environ.get('TEST_WRONG_RUNTIME') else os.environ['TEST_RUNTIME_ID'])
elif command == 'downstream-check':
    assert sys.argv[2:] == ['--policy', 'ccvl-downstream.json', '--upstream-ref', os.environ['UPSTREAM_COMMIT']]
    if os.environ.get('TEST_OWNERSHIP_FAILURE'):
        print('PRIVATE_SENTINEL', file=sys.stderr); sys.exit(1)
elif command == 'check':
    if os.environ.get('TEST_DOCUMENT_FAILURE'):
        print('PRIVATE_SENTINEL', file=sys.stderr); sys.exit(1)
else: sys.exit(99)
''')
        (self.assets / 'ccvl-linux-x86_64.sha256').write_text(
            self.hash(binary) + '  ccvl-linux-x86_64\n')
        curl = self.bin / 'curl'
        curl.write_text('#!' + sys.executable + '\n' + '''import os, pathlib, shutil, sys
name = sys.argv[-1].rsplit('/', 1)[1]
assert sys.argv[1] == '-q'
assert sys.argv[-1] == 'https://github.com/corbet-labs/ccvl/releases/download/runtime-' + os.environ['TEST_RUNTIME_ID'] + '/' + name
with open(os.environ['TEST_TRACE'], 'a') as log: log.write('download:' + name + '\\n')
if os.environ.get('TEST_RUNTIME_UNAVAILABLE'): sys.exit(22)
shutil.copyfile(pathlib.Path(os.environ['TEST_ASSETS']) / name, sys.argv[sys.argv.index('--output') + 1])
''')
        curl.chmod(0o700)
        self.environment.update(
            PATH=str(self.bin) + os.pathsep + self.environment['PATH'], CI='crow',
            CI_REPO='fixture/private-downstream', CI_REPO_PRIVATE='true', CI_PIPELINE_EVENT='manual',
            CI_COMMIT_SHA=self.commit, UPSTREAM_COMMIT=self.upstream,
            CARGO_TARGET_DIR=str(self.base / 'target'), CCID_TARGET_LOCK_HELD=str(self.base / 'target'),
            SOURCE_ARCHIVE=str(self.archive), SOURCE_SHA256=self.hash(self.archive),
            DOWNSTREAM_SOURCE_BUNDLE=str(self.bundle), DOWNSTREAM_SOURCE_SHA256=self.hash(self.bundle),
            TEST_RUNTIME_ID=self.fingerprint, TEST_TRACE=str(self.trace), TEST_ASSETS=str(self.assets),
        )

    @staticmethod
    def hash(path):
        return hashlib.sha256(path.read_bytes()).hexdigest()

    def write(self, name, content):
        path = self.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content if isinstance(content, bytes) else content.encode())

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.repo, env=self.environment, stderr=subprocess.DEVNULL)

    def invoke(self, success=False, **environment):
        self.trace.write_text('')
        result = subprocess.run([sys.executable, str(self.extracted / '.agent/scripts/downstream-check.py')],
                                env={**self.environment, **environment}, capture_output=True, text=True)
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        self.assertNotIn('PRIVATE_SENTINEL', result.stdout + result.stderr)
        self.assertEqual(self.git('rev-parse', 'HEAD').decode().strip(), self.commit)
        self.assertEqual(self.git('status', '--porcelain'), b'')
        return result, self.trace.read_text().splitlines()

    def test_exact_private_commit_runs_both_gates_without_changing_source(self):
        result, trace = self.invoke(success=True)
        receipt = json.loads(result.stdout)
        self.assertEqual(receipt['source_commit'], self.commit)
        self.assertEqual(receipt['upstream_commit'], self.upstream)
        self.assertEqual(receipt['checks'], ['downstream-check', 'check'])
        self.assertFalse(receipt['merged'])
        self.assertFalse(receipt['pushed'])
        self.assertEqual(trace[-3:], ['runtime-id', 'downstream-check', 'check'])

    def test_public_hosted_and_nonmanual_execution_are_refused_before_download(self):
        for environment in ({'CI': 'true'}, {'CI_REPO_PRIVATE': 'false'},
                            {'CI_REPO': 'corbet-labs/ccvl'}, {'GITHUB_ACTIONS': 'true'},
                            {'CI_PIPELINE_EVENT': 'push'}):
            with self.subTest(environment=environment):
                _, trace = self.invoke(**environment)
                self.assertEqual(trace, [])

    def test_publication_credentials_are_refused_without_disclosure(self):
        for key in ('GH_TOKEN', 'DOWNSTREAM_PUSH_KEY', 'JSR_TOKEN'):
            with self.subTest(key=key):
                _, trace = self.invoke(**{key: 'PRIVATE_SENTINEL'})
                self.assertEqual(trace, [])

    def test_archive_checksum_and_extracted_source_changes_are_refused(self):
        _, trace = self.invoke(SOURCE_SHA256='0' * 64)
        self.assertEqual(trace, [])
        (self.extracted / 'interview/profile.md').write_text('changed private source')
        _, trace = self.invoke()
        self.assertEqual(trace, [])

    def test_archive_and_bundle_tree_disagreement_is_refused(self):
        changed = io.BytesIO()
        with tarfile.open(self.archive) as original, tarfile.open(fileobj=changed, mode='w',
                format=tarfile.PAX_FORMAT, pax_headers={'comment': self.commit}) as archive:
            for entry in original:
                data = original.extractfile(entry).read() if entry.isfile() else None
                if entry.name == 'interview/profile.md':
                    data = b'altered transport\n'
                    entry.size = len(data)
                    (self.extracted / entry.name).write_bytes(data)
                archive.addfile(entry, io.BytesIO(data) if data is not None else None)
        self.archive.write_bytes(changed.getvalue())
        _, trace = self.invoke(SOURCE_SHA256=self.hash(self.archive))
        self.assertEqual(trace, [])

    def test_wrong_bundle_and_nonancestor_upstream_are_refused(self):
        _, trace = self.invoke(DOWNSTREAM_SOURCE_SHA256='0' * 64)
        self.assertEqual(trace, [])
        _, trace = self.invoke(UPSTREAM_COMMIT='0' * 40)
        self.assertEqual(trace, [])

    def test_missing_published_runtime_stops_before_private_gates(self):
        result, trace = self.invoke(TEST_RUNTIME_UNAVAILABLE='1')
        self.assertIn('Matching published runtime checksum unavailable', result.stderr)
        self.assertNotIn('downstream-check', trace)

    def test_wrong_runtime_checksum_and_identity_stop_before_private_gates(self):
        _, trace = self.invoke(TEST_WRONG_RUNTIME='1')
        self.assertNotIn('downstream-check', trace)
        (self.assets / 'ccvl-linux-x86_64.sha256').write_text('0' * 64 + '  ccvl-linux-x86_64\n')
        _, trace = self.invoke()
        self.assertNotIn('runtime-id', trace)

    def test_existing_runtime_requires_published_checksum_and_is_not_reinstalled(self):
        existing = self.assets / 'ccvl-linux-x86_64'
        _, trace = self.invoke(success=True, CCVL_CHECK_RUNTIME=str(existing))
        self.assertEqual([v for v in trace if v.startswith('download:')], ['download:ccvl-linux-x86_64.sha256'])
        self.assertFalse((self.extracted / '.agent/cache').exists())

    def test_ownership_and_document_failures_do_not_report_success_or_private_text(self):
        result, trace = self.invoke(TEST_OWNERSHIP_FAILURE='1')
        self.assertIn('Downstream ownership gate failed', result.stderr)
        self.assertNotIn('check', trace)
        result, trace = self.invoke(TEST_DOCUMENT_FAILURE='1')
        self.assertIn('Full private document gate failed', result.stderr)
        self.assertEqual(trace[-2:], ['downstream-check', 'check'])


if __name__ == '__main__':
    unittest.main()
