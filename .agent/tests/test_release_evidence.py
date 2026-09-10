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
            'CI_REPO': 'corbet-labs/ccvl', 'CI_COMMIT_SHA': self.commit,
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
        for platform, host in RELEASE.PLATFORMS.items():
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

    def test_complete_six_native_results_and_gates(self):
        self.check()
        manifest = json.loads((self.output / 'manifest.json').read_text())
        self.assertEqual(len(manifest['platforms']), 6)
        self.assertEqual(len(manifest['receipts']), 10)

    def test_one_platform_absent_never_passes(self):
        (self.output / 'ccvl-windows-arm64.receipt.json').unlink()
        with self.assertRaises(FileNotFoundError):
            self.check()

    def test_cross_host_cannot_claim_native_platform(self):
        self.change('ccvl-macos-arm64.receipt.json', 'rust_host', 'x86_64-unknown-linux-gnu')
        with self.assertRaisesRegex(ValueError, 'native platform'):
            self.check()

    def test_different_lock_or_source_or_binary_refused(self):
        for field in ('source_commit', 'source_sha256', 'cargo_lock_sha256', 'runtime_id'):
            with self.subTest(field=field):
                self.populate()
                self.change('ccvl-windows-arm64.receipt.json', field, 'b' * 64)
                with self.assertRaisesRegex(ValueError, field):
                    self.check()
        self.populate()
        (self.output / 'ccvl-windows-arm64.exe').write_bytes(b'changed binary')
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
    tag=target.split('/tags/')[1]; release=root/tag
    if not (release/'state').exists():
        print('HTTP/2.0 404 Not Found\\n\\n{}');sys.exit(1)
    data=json.loads((release/'state').read_text())
    data['assets']=[{'name':p.name} for p in (release/'assets').iterdir()]
    print('HTTP/2.0 200 OK\\n\\n'+json.dumps(data));sys.exit()
assert args[0]=='release'
action,tag=args[1:3];release=root/tag
if action=='create':
    release.mkdir();(release/'assets').mkdir();(release/'state').write_text(json.dumps({'id':1,'draft':True,'target_commitish':os.environ['CI_COMMIT_SHA']}));sys.exit()
if action=='upload':
    assert '--clobber' not in args
    asset=pathlib.Path(args[-1]);dest=release/'assets'/asset.name
    assert not dest.exists(),'overwriting upload'
    shutil.copyfile(asset,dest);sys.exit()
if action=='download':
    name=args[args.index('--pattern')+1];dest=pathlib.Path(args[args.index('--output')+1])
    assert not dest.exists();shutil.copyfile(release/'assets'/name,dest);sys.exit()
if action=='edit':
    (release/'state').write_text(json.dumps({'id':1,'draft':False,'target_commitish':os.environ['CI_COMMIT_SHA']}));sys.exit()
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
        result = self.publisher(directory)
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(calls.read_text().count('"upload"'), first_uploads)
        runtime = Path(self.temp.name) / 'github' / ('runtime-' + self.identity['runtime_id'])
        (runtime / 'assets/ccvl-linux-x86_64').write_bytes(b'different published bytes')
        result = self.publisher(directory)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('No overwrite attempted', result.stdout)
        self.assertEqual(calls.read_text().count('"upload"'), first_uploads)

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
        (self.output / 'ccvl-windows-arm64.receipt.json').unlink()
        result = self.publisher(directory)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((Path(self.temp.name) / 'github/calls').exists())


if __name__ == '__main__':
    unittest.main()
