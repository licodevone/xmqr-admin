#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Independent startup/drain contracts, without a broker or host mutations."""
import io
from pathlib import Path
import time
import unittest
from unittest.mock import Mock, patch

import verify_channel_cli as CHANNEL


class LifecycleContracts(unittest.TestCase):
    def test_admin_socket_alone_is_not_mqtt_readiness(self):
        process = Mock(stdout=io.BytesIO(), returncode=None)
        process.poll.return_value = None
        endpoint = Mock()
        endpoint.exists.return_value = True
        peer = Mock()
        peer.__enter__ = Mock(return_value=peer)
        peer.__exit__ = Mock(return_value=False)
        peer.recv.return_value = b'\x20\x02\x00\x05'
        now = [0.0]
        def advance(delay):
            now[0] += delay
        with patch.object(CHANNEL.subprocess, 'Popen', return_value=process), \
                patch.object(CHANNEL.time, 'monotonic', side_effect=lambda: now[0]), \
                patch.object(CHANNEL.time, 'sleep', side_effect=advance), \
                patch.object(CHANNEL.socket, 'create_connection',
                             side_effect=[ConnectionRefusedError(), ConnectionRefusedError(), peer]) as connect:
            self.assertIs(CHANNEL.start(Path('owned-broker'), {'MQTT_BIND': '127.0.0.1:1883'}, endpoint), process)
        self.assertEqual(connect.call_count, 3)
        self.assertGreaterEqual(now[0], .04)
        peer.sendall.assert_called_once_with(b'\x10\x1b\x00\x04MQTT\x04\x02\x00\x1e\x00\x0fadmin-readiness')

    def startup(self, process, endpoint, probe, clock):
        guard = Mock()
        guard.thread.is_alive.return_value = False
        with patch.object(CHANNEL.subprocess, 'Popen', return_value=process), \
                patch.object(CHANNEL, 'OutputGuard', return_value=guard), \
                patch.object(CHANNEL.time, 'monotonic', side_effect=lambda: clock[0]), \
                patch.object(CHANNEL.time, 'sleep', side_effect=lambda delay: clock.__setitem__(0, clock[0] + delay)), \
                patch.object(CHANNEL, 'anonymous_refusal', side_effect=probe):
            return CHANNEL.start(Path('owned-broker'), {'MQTT_BIND': '127.0.0.1:1883'}, endpoint)

    def test_dead_process_is_not_ready(self):
        process = Mock()
        process.poll.return_value = 17
        with self.assertRaisesRegex(AssertionError, 'exited before readiness'):
            self.startup(process, Mock(), lambda *_: None, [0.0])
        process.kill.assert_not_called()

    def test_never_ready_uses_one_absolute_deadline_and_cleans_owned_child(self):
        process = Mock()
        process.poll.return_value = None
        endpoint = Mock()
        endpoint.exists.return_value = False
        clock = [0.0]
        with self.assertRaisesRegex(AssertionError, 'startup deadline'):
            self.startup(process, endpoint, lambda *_: None, clock)
        self.assertAlmostEqual(clock[0], 8.0)
        process.kill.assert_called_once_with()

    def test_response_after_deadline_is_not_ready(self):
        process = Mock()
        process.poll.return_value = None
        endpoint = Mock()
        endpoint.exists.return_value = True
        clock = [0.0]
        def late(*_):
            clock[0] = 8.001
        with self.assertRaises(AssertionError):
            self.startup(process, endpoint, late, clock)
        process.kill.assert_called_once_with()

    def test_invalid_or_truncated_mqtt_response_refused(self):
        for data in (b'\x20\x02\x00\x00', b'\x20\x01\x00\x05', b''):
            with self.subTest(data=data):
                peer = Mock()
                peer.__enter__ = Mock(return_value=peer)
                peer.__exit__ = Mock(return_value=False)
                peer.recv.return_value = data
                with patch.object(CHANNEL.socket, 'create_connection', return_value=peer), \
                        self.assertRaises(AssertionError):
                    CHANNEL.anonymous_refusal(('127.0.0.1', 1883), time.monotonic() + 1)

    def test_fragmented_valid_mqtt_refusal(self):
        peer = Mock()
        peer.__enter__ = Mock(return_value=peer)
        peer.__exit__ = Mock(return_value=False)
        peer.recv.side_effect = [b'\x20', b'\x02\x00', b'\x05']
        with patch.object(CHANNEL.socket, 'create_connection', return_value=peer):
            CHANNEL.anonymous_refusal(('127.0.0.1', 1883), time.monotonic() + 1)

    def test_log_drain_detects_split_token_without_retaining_log(self):
        class Chunks:
            def __init__(self):
                self.parts = iter([b'a' * 40, b'a' * 24, b''])
                self.closed = False
            def read(self, size):
                if size != 4096:
                    raise AssertionError('unbounded drain read')
                return next(self.parts)
            def close(self):
                self.closed = True
        stream = Chunks()
        guard = CHANNEL.OutputGuard(stream)
        with self.assertRaisesRegex(AssertionError, 'synthetic token'):
            guard.finish(time.monotonic() + 1)
        self.assertTrue(stream.closed)
        self.assertFalse(guard.thread.is_alive())

    def test_log_read_error_is_failure(self):
        stream = Mock()
        stream.read.side_effect = OSError('synthetic private stream fault')
        guard = CHANNEL.OutputGuard(stream)
        with self.assertRaisesRegex(AssertionError, 'read failure'):
            guard.finish(time.monotonic() + 1)
        stream.close.assert_called_once_with()

    def test_log_close_error_is_failure(self):
        stream = Mock()
        stream.read.return_value = b''
        stream.close.side_effect = OSError('synthetic close fault')
        guard = CHANNEL.OutputGuard(stream)
        with self.assertRaisesRegex(AssertionError, 'read failure'):
            guard.finish(time.monotonic() + 1)

    def test_failed_drain_start_cleans_owned_child(self):
        process = Mock()
        process.poll.return_value = None
        with patch.object(CHANNEL.subprocess, 'Popen', return_value=process), \
                patch.object(CHANNEL, 'OutputGuard', side_effect=RuntimeError('synthetic thread fault')), \
                self.assertRaisesRegex(RuntimeError, 'synthetic thread fault'):
            CHANNEL.start(Path('owned-broker'), {'MQTT_BIND': '127.0.0.1:1883'}, Mock())
        process.kill.assert_called_once_with()
        process.stdout.close.assert_called_once_with()

    def test_sigterm_exit_minus15_is_not_accepted(self):
        process = Mock(returncode=-15)
        process.poll.return_value = -15
        process.channel_output.thread.is_alive.return_value = False
        endpoint = Mock()
        endpoint.exists.return_value = False
        with self.assertRaisesRegex(AssertionError, 'exit=-15'):
            CHANNEL.stop(process, endpoint)

    def test_cleanup_does_not_mask_primary_failure(self):
        process = Mock()
        process.poll.return_value = None
        with patch.object(CHANNEL, 'stop', side_effect=AssertionError('synthetic cleanup fault')):
            with self.assertRaisesRegex(ValueError, 'primary fixture fault') as context:
                try:
                    raise ValueError('primary fixture fault')
                finally:
                    CHANNEL.finish_fixture(process, Mock())
        self.assertEqual(context.exception.__notes__, ['owned fixture cleanup failed: AssertionError'])


if __name__ == '__main__':
    unittest.main()
