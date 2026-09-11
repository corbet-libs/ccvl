#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Julian Y. Richard Corbet
# SPDX-License-Identifier: FSL-1.1-ALv2
"""Run focused date regressions from one separately pinned ccvl candidate."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib


ROOT = Path(__file__).resolve().parents[2]
TEST = '.agent/tests/test_application_dates.py'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def extract(snapshot, destination, commit):
    with tarfile.open(snapshot, mode='r:') as archive:
        require(archive.pax_headers.get('comment') == commit,
                'Candidate archive commit differs from the pinned revision.')
        seen = set()
        for entry in archive.getmembers():
            path = PurePosixPath(entry.name)
            require(not path.is_absolute() and '..' not in path.parts
                    and '.git' not in path.parts and str(path) not in seen,
                    'Candidate archive contains an unsafe or duplicate path.')
            require(entry.isdir() or entry.isfile(),
                    'Candidate archive requires regular files and directories.')
            seen.add(str(path))
        require(TEST in seen, 'Candidate archive lacks the focused date regressions.')
        archive.extractall(destination, filter='data')
    require((destination / TEST).is_file(), 'Candidate date regression is not a file.')


def check():
    require(os.environ.get('CI') == 'crow'
            and os.environ.get('CI_REPO') == 'corbet-labs/ccvl'
            and os.environ.get('CI_PIPELINE_EVENT') == 'manual',
            'Candidate date checks require the manual ccvl Crow route.')
    harness_commit = os.environ.get('CI_COMMIT_SHA', '')
    require(re.fullmatch('[0-9a-f]{40}', harness_commit), 'Missing exact harness commit.')
    pinned = tomllib.loads((ROOT / '.ci/archives.toml').read_text())['archives']['date-candidate']
    commit = pinned['revision']
    require(re.fullmatch('[0-9a-f]{40}', commit), 'Missing exact candidate commit.')
    require(pinned['archive_variable'] == 'DATE_SOURCE_ARCHIVE'
            and pinned['digest_variable'] == 'DATE_SOURCE_SHA256'
            and pinned.get('workflows') == ['candidate-date'],
            'Candidate archive must be bound to this workflow only.')
    expected = os.environ.get('DATE_SOURCE_SHA256', '')
    require(re.fullmatch('[0-9a-f]{64}', expected), 'Missing candidate source digest.')
    target = os.environ.get('CARGO_TARGET_DIR', '')
    require(target and Path(target).is_absolute()
            and os.environ.get('CCID_TARGET_LOCK_HELD') == target,
            'Candidate checks require the shared persistent target lock.')
    parent = Path(target) / 'ccvl-candidate-date' / commit / harness_commit
    parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix='run-', dir=parent))
    receipt = {
        'schema': 1, 'provider': 'crow', 'repository': 'corbet-labs/ccvl',
        'check': 'candidate-date', 'harness_commit': harness_commit,
        'harness_source_sha256': os.environ.get('SOURCE_SHA256'),
        'candidate_commit': commit, 'candidate_source_sha256': expected,
        'tool_revision': os.environ.get('CCID_REVISION'),
        'run': os.environ.get('CI_PIPELINE_NUMBER', os.environ.get('CI_BUILD_NUMBER')),
        'python': sys.version.split()[0], 'uv': subprocess.check_output(
            ['uv', '--version'], text=True).strip(),
        'typst_requirement': 'typst==0.15.0',
        'dependency_access': {'typst': 'offline-cache', 'rust': 'registry-resolve-once-then-locked'},
        'started': int(time.time()), 'status': 'running',
        'scope': 'focused date module and Typst letters; no full ccvl runtime or release validation',
    }
    try:
        with tempfile.TemporaryDirectory(prefix='ccvl-candidate-date-') as temporary:
            scratch = Path(temporary)
            snapshot = scratch / 'candidate.tar'
            shutil.copyfile(os.environ['DATE_SOURCE_ARCHIVE'], snapshot)
            require(digest(snapshot) == expected, 'Candidate archive SHA-256 mismatch.')
            source = scratch / 'source'
            source.mkdir()
            extract(snapshot, source, commit)
            receipt['regression_script_sha256'] = digest(source / TEST)
            environment = {
                **os.environ, 'UV_PYTHON_DOWNLOADS': 'never', 'UV_OFFLINE': 'true',
                'PYTHONDONTWRITEBYTECODE': '1', 'CCVL_DATE_ARTIFACT_DIR': str(output),
                'CCVL_DATE_CANDIDATE_COMMIT': commit,
            }
            subprocess.run([
                'uv', 'run', '--offline', '--no-project', '--no-managed-python',
                '--python', sys.executable, '--with', 'typst==0.15.0',
                'python', '-B', TEST,
            ], cwd=source, env=environment, check=True)
        receipt['status'] = 'success'
    except (ValueError, OSError, KeyError, tarfile.TarError, subprocess.CalledProcessError):
        receipt['status'] = 'failure'
        raise
    finally:
        receipt['finished'] = int(time.time())
        receipt['artifacts'] = {
            str(path.relative_to(output)): digest(path)
            for path in sorted(output.rglob('*')) if path.is_file() and not path.is_symlink()
        }
        (output / 'candidate-date-receipt.json').write_text(
            json.dumps(receipt, indent=2, sort_keys=True) + '\n')
        print(json.dumps({'artifact_directory': str(output), 'receipt': receipt}, sort_keys=True), flush=True)


if __name__ == '__main__':
    check()
