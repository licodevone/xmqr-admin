#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A23: isolated HTTP/child-process contract; optional real mTLS broker/client.

Linux/WSL only. No installed services, real credentials, or operator state.
The report is created exclusively, including when a scenario fails.
"""
import argparse
from contextlib import contextmanager
import hashlib
import http.client
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import ssl
import subprocess
import sys
import tempfile
import threading
import time


FAKE = r'''#!/usr/bin/python3
import hashlib, json, os, pathlib, sys, time
root = pathlib.Path(os.environ['A23_FIXTURE_ROOT'])
args = sys.argv[1:]
mode = args[0]
def flag(name):
    return args[args.index(name) + 1]
def record(kind, **fields):
    with (root / 'calls.jsonl').open('a') as log:
        log.write(json.dumps(dict(kind=kind, pid=os.getpid(), **fields)) + '\n')
record('start', mode=mode, argv=args,
       inherited_secret='MQTT_CLIENT_PASSWORD_FILE' in os.environ)
if mode == 'pub':
    body = sys.stdin.buffer.read(8193)
    record('stdin', length=len(body), sha256=hashlib.sha256(body).hexdigest())
    if body == b'A23_FAIL':
        print('PRIVATE_REMOTE_DIAGNOSTIC_A23', file=sys.stderr, flush=True)
        sys.exit(7)
    if body == b'A23_TIMEOUT':
        time.sleep(30)
    qos = int(flag('--qos'))
    print(json.dumps(dict(event='publish_complete', qos=qos,
        confirmation='sent' if qos == 0 else 'broker_protocol_ack')), flush=True)
    sys.exit(0)
time.sleep(1.2)
print('Assinatura ativa. Aguardando mensagens; Ctrl+C para sair.', file=sys.stderr, flush=True)
record('suback')
def emit(payload, topic='a23/value'):
    print(json.dumps(dict(event='publish', topic=topic, qos=1, retain=False,
                         payload_bytes=list(payload))), flush=True)
emit(b'<img src=x onerror=alert(1)>')
emit(bytes([0, 255, 10]))
previous = ''
while True:
    try:
        control = (root / 'control').read_text()
    except FileNotFoundError:
        control = ''
    if control != previous:
        previous = control
        if control == 'burst':
            for value in range(230):
                emit(('a23-' + str(value)).encode())
        elif control == 'oversize':
            emit(b'x' * 4097)
        elif control == 'exit':
            print('PRIVATE_REMOTE_DIAGNOSTIC_A23', file=sys.stderr, flush=True)
            sys.exit(9)
    time.sleep(.03)
'''


def private(path, text, mode=0o600):
    path.write_text(text, encoding='utf-8')
    path.chmod(mode)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def port():
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 0))
        return probe.getsockname()[1]


def wait_for(check, seconds=8, message='condition timeout'):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        result = check()
        if result:
            return result
        time.sleep(.04)
    raise AssertionError(message)


def wait_port(process, number):
    def ready():
        assert process.poll() is None, 'owned process exited before readiness'
        try:
            with socket.create_connection(('127.0.0.1', number), timeout=.1):
                return True
        except OSError:
            return False
    wait_for(ready, 12, 'owned process did not bind')


def stop(process):
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=7)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)


def alive(pid):
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False


class Board:
    def __init__(self, number, token):
        self.number = number
        self.token = token
        self.origin = f'http://127.0.0.1:{number}'

    def request(self, path='/api/mqtt', method='GET', data=None, *, token=True,
                origin=True, host=None, headers=None, raw=None):
        fields = {'Host': host or f'127.0.0.1:{self.number}'}
        if token:
            fields['Authorization'] = 'Bearer ' + self.token
        if origin:
            fields['Origin'] = self.origin if origin is True else origin
        if data is not None:
            raw = json.dumps(data).encode()
            fields['Content-Type'] = 'application/json'
        fields.update(headers or {})
        connection = http.client.HTTPConnection('127.0.0.1', self.number, timeout=18)
        try:
            connection.request(method, path, body=raw, headers=fields)
            response = connection.getresponse()
            body = response.read(1024 * 1024)
            return response.status, dict(response.getheaders()), body
        finally:
            connection.close()

    def snapshot(self):
        status, _, body = self.request()
        assert status == 200, 'snapshot HTTP status'
        return json.loads(body)

    def post(self, action, data):
        return self.request('/api/mqtt/' + action, 'POST', data)


@contextmanager
def server(binary, root, *, enabled=True, client=None, profiles=None):
    number = port()
    token = secrets.token_hex(32)
    private(root / 'access.token', token)
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith(('XMQR_ADMIN_', 'MQTT_', 'A23_'))}
    environment.update(XMQR_ADMIN_BIND=f'127.0.0.1:{number}',
                       XMQR_ADMIN_MONITOR_BIND=f'127.0.0.1:{port()}',
                       XMQR_ADMIN_TOKEN_FILE=str(root / 'access.token'))
    if enabled:
        sub, pub = profiles or (root / 'sub.toml', root / 'pub.toml')
        environment.update(XMQR_ADMIN_MQTT_CLIENT_BIN=str(client or root / 'fake-client'),
                           XMQR_ADMIN_MQTT_SUB_PROFILE=str(sub),
                           XMQR_ADMIN_MQTT_PUB_PROFILE=str(pub),
                           XMQR_ADMIN_MQTT_LABEL='A23 isolated fixture',
                           A23_FIXTURE_ROOT=str(root),
                           MQTT_CLIENT_PASSWORD_FILE='PRIVATE_ENV_SENTINEL_A23')
    with (root / 'admin.log').open('ab') as log:
        process = subprocess.Popen([str(binary), 'serve'], env=environment,
                                   stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        try:
            wait_port(process, number)
            yield Board(number, token), process
        finally:
            stop(process)


def calls(root):
    path = root / 'calls.jsonl'
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


def sub_pids(root):
    return [row['pid'] for row in calls(root)
            if row['kind'] == 'start' and row['mode'] == 'sub']


def disconnected_children(root):
    return all(not alive(pid) for pid in sub_pids(root))


def run_fake(binary, root, scenario):
    private(root / 'fake-client', FAKE, 0o700)
    private(root / 'sub.toml', 'version=1\n[connection]\nhost="127.0.0.1"\n')
    private(root / 'pub.toml', 'version=1\n[connection]\nhost="127.0.0.1"\n')
    with server(binary, root, enabled=False) as (board, _):
        with scenario('console disabled without private profiles'):
            data = board.snapshot()
            assert data['enabled'] is False and data['status'] == 'disabled'
            assert board.post('connect', {'topic': 'a23/#', 'qos': 1})[0] == 503
    with server(binary, root) as (board, process):
        with scenario('exact Host Origin Bearer and security headers'):
            for kwargs, expected in [({'token': False}, 401),
                                     ({'host': 'evil.invalid'}, 403),
                                     ({'origin': 'http://evil.invalid'}, 403),
                                     ({'headers': {'Authorization': 'Bearer wrong'}}, 401)]:
                status, headers, body = board.request(**kwargs)
                assert status == expected, 'security denial status'
                assert headers.get('cache-control') == 'no-store'
                assert "default-src 'none'" in headers.get('content-security-policy', '')
                assert 'access-control-allow-origin' not in headers
                assert board.token.encode() not in body
            status, headers, _ = board.request('/')
            assert status == 200 and headers.get('x-content-type-options') == 'nosniff'
        with scenario('query body methods JSON origin and route negatives'):
            checks = [board.request('/api/mqtt?x=1'),
                      board.request(raw=b'bad'),
                      board.request(method='PUT'),
                      board.request('/api/mqtt/connect', 'POST', {}, origin=False),
                      board.request('/api/mqtt/connect', 'POST', raw=b'{',
                                    headers={'Content-Type': 'application/json'}),
                      board.request('/api/mqtt/connect', 'POST', raw=b'x' * 8193,
                                    headers={'Content-Type': 'application/json'}),
                      board.request('/api/mqtt/connect?x=1', 'POST', {}),
                      board.request('/api/mqtt/connect', 'POST', raw=b'0\r\n\r\n',
                                    headers={'Transfer-Encoding': 'chunked',
                                             'Content-Type': 'application/json'}),
                      board.request('/api/status', 'POST', {})]
            for status, _, _ in checks:
                assert status >= 400, 'invalid HTTP input accepted'
            assert not calls(root), 'invalid HTTP request spawned a child'
        with scenario('topic QoS payload and unknown-field bounds'):
            invalid = [('connect', {'topic': 'a23/#/bad', 'qos': 1}),
                       ('connect', {'topic': '', 'qos': 1}),
                       ('connect', {'topic': 'a23/#', 'qos': 3}),
                       ('connect', {'topic': 'a23/#', 'qos': 1, 'profile': '/tmp/evil'}),
                       ('publish', {'topic': 'a23/+', 'payload': 'x', 'qos': 1, 'retain': False}),
                       ('publish', {'topic': 'a23/value', 'payload': 'x' * 4097,
                                    'qos': 1, 'retain': False})]
            for action, data in invalid:
                assert board.post(action, data)[0] == 400, 'invalid MQTT input accepted'
            assert not calls(root), 'invalid MQTT request spawned a child'
        with scenario('connection becomes active only after real SUBACK marker'):
            status, _, body = board.post('connect', {'topic': 'a23/#', 'qos': 1})
            assert status == 200 and json.loads(body)['status'] == 'connecting'
            assert board.snapshot()['status'] == 'connecting'
            assert board.post('connect', {'topic': 'a23/#', 'qos': 1})[0] == 409
            wait_for(lambda: board.snapshot()['status'] == 'connected')
        with scenario('JSONL text binary representation and bounded message queue'):
            data = wait_for(lambda: (value if len(value['messages']) >= 2 else None)
                            if (value := board.snapshot()) else None)
            assert data['messages'][0]['payload'] == '<img src=x onerror=alert(1)>'
            assert data['messages'][1]['payload'] == '[payload binario]'
            assert all(row['direction'] == 'received' for row in data['messages'])
            assert all(row['qos'] == 1 and row['received_at'] > 0 for row in data['messages'])
            private(root / 'control', 'burst')
            data = wait_for(lambda: (value if value['received_total'] >= 232 else None)
                            if (value := board.snapshot()) else None)
            assert len(data['messages']) == 200 and data['dropped_total'] >= 32
            assert len({row['id'] for row in data['messages']}) == 200
            private(root / 'control', 'oversize')
            after = wait_for(lambda: (value if value['received_total'] >= 233 else None)
                             if (value := board.snapshot()) else None)
            assert len(after['messages']) == 200 and after['dropped_total'] >= 33
            assert all(len(row['payload'].encode()) <= 4096 for row in after['messages'])
        with scenario('direct argv private profiles stdin and publication acknowledgements'):
            marker = root / 'SHELL_MUST_NOT_RUN'
            payload = f'$(touch {marker}) ; <script>alert(1)</script>'
            topic = f'a23/$(touch {marker})'
            for qos in range(3):
                status, _, body = board.post('publish', dict(topic=topic, payload=payload,
                                                           qos=qos, retain=False))
                assert status == 200, 'publication failed'
                assert json.loads(body)['published_total'] >= qos + 1
            assert not marker.exists(), 'shell evaluated payload'
            rows = [row for row in calls(root) if row['kind'] == 'start']
            identities = {'sub': set(), 'pub': set()}
            for row in rows:
                args = row['argv']
                assert '--password' not in args and '--message' not in args
                assert '--count' not in args and args[args.index('--output') + 1] == 'jsonl'
                expected = 'sub.toml' if row['mode'] == 'sub' else 'pub.toml'
                assert args[args.index('--profile') + 1] == str(root / expected)
                assert not row['inherited_secret'], 'MQTT environment overrode private profile'
                assert payload not in args
                identity = args[args.index('--client-id') + 1]
                assert len(identity.encode()) <= 23
                identities[row['mode']].add(identity)
                if row['mode'] == 'pub':
                    assert '--stdin' in args
                    assert args[args.index('--topic') + 1] == topic
            assert len(identities['sub']) == len(identities['pub']) == 1
            assert identities['sub'].isdisjoint(identities['pub'])
            assert sum(row['kind'] == 'stdin' for row in calls(root)) == 3
        with scenario('CLI failure exposes only generic error and no successful publication'):
            before = board.snapshot()['published_total']
            status, _, body = board.post('publish', dict(topic='a23/value', payload='A23_FAIL',
                                                       qos=1, retain=False))
            assert status == 502 and b'PRIVATE_REMOTE_DIAGNOSTIC_A23' not in body
            assert board.snapshot()['published_total'] == before
        with scenario('publication concurrency bound and deadline reap owned child'):
            started = time.monotonic()
            count = len([row for row in calls(root) if row['kind'] == 'start'])
            response = []
            failures = []
            def timed_publication():
                try:
                    response.append(board.post('publish', dict(topic='a23/value', payload='A23_TIMEOUT',
                                                              qos=1, retain=False)))
                except Exception as failure:
                    failures.append(failure)
            task = threading.Thread(target=timed_publication, daemon=True)
            task.start()
            wait_for(lambda: len([row for row in calls(root) if row['kind'] == 'start']) > count)
            assert board.post('publish', dict(topic='a23/value', payload='a23-concurrent',
                                             qos=1, retain=False))[0] == 429
            task.join(timeout=16)
            if failures:
                raise failures[0]
            assert not task.is_alive() and response, 'publication thread deadline'
            status, _, _ = response[0]
            assert status == 502 and time.monotonic() - started < 10
            pubs = [row for row in calls(root) if row['kind'] == 'start' and row['mode'] == 'pub']
            wait_for(lambda: all(not alive(row['pid']) for row in pubs))
        with scenario('disconnect and logout cleanup are idempotent'):
            for _ in range(2):
                assert board.post('disconnect', {})[0] == 200
            wait_for(lambda: disconnected_children(root))
            assert board.snapshot()['status'] == 'disconnected'
        with scenario('unexpected subscriber exit becomes visible generic error'):
            private(root / 'control', '')
            assert board.post('connect', {'topic': 'a23/#', 'qos': 1})[0] == 200
            wait_for(lambda: board.snapshot()['status'] == 'connected')
            private(root / 'control', 'exit')
            data = wait_for(lambda: (value if value['status'] == 'error' else None)
                            if (value := board.snapshot()) else None)
            assert 'PRIVATE_REMOTE_DIAGNOSTIC_A23' not in json.dumps(data)
            assert board.post('disconnect', {})[0] == 200
        with scenario('subscription lease expires without browser polling'):
            private(root / 'control', '')
            assert board.post('connect', {'topic': 'a23/#', 'qos': 1})[0] == 200
            wait_for(lambda: board.snapshot()['status'] == 'connected')
            wait_for(lambda: disconnected_children(root), 68, 'subscription lease did not reap child')
            assert board.snapshot()['status'] != 'connected'
        with scenario('server shutdown reaps owned subscriber'):
            assert board.post('connect', {'topic': 'a23/#', 'qos': 1})[0] == 200
            wait_for(lambda: board.snapshot()['status'] == 'connected')
            stop(process)
            wait_for(lambda: disconnected_children(root))


def openssl(root, *args):
    subprocess.run(['openssl', *map(str, args)], cwd=root, capture_output=True,
                   check=True, timeout=20)


def certificate_fixture(root):
    openssl(root, 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
            '-keyout', 'ca.key', '-out', 'ca.crt', '-subj', '/CN=A23-fixture-CA',
            '-addext', 'basicConstraints=critical,CA:TRUE',
            '-addext', 'keyUsage=critical,keyCertSign,cRLSign')
    for serial, name, usage, san in [(2, 'server', 'serverAuth', 'IP:127.0.0.1'),
                                     (3, 'client', 'clientAuth', 'DNS:a23.fixture')]:
        openssl(root, 'req', '-new', '-newkey', 'rsa:2048', '-nodes',
                '-keyout', f'{name}.key', '-out', f'{name}.csr', '-subj', f'/CN={name}')
        private(root / f'{name}.ext', 'basicConstraints=critical,CA:FALSE\n'
                'keyUsage=critical,digitalSignature,keyEncipherment\n'
                f'extendedKeyUsage={usage}\nsubjectAltName={san}\n')
        openssl(root, 'x509', '-req', '-in', f'{name}.csr', '-CA', 'ca.crt',
                '-CAkey', 'ca.key', '-set_serial', serial, '-days', '1',
                '-extfile', f'{name}.ext', '-out', f'{name}.crt')
    for path in root.iterdir():
        if path.is_file():
            path.chmod(0o600)


def fixture_password_hash(admin, password):
    # PTY is required by rpassword; no shell and no credentials in argv.
    import pty
    import re
    import select
    import termios
    pid, descriptor = pty.fork()
    if pid == 0:
        os.execl(str(admin), str(admin), 'hash-password')
    output, sent, reaped = b'', 0, False
    deadline = time.monotonic() + 15
    try:
        while time.monotonic() < deadline:
            if select.select([descriptor], [], [], .1)[0]:
                try:
                    chunk = os.read(descriptor, 4096)
                except OSError:
                    break
                if not chunk:
                    break
                output += chunk
            expected = b'Password: ' if sent == 0 else b'Confirm password: '
            if sent < 2 and expected in output:
                while termios.tcgetattr(descriptor)[3] & termios.ECHO:
                    assert time.monotonic() < deadline, 'password prompt deadline'
                    time.sleep(.01)
                os.write(descriptor, password.encode() + b'\n')
                sent += 1
        waited, status = os.waitpid(pid, os.WNOHANG)
        while waited == 0 and time.monotonic() < deadline:
            time.sleep(.01)
            waited, status = os.waitpid(pid, os.WNOHANG)
        assert waited == pid, 'password hash process deadline'
        reaped = True
        match = re.search(rb'password_hash = "(\$argon2id\$[^"\r\n]+)"', output)
        assert status == 0 and match and password.encode() not in output
        return match.group(1).decode()
    finally:
        os.close(descriptor)
        if not reaped:
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            os.waitpid(pid, 0)


def run_real(admin, client, broker, root, scenario):
    with scenario('real mTLS fixture certificates and private operator-free policy'):
        certificate_fixture(root)
        password = secrets.token_urlsafe(24)
        private(root / 'password', password)
        hashed = fixture_password_hash(admin, password)
        fingerprint = hashlib.sha256(ssl.PEM_cert_to_DER_cert(
            (root / 'client.crt').read_text())).hexdigest()
        private(root / 'security.toml', 'version=1\n[[users]]\nusername="a23-fixture"\n'
                f'password_hash="{hashed}"\ncert_sha256="sha256:{fingerprint}"\n'
                'publish=["a23/value"]\nsubscribe=["a23/#"]\n')
        mqtt_port = port()
        private(root / 'broker.toml', 'version=1\n[broker]\nmode="secure-mtls"\n'
                f'bind="127.0.0.1:{mqtt_port}"\nstate_dir="state"\n'
                '[tls]\nserver_cert="server.crt"\nserver_key="server.key"\nclient_ca="ca.crt"\n'
                '[security]\nbundle="security.toml"\n')
        for name in ['sub', 'pub']:
            private(root / f'{name}.toml', 'version=1\n[connection]\nhost="127.0.0.1"\n'
                    f'port={mqtt_port}\nusername="a23-fixture"\nca="ca.crt"\n'
                    'cert="client.crt"\nkey="client.key"\npassword_file="password"\n'
                    'protocol_version="5"\n')
    environment = {key: value for key, value in os.environ.items() if not key.startswith('MQTT_')}
    environment['MQTT_CONFIG'] = str(root / 'broker.toml')
    with (root / 'broker.log').open('ab') as log:
        child = subprocess.Popen([str(broker)], env=environment, stdin=subprocess.DEVNULL,
                                 stdout=log, stderr=log)
        try:
            wait_port(child, mqtt_port)
            with server(admin, root, client=client) as (board, _):
                with scenario('real mTLS SUBACK and continuous QoS 0 1 2 publication and reception'):
                    assert board.post('connect', {'topic': 'a23/#', 'qos': 2})[0] == 200
                    wait_for(lambda: board.snapshot()['status'] == 'connected', 12)
                    for qos in range(3):
                        payload = f'a23-isolated-qos-{qos}'
                        assert board.post('publish', dict(topic='a23/value', payload=payload,
                                                         qos=qos, retain=False))[0] == 200
                        wait_for(lambda: any(row['direction'] == 'received' and row['payload'] == payload
                                             for row in board.snapshot()['messages']), 12)
                    assert board.snapshot()['published_total'] == 3
                with scenario('real broker ACL denial is not counted as publication success'):
                    status, _, _ = board.post('publish', dict(topic='a23/forbidden', payload='a23-denied',
                                                            qos=1, retain=False))
                    assert status == 502 and board.snapshot()['published_total'] == 3
                with scenario('real subscriber disconnect keeps broker alive'):
                    assert board.post('disconnect', {})[0] == 200
                    assert board.snapshot()['status'] == 'disconnected' and child.poll() is None
        finally:
            stop(child)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--admin', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--client', type=Path)
    parser.add_argument('--broker', type=Path)
    args = parser.parse_args()
    if os.name != 'posix':
        parser.error('Linux/WSL is required')
    if bool(args.client) != bool(args.broker):
        parser.error('--client and --broker must be supplied together')
    args.admin = args.admin.resolve(strict=True)
    if args.client:
        args.client, args.broker = args.client.resolve(strict=True), args.broker.resolve(strict=True)
    # Reserve the report first: never overwrite prior PASS/FAIL evidence.
    with args.report.open('x', encoding='utf-8') as report:
        result = dict(schema='xmqr-admin-mqtt-board-a23-v1', status='FAIL',
                      started_unix=int(time.time()),
                      hashes=dict(admin_elf_sha256=digest(args.admin),
                                  fixture_sha256=digest(Path(__file__).resolve()),
                                  fake_client_sha256=hashlib.sha256(FAKE.encode()).hexdigest()),
                      real_integration='not requested', scenarios=[])
        if args.client:
            result['hashes'].update(client_elf_sha256=digest(args.client),
                                    broker_elf_sha256=digest(args.broker))
            result['real_integration'] = 'requested'

        @contextmanager
        def scenario(name):
            entry = dict(name=name, status='FAIL')
            result['scenarios'].append(entry)
            try:
                yield
                entry['status'] = 'PASS'
                print('PASS:', name, flush=True)
            except Exception as failure:
                entry['error_type'] = type(failure).__name__
                trace = failure.__traceback__
                while trace and trace.tb_next:
                    trace = trace.tb_next
                if trace:
                    entry['error_line'] = trace.tb_lineno
                if isinstance(failure, AssertionError):
                    entry['reason'] = str(failure)[:160]
                raise

        try:
            with tempfile.TemporaryDirectory(prefix='xmqr-admin-a23-') as directory:
                root = Path(directory)
                root.chmod(0o700)
                fake = root / 'fake'
                fake.mkdir(mode=0o700)
                run_fake(args.admin, fake, scenario)
                if args.client:
                    real = root / 'real'
                    real.mkdir(mode=0o700)
                    run_real(args.admin, args.client, args.broker, real, scenario)
                    result['real_integration'] = 'PASS'
            result['status'] = 'PASS'
        except Exception as failure:
            result['error_type'] = type(failure).__name__
            print('FAIL: fixture stopped; see immutable report (no payloads/secrets).', file=sys.stderr)
        finally:
            result['finished_unix'] = int(time.time())
            result['passed'] = sum(row['status'] == 'PASS' for row in result['scenarios'])
            result['total'] = len(result['scenarios'])
            json.dump(result, report, indent=2, ensure_ascii=False)
            report.write('\n')
        return 0 if result['status'] == 'PASS' else 1


if __name__ == '__main__':
    sys.exit(main())
