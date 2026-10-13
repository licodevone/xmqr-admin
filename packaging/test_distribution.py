#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A03 contracts and independent negative fixtures; no network or host install."""
from pathlib import Path
import sys
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import collect_notices as NOTICES
import build_distribution as BUILD


class DistributionContract(unittest.TestCase):
    def test_revision_negative_matrix(self):
        for bad in ('0', '-1', '01', '1;touch', 'one', '9999999', '1\n'):
            with self.subTest(value=bad), self.assertRaises(ValueError):
                BUILD.revision_number(bad)
        self.assertEqual(BUILD.revision_number('2'), '2')

    def test_original_nested_notices(self):
        with tempfile.TemporaryDirectory(prefix='xmqr-notice-test-') as temp:
            root = Path(temp)
            (root / 'Cargo.toml').write_text('fixture')
            (root / 'LICENSE').write_bytes(b'copyright fixture\x00\r\n')
            (root / 'AUTHORS').write_bytes(b'original authors notice\r\n')
            (root / 'vendor').mkdir()
            (root / 'vendor/NOTICE.txt').write_bytes(b'vendored notice\r\n')
            (root / 'vendor/source.rs').write_text('not a notice')
            base, found = NOTICES.notice_paths({'name': 'fixture', 'manifest_path': str(root / 'Cargo.toml')})
            self.assertEqual(base, root.resolve())
            self.assertEqual({p.relative_to(base).as_posix() for p in found}, {'LICENSE', 'AUTHORS', 'vendor/NOTICE.txt'})

    def test_empty_or_escape_license_refused(self):
        with tempfile.TemporaryDirectory(prefix='xmqr-notice-test-') as temp:
            root = Path(temp)
            (root / 'package').mkdir()
            (root / 'package/Cargo.toml').write_text('fixture')
            (root / 'LICENSE').write_text('outside')
            package = {'name': 'fixture', 'manifest_path': str(root / 'package/Cargo.toml')}
            with self.assertRaises(ValueError):
                NOTICES.notice_paths(package)
            package['license_file'] = '../LICENSE'
            with self.assertRaises(ValueError):
                NOTICES.notice_paths(package)

    def test_symlink_notice_refused(self):
        with tempfile.TemporaryDirectory(prefix='xmqr-notice-test-') as temp:
            root = Path(temp)
            (root / 'Cargo.toml').write_text('fixture')
            (root / 'original').write_text('fixture original')
            (root / 'LICENSE').symlink_to(root / 'original')
            with self.assertRaises(ValueError):
                NOTICES.notice_paths({'name': 'fixture', 'manifest_path': str(root / 'Cargo.toml')})

    def test_tar_relative_readonly_root_owned(self):
        import tarfile
        with tempfile.TemporaryDirectory(prefix='xmqr-tar-test-') as temp:
            root = Path(temp)
            stage = root / 'stage'
            stage.mkdir()
            (stage / 'LICENSE').write_bytes(b'original\r\n')
            BUILD.make_tar(stage, root / 'output.tar.gz', 0)
            with tarfile.open(root / 'output.tar.gz') as archive:
                members = archive.getmembers()
                self.assertEqual([m.name for m in members], ['LICENSE'])
                self.assertTrue(all(m.uid == m.gid == 0 and m.mtime == 0 for m in members))
                self.assertEqual(archive.extractfile('LICENSE').read(), b'original\r\n')

    def test_context_only_allowlisted_public_inputs(self):
        rules = [r for r in (ROOT / '.dockerignore').read_text().splitlines() if r and not r.startswith('#')]
        self.assertEqual(rules[0], '**')
        allowed = {'!Cargo.toml', '!Cargo.lock', '!Dockerfile', '!.dockerignore', '!LICENSE',
                   '!ORIGIN.json', '!src/', '!src/**', '!assets/', '!assets/**', '!scripts/',
                   '!scripts/collect_notices.py', '!docs/', '!docs/third-party-licenses.md',
                   '!packaging/', '!packaging/notices/', '!packaging/notices/valuable-0.1.1/',
                   '!packaging/notices/valuable-0.1.1/LICENSE'}
        self.assertEqual({r for r in rules if r.startswith('!')}, allowed)
        for exclusion in ('**/.env*', '**/*.key', '**/*.pem', '**/*.crt', '**/*.wal', '**/*.token', '**/users.toml'):
            self.assertGreater(rules.index(exclusion), rules.index('!assets/**'))

    def test_docker_independent_abi_nonroot_and_original_notices(self):
        dockerfile = (ROOT / 'Dockerfile').read_text()
        package = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']
        self.assertIn('FROM rust:${RUST_VERSION}-slim-bookworm AS build', dockerfile)
        self.assertIn('FROM debian:bookworm-slim AS runtime', dockerfile)
        self.assertIn('ARG VERSION=' + package['version'], dockerfile)
        self.assertIn('USER 10001:10001', dockerfile)
        self.assertIn('STOPSIGNAL SIGTERM', dockerfile)
        self.assertIn('COPY --from=build /notices ', dockerfile)
        self.assertIn('git python3', dockerfile)
        self.assertTrue(dockerfile.rstrip().endswith('ENTRYPOINT ["/usr/local/bin/xmqr-admin"]'))
        self.assertNotIn('0.0.0.0', dockerfile)

    def test_valuable_supplement_matches_original_commit_hash(self):
        self.assertEqual(NOTICES.digest(ROOT / 'packaging/notices/valuable-0.1.1/LICENSE'),
                         'ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44')

    def test_unknown_missing_notice_has_no_generic_fallback(self):
        with self.assertRaises(ValueError):
            NOTICES.supplement_paths(ROOT, {'name': 'unknown', 'version': '0.1.0'}, [])


if __name__ == '__main__':
    unittest.main()
