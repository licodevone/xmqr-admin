#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A24: isolated Linux HTTP authentication, authorization and persistence checks.

No installed service, real user, MQTT credential, or operator database is used.
Never overwrite reports or print test passwords, cookies, CSRFs or KDF hashes.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
from contextlib import contextmanager
import hashlib
import http.client
from http.cookies import SimpleCookie
import json
import os
from pathlib import Path
import secrets
import socket
import stat
import subprocess
import sys
import tempfile
import time


def private(path, value):
    path.write_text(value, encoding='utf-8')
    path.chmod(0o600)


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def port():
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def stop(child):
    if child.poll() is None:
        child.terminate()
        try:
            child.wait(timeout=7)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=3)


def wait_port(child, number):
    deadline = time.monotonic() + 12
    while time.monotonic() < deadline:
        assert child.poll() is None, 'owned server exited before readiness'
        try:
            with socket.create_connection(('127.0.0.1', number), timeout=.1):
                return
        except OSError:
            time.sleep(.03)
    raise AssertionError('owned server readiness deadline')


def environment(root, number, users=None):
    fields = {key: value for key, value in os.environ.items()
              if not key.startswith(('XMQR_ADMIN_', 'MQTT_'))}
    fields.update(XMQR_ADMIN_BIND=f'127.0.0.1:{number}',
                  XMQR_ADMIN_MONITOR_BIND=f'127.0.0.1:{port()}',
                  XMQR_ADMIN_TOKEN_FILE=str(root / 'access.token'))
    if users is not None:
        fields['XMQR_ADMIN_WEB_USERS_FILE'] = str(users)
    return fields


