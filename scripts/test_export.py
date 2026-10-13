#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
import copy
import unittest
from export_image import validate_image


class ExportContract(unittest.TestCase):
    def setUp(self):
        self.image = {'Id': 'sha256:fixture', 'Os': 'linux', 'Architecture': 'amd64',
                      'Config': {'User': '10001:10001', 'StopSignal': 'SIGTERM',
                                 'Entrypoint': ['/usr/local/bin/xmqr-admin'],
                                 'Labels': {'org.opencontainers.image.version': '0.1.0',
                                            'org.opencontainers.image.revision': 'accepted-fixture'}}}

    def test_accepts_exact_runtime_and_revision(self):
        self.assertEqual(validate_image(self.image, '0.1.0', 'accepted-fixture')['id'], 'sha256:fixture')

    def test_refuses_other_architectures_runtime_and_legacy_binary(self):
        for target, key, value in [('image', 'Os', 'windows'), ('image', 'Architecture', 'arm64'),
                                   ('config', 'User', 'root'), ('config', 'StopSignal', 'SIGKILL'),
                                   ('config', 'Entrypoint', ['/usr/local/bin/mqtt-admin'])]:
            wrong = copy.deepcopy(self.image)
            (wrong if target == 'image' else wrong['Config'])[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_image(wrong, '0.1.0', 'accepted-fixture')

    def test_refuses_mismatched_version_and_commit(self):
        for version, revision in [('0.1.1', 'accepted-fixture'), ('0.1.0', 'other-fixture')]:
            with self.subTest(version=version, revision=revision), self.assertRaises(ValueError):
                validate_image(self.image, version, revision)


if __name__ == '__main__':
    unittest.main()
