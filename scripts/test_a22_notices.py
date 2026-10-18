#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""Exact locked README declarations: positives and independent refusals."""
import copy
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

import collect_notices as NOTICES


ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {
    'pharos': ('0.5.3', 'f5b985a59cfeced5ae66325d477b6c982c9408bf',
               '19578b95fc9fe905a8b1b2cdf27a32fd4df37c10f7002a18e6f2044209f5f331'),
    'ws_stream_tungstenite': ('0.15.0', '508c0f28037b5decab5eb61ac4ab805f05c96cdf',
                             '3d73da6e62bbed9fb2f082398be3f178b16f5e163415199494c87ee3bd0962d6'),
}


class ReadmeNoticeContracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        metadata = json.loads(subprocess.check_output(
            ['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT))
        cls.packages = {p['name']: p for p in metadata['packages']
                        if p['name'] in EXPECTED}
        if set(cls.packages) != set(EXPECTED):
            raise AssertionError('locked dependencies missing from notice contract')

    def fixture(self, name, directory):
        package = copy.deepcopy(self.packages[name])
        source = Path(package['manifest_path']).parent
        base = Path(directory) / name
        base.mkdir()
        for filename in ('README.md', '.cargo_vcs_info.json'):
            shutil.copyfile(source / filename, base / filename)
        (base / 'Cargo.toml').write_text('# synthetic fixture; no build\n')
        package['manifest_path'] = str(base / 'Cargo.toml')
        return package, base

    def test_exact_original_readme_and_provenance(self):
        for name, (version, commit, checksum) in EXPECTED.items():
            with self.subTest(name=name):
                package = self.packages[name]
                self.assertEqual(package['version'], version)
                base, files, provenance = NOTICES.supplement_paths(ROOT, package, [])
                self.assertEqual(files, [base / 'README.md'])
                self.assertEqual(NOTICES.digest(files[0]), checksum)
                self.assertEqual(provenance['source_revision'], commit)
                self.assertEqual(provenance['repository'], package['repository'])
                self.assertTrue(provenance['url'].endswith('/' + commit + '/README.md#license'))

    def test_changed_source_commit_refused(self):
        for name in EXPECTED:
            with self.subTest(name=name), tempfile.TemporaryDirectory(prefix='a22-notices-') as temp:
                package, base = self.fixture(name, temp)
                vcs = json.loads((base / '.cargo_vcs_info.json').read_text())
                vcs['git']['sha1'] = '0' * 40
                (base / '.cargo_vcs_info.json').write_text(json.dumps(vcs))
                with self.assertRaises(ValueError):
                    NOTICES.supplement_paths(ROOT, package, [])

    def test_changed_original_readme_refused(self):
        for name in EXPECTED:
            with self.subTest(name=name), tempfile.TemporaryDirectory(prefix='a22-notices-') as temp:
                package, base = self.fixture(name, temp)
                with (base / 'README.md').open('ab') as stream:
                    stream.write(b'\nchanged fixture\n')
                with self.assertRaises(ValueError):
                    NOTICES.supplement_paths(ROOT, package, [])

    def test_changed_declaration_or_repository_refused(self):
        for name in EXPECTED:
            for field, value in (('license', 'MIT'), ('repository', 'https://example.invalid/wrong')):
                with self.subTest(name=name, field=field):
                    package = dict(self.packages[name], **{field: value})
                    with self.assertRaises(ValueError):
                        NOTICES.supplement_paths(ROOT, package, [])

    def test_other_version_has_no_generic_fallback(self):
        for name in EXPECTED:
            with self.subTest(name=name):
                with self.assertRaisesRegex(ValueError, 'no reviewed notice supplement'):
                    NOTICES.supplement_paths(ROOT, dict(self.packages[name], version='999.0.0'), [])

    def test_symlink_readme_refused(self):
        for name in EXPECTED:
            with self.subTest(name=name), tempfile.TemporaryDirectory(prefix='a22-notices-') as temp:
                package, base = self.fixture(name, temp)
                (base / 'README.md').rename(base / 'original-readme')
                (base / 'README.md').symlink_to(base / 'original-readme')
                with self.assertRaises(ValueError):
                    NOTICES.supplement_paths(ROOT, package, [])

    def test_ws_nonroot_source_path_refused(self):
        with tempfile.TemporaryDirectory(prefix='a22-notices-') as temp:
            package, base = self.fixture('ws_stream_tungstenite', temp)
            vcs = json.loads((base / '.cargo_vcs_info.json').read_text())
            vcs['path_in_vcs'] = 'different-subdirectory'
            (base / '.cargo_vcs_info.json').write_text(json.dumps(vcs))
            with self.assertRaises(ValueError):
                NOTICES.supplement_paths(ROOT, package, [])


if __name__ == '__main__':
    unittest.main()
