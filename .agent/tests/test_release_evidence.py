#!/usr/bin/env python3
"""Guard fixtures: release identity, complete native evidence and immutable uploads."""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import textwrap
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('release_evidence', ROOT / '.agent/scripts/release-evidence.py')
RELEASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RELEASE)


class ReleaseGuards(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ccvl-release-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'source'
        self.root.mkdir()
        self.output = Path(self.temp.name) / 'dist'
        self.output.mkdir()
        self.commit = 'a' * 40
        files = {
            'Cargo.toml': b'[package]\nrust-version="1.94"\n', 'Cargo.lock': b'fixture lock\n',
            'rust-toolchain.toml': b'[toolchain]\nchannel="stable"\n', '.agent/build.rs': b'fn main() {}\n',
            '.agent/src/main.rs': b'fn main() {}\n',
            '.agent/core/Cargo.toml': b'[package]\nname="ccvl-core"\n',
            '.agent/core/src/lib.rs': b'pub fn fixture() {}\n',
            '.agent/release-platforms.txt': (ROOT / '.agent/release-platforms.txt').read_bytes(),
        }
        for name in ('release-evidence.py', 'publish-release.sh'):
            files['.agent/scripts/' + name] = (ROOT / '.agent/scripts' / name).read_bytes()
        self.archive = Path(self.temp.name) / 'source.tar'
        with tarfile.open(self.archive, 'w', format=tarfile.PAX_FORMAT, pax_headers={'comment': self.commit}) as archive:
            for name, payload in files.items():
                path = self.root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(payload)
                entry = tarfile.TarInfo(name)
                entry.size, entry.mode = len(payload), 0o644
                archive.addfile(entry, io.BytesIO(payload))
        self.env = {
            'CI_REPO': 'corbet-libs/ccvl', 'CI_COMMIT_SHA': self.commit,
            'CI_PIPELINE_NUMBER': '123', 'SOURCE_ARCHIVE': str(self.archive),
            'SOURCE_SHA256': RELEASE.file_hash(self.archive),
        }
        self.patches = [patch.object(RELEASE, 'ROOT', self.root), patch.dict(os.environ, self.env)]
        for active in self.patches:
            active.start()
            self.addCleanup(active.stop)
        self.identity, _ = RELEASE.source()
        self.populate()

    def populate(self):
        for platform in RELEASE.release_platforms():
            host = RELEASE.PLATFORMS[platform]
            name = 'ccvl-' + platform + ('.exe' if platform.startswith('windows-') else '')
            assets = {}
            for filename in (name, 'ccvl-' + platform + '.tar.gz'):
                payload = ('fixture ' + filename).encode()
                (self.output / filename).write_bytes(payload)
                checksum = hashlib.sha256(payload).hexdigest()
                assets[filename] = checksum
                (self.output / (filename + '.sha256')).write_text(f'{checksum}  {filename}\n')
            (self.output / (name + '.runtime-id')).write_text(self.identity['runtime_id'] + '\n')
            receipt = {
                **self.identity, 'kind': 'native', 'platform': platform, 'rust_host': host,
                'rustc': 'rustc 1.97.0 fixture', 'cargo': 'cargo 1.97.0 fixture',
                'checks': ['public-check', 'native-runtime'], 'assets': assets,
                'provider': 'crow', 'run': '123',
            }
            self.write('ccvl-' + platform + '.receipt.json', receipt)
        for gate in RELEASE.GATES:
            receipt = {**self.identity, 'kind': 'gate', 'gate': gate, 'provider': 'crow', 'run': '123'}
            if gate in ('linux-deep', 'archive'):
                receipt['native_receipt_sha256'] = RELEASE.file_hash(self.output / 'ccvl-linux-x86_64.receipt.json')
            self.write('gate-' + gate + '.json', receipt)

    def write(self, name, value):
        (self.output / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

    def change(self, name, field, value):
        data = json.loads((self.output / name).read_text())
        data[field] = value
        self.write(name, data)

    def check(self):
        with contextlib.redirect_stdout(io.StringIO()):
            RELEASE.check(type('Arguments', (), {'output': str(self.output)})())

    def test_complete_explicit_native_results_and_gates(self):
        self.check()
        manifest = json.loads((self.output / 'manifest.json').read_text())
        self.assertEqual(manifest['platforms'], ['linux-x86_64'])
        self.assertEqual(len(manifest['receipts']), 5)
        runtime = json.loads((self.output / 'runtime-manifest.json').read_text())
        self.assertEqual(list(runtime['platforms']), ['linux-x86_64'])

    def test_one_platform_absent_never_passes(self):
        (self.output / 'ccvl-linux-x86_64.receipt.json').unlink()
        with self.assertRaises(FileNotFoundError):
            self.check()

    def test_cross_host_cannot_claim_native_platform(self):
        self.change('ccvl-linux-x86_64.receipt.json', 'rust_host', 'aarch64-apple-darwin')
        with self.assertRaisesRegex(ValueError, 'native platform'):
            self.check()

    def test_different_lock_or_source_or_binary_refused(self):
        for field in ('source_commit', 'source_sha256', 'cargo_lock_sha256', 'runtime_id'):
            with self.subTest(field=field):
                self.populate()
                self.change('ccvl-linux-x86_64.receipt.json', field, 'b' * 64)
                with self.assertRaisesRegex(ValueError, field):
                    self.check()
        self.populate()
        (self.output / 'ccvl-linux-x86_64').write_bytes(b'changed binary')
        with self.assertRaisesRegex(ValueError, 'checksum'):
            self.check()

    def test_missing_gate_and_wrong_linux_receipt_refused(self):
        (self.output / 'gate-archive.json').unlink()
        with self.assertRaises(FileNotFoundError):
            self.check()
        self.populate()
        self.change('gate-linux-deep.json', 'native_receipt_sha256', '0' * 64)
        with self.assertRaisesRegex(ValueError, 'different native package'):
            self.check()

    def test_artifact_availability_cannot_expand_release_scope(self):
        self.write('ccvl-windows-arm64.receipt.json', {})
        with self.assertRaisesRegex(ValueError, 'outside the explicit release platform policy'):
            self.check()

    def test_invalid_platform_policy_refused(self):
        for policy in ('', 'macos-arm64\n', 'linux-x86_64\nlinux-x86_64\n',
                       'linux-x86_64\ninvented\n'):
            with self.subTest(policy=policy):
                (self.root / '.agent/release-platforms.txt').write_text(policy)
                with self.assertRaisesRegex(ValueError, 'Invalid release platform policy'):
                    RELEASE.release_platforms()

    def test_source_archive_or_worktree_mutation_refused(self):
        with patch.dict(os.environ, {'SOURCE_SHA256': '0' * 64}):
            with self.assertRaisesRegex(ValueError, 'archive checksum'):
                RELEASE.source()
        (self.root / 'Cargo.lock').write_text('changed lock')
        with self.assertRaisesRegex(ValueError, 'Source input changed'):
            RELEASE.source()

    def test_executable_source_mode_change_refused(self):
        if os.name == 'nt':
            self.skipTest('Windows does not expose POSIX executable modes')
        (self.root / 'Cargo.lock').chmod(0o755)
        with self.assertRaisesRegex(ValueError, 'executable mode'):
            RELEASE.source()

    def test_private_repository_refused(self):
        with patch.dict(os.environ, {'CI_REPO': 'corbet-labs/applications'}):
            with self.assertRaisesRegex(ValueError, 'public'):
                RELEASE.source()

    def fake_gh(self):
        directory = Path(self.temp.name) / 'bin'
        directory.mkdir()
        path = directory / 'gh'
        path.write_text('''#!/usr/bin/env python3
import json,os,pathlib,shutil,sys
args=sys.argv[1:]
root=pathlib.Path(os.environ['FAKE_GH_ROOT']); root.mkdir(exist_ok=True)
with (root/'calls').open('a') as log: log.write(json.dumps(args)+'\\n')
if args[0]=='api':
    if args[1]=='graphql':
        if (root/'lookup-error').exists():
            print((root/'lookup-error').read_text());sys.exit()
        tag=next(arg[4:] for arg in args if arg.startswith('tag='));release=root/tag
        found=None
        if (release/'state').exists():
            found={'databaseId':json.loads((release/'state').read_text())['id'],'tagName':tag}
        print(json.dumps({'data':{'repository':{'release':found}}}));sys.exit()
    target=next(arg for arg in args[1:] if arg.startswith('repos/'))
    if target.endswith('/git/ref/heads/main'): print(os.environ['CI_COMMIT_SHA']); sys.exit()
    if '/git/tags/' in target:
        print('HTTP/2.0 200 OK\\n\\n'+json.dumps({'object':{'type':'commit','sha':os.environ['CI_COMMIT_SHA']}}));sys.exit()
    if '/git/ref/tags/' in target:
        tag=target.split('/tags/')[1]; release=root/tag
        if (release/'state').exists() and not json.loads((release/'state').read_text())['draft']:
            commit=(release/'tag_override').read_text() if (release/'tag_override').exists() else os.environ['CI_COMMIT_SHA']
            kind='tag' if (release/'annotated').exists() else 'commit'
            print('HTTP/2.0 200 OK\\n\\n'+json.dumps({'object':{'type':kind,'sha':commit}}));sys.exit()
        print('HTTP/2.0 404 Not Found\\n\\n{}');sys.exit(1)
    if '/releases/tags/' in target:
        tag=target.split('/tags/')[1];release=root/tag
        if (release/'state').exists() and json.loads((release/'state').read_text())['draft']:
            print('HTTP/2.0 404 Not Found\\n\\n{}');sys.exit(1)
    else:
        release=next(p for p in root.iterdir() if p.is_dir() and (p/'state').exists()
                     and str(json.loads((p/'state').read_text())['id'])==target.rsplit('/',1)[1])
        tag=release.name
    if not (release/'state').exists():
        print('HTTP/2.0 404 Not Found\\n\\n{}');sys.exit(1)
    data=json.loads((release/'state').read_text())
    data['tag_name']=tag
    data['assets']=[{'name':p.name} for p in (release/'assets').iterdir()]
    print('HTTP/2.0 200 OK\\n\\n'+json.dumps(data));sys.exit()
assert args[0]=='release'
action,tag=args[1:3];release=root/tag
if action=='create':
    release.mkdir();(release/'assets').mkdir();(release/'state').write_text(json.dumps({'id':1 if tag.startswith('runtime-') else 2,'draft':True,'target_commitish':os.environ['CI_COMMIT_SHA']}));sys.exit()
if action=='upload':
    assert '--clobber' not in args
    asset=pathlib.Path(args[-1]);dest=release/'assets'/asset.name
    assert not dest.exists(),'overwriting upload'
    shutil.copyfile(asset,dest);sys.exit()
if action=='download':
    name=args[args.index('--pattern')+1];dest=pathlib.Path(args[args.index('--output')+1])
    assert not dest.exists();shutil.copyfile(release/'assets'/name,dest);sys.exit()
if action=='edit':
    data=json.loads((release/'state').read_text());data['draft']=False
    (release/'state').write_text(json.dumps(data));sys.exit()
raise SystemExit('Unexpected gh call '+repr(args))
''')
        path.chmod(0o755)
        return directory

    def publisher(self, directory):
        environment = {**os.environ, 'PATH': str(directory) + os.pathsep + os.environ['PATH'],
                       'GH_TOKEN': 'fixture-token', 'FAKE_GH_ROOT': str(Path(self.temp.name) / 'github')}
        return subprocess.run(['bash', str(self.root / '.agent/scripts/publish-release.sh'), str(self.output)],
                              env=environment, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)

    def test_publication_is_complete_and_repeat_is_immutable(self):
        if not shutil.which('jq'):
            self.fail('jq is required for actual publisher guard fixtures')
        directory = self.fake_gh()
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        calls = Path(self.temp.name) / 'github/calls'
        first_uploads = calls.read_text().count('"upload"')
        self.assertGreater(first_uploads, 0)
        create_calls = [json.loads(line) for line in calls.read_text().splitlines()
                        if json.loads(line)[:2] == ['release', 'create']]
        for call in create_calls:
            notes = call[call.index('--notes') + 1]
            self.assertIn('Released platforms: linux-x86_64.', notes)
            self.assertNotIn('All six', notes)
        self.assertEqual(first_uploads, 12)
        self.assertIn('"graphql"', calls.read_text())
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(calls.read_text().count('"upload"'), first_uploads)
        runtime = Path(self.temp.name) / 'github' / ('runtime-' + self.identity['runtime_id'])
        (runtime / 'assets/ccvl-linux-x86_64').write_bytes(b'different published bytes')
        result = self.publisher(directory)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('No overwrite attempted', result.stdout)
        self.assertEqual(calls.read_text().count('"upload"'), first_uploads)

    def test_unknown_draft_lookup_never_creates_or_uploads(self):
        directory = self.fake_gh()
        remote = Path(self.temp.name) / 'github'
        remote.mkdir()
        cases = [
            {'errors': [{'message': 'fixture lookup unavailable'}]},
            {'data': {'repository': None}},
            {'data': {'repository': {}}},
            {'data': {'repository': {'release': {'databaseId': 1, 'tagName': 'another-release'}}}},
            {'data': {'repository': {'release': {'databaseId': None,
                                                'tagName': 'runtime-' + self.identity['runtime_id']}}}},
        ]
        for response in cases:
            with self.subTest(response=response):
                (remote / 'lookup-error').write_text(json.dumps(response))
                result = self.publisher(directory)
                self.assertNotEqual(result.returncode, 0)
        calls = (remote / 'calls').read_text()
        self.assertNotIn('"create"', calls)
        self.assertNotIn('"upload"', calls)

    def test_existing_wrong_tag_prevents_further_publication(self):
        directory = self.fake_gh()
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        root = Path(self.temp.name) / 'github'
        calls = root / 'calls'
        uploads = calls.read_text().count('"upload"')
        runtime = root / ('runtime-' + self.identity['runtime_id'])
        (runtime / 'tag_override').write_text('b' * 40)
        result = self.publisher(directory)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('targets a different source commit', result.stdout)
        self.assertEqual(calls.read_text().count('"upload"'), uploads)

    def test_annotated_tag_is_peeled_and_published_gap_is_not_repaired(self):
        directory = self.fake_gh()
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        root = Path(self.temp.name) / 'github'
        runtime = root / ('runtime-' + self.identity['runtime_id'])
        (runtime / 'annotated').touch()
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        calls = root / 'calls'
        uploads = calls.read_text().count('"upload"')
        (runtime / 'assets/ccvl-linux-x86_64').unlink()
        result = self.publisher(directory)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('is incomplete', result.stdout)
        self.assertEqual(calls.read_text().count('"upload"'), uploads)

    def test_runtime_may_reuse_earlier_source_but_workspace_tag_may_not(self):
        directory = self.fake_gh()
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        root = Path(self.temp.name) / 'github'
        runtime = root / ('runtime-' + self.identity['runtime_id'])
        state = json.loads((runtime / 'state').read_text())
        state['target_commitish'] = 'b' * 40
        (runtime / 'state').write_text(json.dumps(state))
        (runtime / 'tag_override').write_text('b' * 40)
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        workspace = root / ('build-' + self.commit)
        state = json.loads((workspace / 'state').read_text())
        state['target_commitish'] = 'b' * 40
        (workspace / 'state').write_text(json.dumps(state))
        (workspace / 'tag_override').write_text('b' * 40)
        result = self.publisher(directory)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('targets a different source commit', result.stdout)

    def test_missing_native_evidence_prevents_any_publication_call(self):
        directory = self.fake_gh()
        (self.output / 'ccvl-linux-x86_64.receipt.json').unlink()
        result = self.publisher(directory)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((Path(self.temp.name) / 'github/calls').exists())


class ArchiveExecutorGuards(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ccvl-archive-adapter-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.dist = self.root / 'package directory'
        self.dist.mkdir()
        self.marker = self.root / 'gate-called'
        gate = b'#!/usr/bin/env bash\nset -eu\ntest "$1" = "$EXPECTED_DIST"\nprintf gate > "$GATE_MARKER"\n'
        self.archive = self.root / 'source.tar'
        with tarfile.open(self.archive, 'w') as archive:
            entry = tarfile.TarInfo('.agent/scripts/check-release-archive.sh')
            entry.size = len(gate)
            archive.addfile(entry, io.BytesIO(gate))
        workflow = (ROOT / '.crow/release-archive.yaml').read_text()
        self.command = textwrap.dedent(workflow.split('      - |\n', 1)[1])
        self.executor = self.root / 'external executor'
        self.executor.write_text(
            '#!/usr/bin/env bash\nset -eu\n'
            'test "$0" != "$ORIGINAL_EXECUTOR"\n'
            'printf executor > "$EXECUTOR_MARKER"\n'
            'exec bash "$1" "$2"\n')
        self.env = {**os.environ, 'CI_REPO': 'corbet-libs/ccvl',
                    'SOURCE_ARCHIVE': str(self.archive), 'SOURCE_SHA256': RELEASE.file_hash(self.archive),
                    'CCVL_RELEASE_DIR': str(self.dist), 'EXPECTED_DIST': str(self.dist),
                    'GATE_MARKER': str(self.marker), 'EXECUTOR_MARKER': str(self.root / 'executor-called'),
                    'ORIGINAL_EXECUTOR': str(self.executor), 'ARCHIVE_EXECUTOR': '',
                    'ARCHIVE_EXECUTOR_SHA256': ''}

    def invoke(self, **changes):
        return subprocess.run(['bash', '-c', self.command], env={**self.env, **changes},
                              text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)

    def test_default_route_runs_verified_check_directly(self):
        result = self.invoke()
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(self.marker.read_text(), 'gate')
        self.assertFalse((self.root / 'executor-called').exists())

    def test_verified_executor_snapshot_receives_unchanged_check_and_package(self):
        result = self.invoke(ARCHIVE_EXECUTOR=str(self.executor),
                             ARCHIVE_EXECUTOR_SHA256=RELEASE.file_hash(self.executor))
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(self.marker.read_text(), 'gate')
        self.assertEqual((self.root / 'executor-called').read_text(), 'executor')

    def test_incomplete_changed_or_untrusted_executor_never_executes(self):
        checksum = RELEASE.file_hash(self.executor)
        link = self.root / 'executor-link'
        link.symlink_to(self.executor)
        cases = [
            {'ARCHIVE_EXECUTOR': str(self.executor)},
            {'ARCHIVE_EXECUTOR_SHA256': checksum},
            {'ARCHIVE_EXECUTOR': str(self.executor), 'ARCHIVE_EXECUTOR_SHA256': '0' * 64},
            {'ARCHIVE_EXECUTOR': str(link), 'ARCHIVE_EXECUTOR_SHA256': checksum},
            {'ARCHIVE_EXECUTOR': str(self.executor), 'ARCHIVE_EXECUTOR_SHA256': checksum,
             'SOURCE_SHA256': '0' * 64},
        ]
        for inputs in cases:
            with self.subTest(inputs=inputs):
                result = self.invoke(**inputs)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertFalse(self.marker.exists())
                self.assertFalse((self.root / 'executor-called').exists())


if __name__ == '__main__':
    unittest.main()