@contextmanager
def server(admin, root, users=None):
    token = secrets.token_hex(32)
    private(root / 'access.token', token)
    number = port()
    with (root / 'server.log').open('ab') as log:
        child = subprocess.Popen([str(admin), 'serve'], cwd=root,
                                 env=environment(root, number, users),
                                 stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        try:
            wait_port(child, number)
            yield Browser(number, token), child
        finally:
            stop(child)


class Browser:
    def __init__(self, number, token):
        self.number, self.token = number, token
        self.cookie = None
        self.csrf = None

    def clone(self):
        other = Browser(self.number, self.token)
        other.cookie, other.csrf = self.cookie, self.csrf
        return other

    def request(self, path, method='GET', data=None, *, bearer=False,
                cookie=True, csrf=True, origin=True, headers=None, raw=None):
        fields = {'Host': f'127.0.0.1:{self.number}'}
        if origin:
            fields['Origin'] = f'http://127.0.0.1:{self.number}' if origin is True else origin
        if bearer:
            fields['Authorization'] = 'Bearer ' + self.token
        if cookie and self.cookie:
            fields['Cookie'] = self.cookie
        if method == 'POST' and csrf and self.csrf:
            fields['X-XMQR-CSRF'] = self.csrf if csrf is True else csrf
        if data is not None:
            raw = json.dumps(data).encode()
            fields['Content-Type'] = 'application/json'
        fields.update(headers or {})
        connection = http.client.HTTPConnection('127.0.0.1', self.number, timeout=15)
        try:
            connection.request(method, path, body=raw, headers=fields)
            response = connection.getresponse()
            body = response.read(1024 * 1024)
            return response.status, dict(response.getheaders()), body
        finally:
            connection.close()

    def json(self, path, method='GET', data=None, **kwargs):
        status, headers, body = self.request(path, method, data, **kwargs)
        return status, headers, json.loads(body) if body else None

    def authenticated(self, reply):
        status, headers, body = reply
        assert status == 200, 'authentication status'
        info = json.loads(body)
        assert set(info) == {'username', 'display_name', 'role', 'csrf_token'}, 'session fields'
        assert isinstance(info['csrf_token'], str) and len(info['csrf_token']) >= 32
        jar = SimpleCookie()
        jar.load(headers.get('set-cookie', ''))
        assert len(jar) == 1 and 'xmqr_session' in jar, 'session cookie missing'
        cookie = jar['xmqr_session']
        assert cookie['httponly'] and cookie['samesite'].lower() == 'strict'
        assert cookie['path'] == '/' and cookie['max-age'] == '28800'
        assert len(cookie.value) == 64 and all(c in '0123456789abcdef' for c in cookie.value)
        assert self.token not in cookie.value
        assert headers.get('cache-control') == 'no-store'
        self.cookie = 'xmqr_session=' + cookie.value
        self.csrf = info['csrf_token']
        return info


def rejected_startup(admin, root, users):
    number = port()
    if not (root / 'access.token').exists():
        private(root / 'access.token', secrets.token_hex(32))
    with (root / 'rejected.log').open('ab') as log:
        child = subprocess.Popen([str(admin), 'serve'], cwd=root,
                                 env=environment(root, number, users),
                                 stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        try:
            try:
                code = child.wait(timeout=5)
            except subprocess.TimeoutExpired as failure:
                raise AssertionError('unsafe startup was not rejected') from failure
            assert code != 0, 'unsafe startup returned success'
        finally:
            stop(child)


def user_payload(username, password, role='viewer'):
    return dict(username=username, display_name='A24 fixture', password=password, role=role)


def run(admin, root, scenario):
    password = secrets.token_urlsafe(30)
    viewer_password = secrets.token_urlsafe(30)
    operator_password = secrets.token_urlsafe(30)
    users = root / 'web-users.json'
    with server(admin, root, users) as (browser, child):
        with scenario('public configuration and anonymous APIs do not expose protected data'):
            status, headers, data = browser.json('/api/auth/config', origin=False)
            assert status == 200 and data == dict(mode='users', setup_required=True)
            assert headers.get('cache-control') == 'no-store'
            assert "default-src 'none'" in headers.get('content-security-policy', '')
            for path in ['/api/auth/session', '/api/status', '/api/mqtt', '/api/auth/users']:
                assert browser.request(path, origin=False)[0] == 401, 'anonymous protected endpoint'
                assert browser.request(path, bearer=True)[0] == 401, 'legacy bearer bypassed login'
        with scenario('Host Origin query method schema and body limits precede authentication'):
            invalid = [browser.request('/api/auth/config', headers={'Host': 'evil.invalid'}),
                       browser.request('/api/auth/config', origin='http://evil.invalid'),
                       browser.request('/api/auth/config?token=forbidden'),
                       browser.request('/api/auth/config', raw=b'x'),
                       browser.request('/api/auth/login', 'POST', {}, origin=False),
                       browser.request('/api/auth/login', method='PUT'),
                       browser.request('/api/auth/login', 'POST', raw=b'{',
                                       headers={'Content-Type': 'application/json'}),
                       browser.request('/api/auth/login', 'POST', dict(username='x', password='y', extra=1)),
                       browser.request('/api/auth/login', 'POST', raw=b'x' * 8193,
                                       headers={'Content-Type': 'application/json'})]
            expected = [403, 403, 400, 400, 403, 405, 400, 400, 413]
            assert [entry[0] for entry in invalid] == expected, 'HTTP validation status'
        with scenario('bootstrap rejects an incorrect local token without creating users'):
            request = dict(bootstrap_token=secrets.token_hex(32), username='Admin_A24',
                           display_name='Administrador A24', password=password)
            assert browser.request('/api/auth/setup', 'POST', request)[0] == 401
            assert browser.json('/api/auth/config')[2]['setup_required'] is True
            if users.exists():
                assert json.loads(users.read_text())['users'] == []
        with scenario('single bootstrap creates normalized administrator and opaque secure cookie'):
            request['bootstrap_token'] = browser.token
            info = browser.authenticated(browser.request('/api/auth/setup', 'POST', request))
            assert info['username'] == 'admin_a24' and info['role'] == 'admin'
            assert browser.json('/api/auth/config')[2]['setup_required'] is False
            assert browser.request('/api/auth/setup', 'POST', request)[0] == 409
        with scenario('persistent private database contains Argon2id and no original secrets or sessions'):
            raw = users.read_bytes()
            assert stat.S_IMODE(users.stat().st_mode) == 0o600 and not users.is_symlink()
            assert b'$argon2id$' in raw
            for value in [password, viewer_password, operator_password, browser.token,
                          browser.csrf, browser.cookie.split('=', 1)[1]]:
                assert value.encode() not in raw, 'secret persisted in clear text'
            assert browser.json('/api/auth/session')[2]['username'] == 'admin_a24'
        with scenario('user listing hides KDF hashes and admin-only account creation validates schema'):
            status, _, listed = browser.json('/api/auth/users')
            assert status == 200 and len(listed) == 1
            assert set(listed[0]) == {'username', 'display_name', 'role', 'enabled'}
            invalid_users = [user_payload('bad name', password), user_payload('a', password),
                             user_payload('valid_a24', 'short'), user_payload('valid_a24', 'x' * 129),
                             user_payload('valid_a24', 'control\nnotallowed'),
                             user_payload('valid_a24', password, 'superadmin')]
            for value in invalid_users:
                assert browser.request('/api/auth/users', 'POST', value)[0] == 400
            assert browser.request('/api/auth/users', 'POST', user_payload('viewer_a24', viewer_password))[0] == 200
            assert browser.request('/api/auth/users', 'POST', user_payload('operator_a24', operator_password,
                                                                          'operator'))[0] == 200
            assert len(browser.json('/api/auth/users')[2]) == 3
            stored = users.read_bytes()
            assert viewer_password.encode() not in stored and operator_password.encode() not in stored
            records = json.loads(stored)['users']
            assert all(row['password_hash'].startswith('$argon2id$') for row in records)
            assert len({row['password_hash'].split('$')[4] for row in records}) == 3, 'per-user KDF salts'
        with scenario('missing or incorrect CSRF cannot mutate users or MQTT operations'):
            for csrf in [False, 'incorrect-csrf']:
                assert browser.request('/api/auth/users/state', 'POST',
                                       dict(username='viewer_a24', enabled=False), csrf=csrf)[0] == 403
                assert browser.request('/api/mqtt/disconnect', 'POST', {}, csrf=csrf)[0] == 403
            assert all(row['enabled'] for row in browser.json('/api/auth/users')[2])
        with scenario('wrong and absent-user login failures are generic without credential disclosure'):
            wrong = browser.request('/api/auth/login', 'POST',
                                    dict(username='admin_a24', password=secrets.token_urlsafe(30)), cookie=False)
            absent = browser.request('/api/auth/login', 'POST',
                                     dict(username='absent_a24', password=secrets.token_urlsafe(30)), cookie=False)
            assert wrong[0] == absent[0] == 401 and wrong[2] == absent[2]
            assert b'argon2' not in wrong[2] and password.encode() not in wrong[2]
        viewer, operator = Browser(browser.number, browser.token), Browser(browser.number, browser.token)
        with scenario('viewer can consult monitor but cannot access console messages or users'):
            info = viewer.authenticated(viewer.request('/api/auth/login', 'POST',
                                                      dict(username='viewer_a24', password=viewer_password)))
            assert info['role'] == 'viewer' and viewer.request('/api/status')[0] == 200
            for path in ['/api/mqtt', '/api/auth/users']:
                assert viewer.request(path)[0] == 403
            for path, value in [('/api/mqtt/connect', dict(topic='a24/#', qos=1)),
                                ('/api/mqtt/disconnect', {}),
                                ('/api/mqtt/publish', dict(topic='a24/value', payload='a24', qos=1, retain=False)),
                                ('/api/auth/users', user_payload('elevated_a24', password, 'admin')),
                                ('/api/auth/users/state', dict(username='operator_a24', enabled=False))]:
                assert viewer.request(path, 'POST', value)[0] == 403
        with scenario('operator can access MQTT but cannot list create or disable web users'):
            info = operator.authenticated(operator.request('/api/auth/login', 'POST',
                                                          dict(username='operator_a24', password=operator_password)))
            assert info['role'] == 'operator'
            assert operator.request('/api/status')[0] == operator.request('/api/mqtt')[0] == 200
            assert operator.request('/api/mqtt/disconnect', 'POST', {})[0] == 200
            assert operator.request('/api/auth/users')[0] == 403
            assert operator.request('/api/auth/users', 'POST',
                                    user_payload('elevated_a24', password, 'admin'))[0] == 403
            assert operator.request('/api/auth/users/state', 'POST',
                                    dict(username='admin_a24', enabled=False))[0] == 403
        with scenario('last enabled administrator cannot be disabled'):
            assert browser.request('/api/auth/users/state', 'POST',
                                   dict(username='admin_a24', enabled=False))[0] == 409
            assert browser.request('/api/auth/users')[0] == 200
        with scenario('disabling a web user revokes all existing cookies and prevents login'):
            another = Browser(browser.number, browser.token)
            another.authenticated(another.request('/api/auth/login', 'POST',
                                                  dict(username='operator_a24', password=operator_password)))
            assert browser.request('/api/auth/users/state', 'POST',
                                   dict(username='operator_a24', enabled=False))[0] == 200
            for session in [operator, another]:
                assert session.request('/api/auth/session')[0] == 401
                assert session.request('/api/mqtt')[0] == 401
            disabled = operator.request('/api/auth/login', 'POST',
                                        dict(username='operator_a24', password=operator_password), cookie=False)
            assert disabled[0] == 401 and disabled[2] == wrong[2]
        with scenario('logout expires the cookie revokes the session and requires new login'):
            stale_viewer = viewer.clone()
            status, headers, data = viewer.json('/api/auth/logout', 'POST', {})
            assert status == 200 and data == {'ok': True}
            assert 'max-age=0' in headers.get('set-cookie', '').lower()
            assert stale_viewer.request('/api/auth/session')[0] == 401
            fresh = Browser(browser.number, browser.token)
            fresh.authenticated(fresh.request('/api/auth/login', 'POST',
                                             dict(username='ADMIN_A24', password=password)))
            assert fresh.cookie != browser.cookie and fresh.csrf != browser.csrf
        with scenario('a second server cannot open the same active web-user store'):
            before = users.read_bytes()
            rejected_startup(admin, root, users)
            assert child.poll() is None and users.read_bytes() == before
            assert browser.request('/api/auth/session')[0] == 200
        stale_cookie, stale_csrf = browser.cookie, browser.csrf
    with server(admin, root, users) as (restarted, _):
        with scenario('restart preserves users but invalidates prior session cookies'):
            restarted.cookie, restarted.csrf = stale_cookie, stale_csrf
            assert restarted.request('/api/auth/session')[0] == 401
            assert restarted.json('/api/auth/config')[2]['setup_required'] is False
            restarted.authenticated(restarted.request('/api/auth/login', 'POST',
                                                      dict(username='admin_a24', password=password), cookie=False))
            listed = restarted.json('/api/auth/users')[2]
            assert len(listed) == 3
            assert next(row for row in listed if row['username'] == 'operator_a24')['enabled'] is False
    with scenario('corrupt oversized symlink and unsafe modes are refused without resetting users'):
        for name, content, mode in [('corrupt', '{broken', 0o600),
                                     ('oversized', '{"version":1,"users":[]}' + ' ' * 65536, 0o600),
                                     ('public-mode', '{"version":1,"users":[]}', 0o644)]:
            leaf = root / name
            leaf.mkdir(mode=0o700)
            private(leaf / 'users.json', content)
            (leaf / 'users.json').chmod(mode)
            before = (leaf / 'users.json').read_bytes()
            rejected_startup(admin, leaf, leaf / 'users.json')
            assert (leaf / 'users.json').read_bytes() == before
        linked = root / 'linked'
        linked.mkdir(mode=0o700)
        private(linked / 'target.json', '{"version":1,"users":[]}')
        (linked / 'users.json').symlink_to(linked / 'target.json')
        rejected_startup(admin, linked, linked / 'users.json')
        assert (linked / 'users.json').is_symlink()
        assert (linked / 'target.json').read_text() == '{"version":1,"users":[]}'
        unsafe = root / 'unsafe-parent'
        unsafe.mkdir(mode=0o755)
        # Token remains in the safe root so this proves the users-parent check.
        rejected_startup(admin, root, unsafe / 'users.json')
        assert not (unsafe / 'users.json').exists()
    with scenario('sixty-four users are accepted and user sixty-five is refused without rewriting data'):
        limited = root / 'limit'
        limited.mkdir(mode=0o700)
        prototype = next(row for row in json.loads(users.read_text())['users']
                         if row['username'] == 'admin_a24')
        store = {'version': 1, 'users': []}
        for index in range(64):
            row = dict(prototype, username='limit_admin' if index == 0 else f'fixture_{index:03}',
                       display_name='A24 limit fixture', role='admin' if index == 0 else 'viewer', enabled=True)
            store['users'].append(row)
        limited_users = limited / 'users.json'
        private(limited_users, json.dumps(store))
        before = limited_users.read_bytes()
        with server(admin, limited, limited_users) as (bounded, _):
            bounded.authenticated(bounded.request('/api/auth/login', 'POST',
                                                  dict(username='limit_admin', password=password)))
            assert len(bounded.json('/api/auth/users')[2]) == 64
            assert bounded.request('/api/auth/users', 'POST',
                                   user_payload('fixture_065', secrets.token_urlsafe(30)))[0] in (400, 409)
            assert limited_users.read_bytes() == before
    with scenario('parallel login attempts are bounded rejected and leave public HTTP responsive'):
        flood = root / 'flood'
        flood.mkdir(mode=0o700)
        flood_users = flood / 'users.json'
        private(flood_users, users.read_text())
        with server(admin, flood, flood_users) as (bounded, _):
            def attempt(_):
                return Browser(bounded.number, bounded.token).request('/api/auth/login', 'POST',
                                                                      dict(username='absent_a24',
                                                                           password=secrets.token_urlsafe(30)))[0]
            started = time.monotonic()
            with ThreadPoolExecutor(max_workers=12) as pool:
                statuses = list(pool.map(attempt, range(24)))
            assert all(code in (401, 429) for code in statuses), 'parallel login result'
            assert 429 in statuses and time.monotonic() - started < 15
            assert bounded.json('/api/auth/config')[0] == 200


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--admin', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if os.name != 'posix':
        parser.error('Linux/WSL is required')
    args.admin = args.admin.resolve(strict=True)
    with args.report.open('x', encoding='utf-8') as report:
        result = dict(schema='xmqr-admin-web-auth-a24-v1', status='FAIL',
                      started_unix=int(time.time()),
                      hashes=dict(admin_elf_sha256=digest(args.admin),
                                  fixture_sha256=digest(Path(__file__).resolve())), scenarios=[])

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
            with tempfile.TemporaryDirectory(prefix='xmqr-admin-a24-') as directory:
                root = Path(directory)
                root.chmod(0o700)
                run(args.admin, root, scenario)
            result['status'] = 'PASS'
        except Exception as failure:
            result['error_type'] = type(failure).__name__
            print('FAIL: authentication fixture stopped; see immutable sanitized report.', file=sys.stderr)
        finally:
            result['finished_unix'] = int(time.time())
            result['passed'] = sum(row['status'] == 'PASS' for row in result['scenarios'])
            result['total'] = len(result['scenarios'])
            json.dump(result, report, indent=2, ensure_ascii=False)
            report.write('\n')
        return 0 if result['status'] == 'PASS' else 1


if __name__ == '__main__':
    sys.exit(main())
