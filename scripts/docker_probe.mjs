// SPDX-License-Identifier: MIT
// Runs only inside the owned shared loopback namespace; never logs the token.
import { readFileSync } from 'node:fs';
import { request as httpRequest } from 'node:http';

const mode = process.argv[2];
const token = readFileSync('/work/config/access.token', 'utf8').trim();
const url = 'http://127.0.0.1:9080';
const checks = [];
let phase = 'startup';
function requireCheck(condition) {
  if (!condition) throw new Error('owned HTTP probe assertion failed');
}
async function request(path, options = {}) {
  // fetch replaces a custom Host; raw HTTP is needed for the Host negative.
  const { response, body } = await new Promise((resolve, reject) => {
    const req = httpRequest(url + path, { method: options.method || 'GET',
      headers: options.headers, signal: AbortSignal.timeout(5000) }, res => {
      const chunks = [];
      let bytes = 0;
      res.on('error', reject);
      res.on('data', chunk => {
        bytes += chunk.length;
        if (bytes > 100000) { req.destroy(new Error('owned response bound')); return; }
        chunks.push(chunk);
      });
      res.on('end', () => resolve({ response: { status: res.statusCode,
        headers: { get: key => res.headers[key] || null, has: key => key in res.headers } },
        body: Buffer.concat(chunks).toString('utf8') }));
    });
    req.on('error', reject);
    req.end(options.body);
  });
  requireCheck(!body.includes(token));
  requireCheck(response.headers.get('cache-control') === 'no-store');
  requireCheck(!response.headers.has('access-control-allow-origin'));
  return { response, body };
}
try {
  requireCheck(['live', 'stopped'].includes(mode) && /^[a-f0-9]{64}$/.test(token));
  for (let attempt = 0; attempt < 80; attempt++) {
    try { await request('/'); break; }
    catch { if (attempt === 79) throw new Error('owned panel unavailable');
      await new Promise(resolve => setTimeout(resolve, 100)); }
  }
  phase = 'authenticated aggregate';
  const { response, body } = await request('/api/status', { headers: { Authorization: 'Bearer ' + token } });
  requireCheck(response.status === 200);
  const status = JSON.parse(body);
  if (mode === 'live') {
    requireCheck(status.reachable && status.ready);
    requireCheck(status.metrics.xmqr_retained_messages === 1 && status.metrics.xmqr_persistence_commits_total >= 1);
    checks.push('real retained publication observed in shared loopback namespace');
    for (const [label, options, code, path] of [
      ['missing bearer', {}, 401],
      ['wrong bearer', { headers: { Authorization: 'Bearer ' + 'b'.repeat(64) } }, 401],
      ['foreign origin', { headers: { Authorization: 'Bearer ' + token, Origin: 'http://evil.test' } }, 403],
      ['foreign host', { headers: { Authorization: 'Bearer ' + token, Host: 'evil.test:9080' } }, 403],
      ['write method', { method: 'POST', headers: { Authorization: 'Bearer ' + token } }, 405],
      ['query denied', { headers: { Authorization: 'Bearer ' + token } }, 400, '/api/status?fixture=1'],
      ['body denied', { method: 'POST', body: 'fixture', headers: { Authorization: 'Bearer ' + token } }, 405],
    ]) {
      phase = label;
      const value = await request(path || '/api/status', options);
      requireCheck(value.response.status === code);
      checks.push(label);
    }
    phase = 'shell';
    const shell = await request('/');
    requireCheck(shell.response.status === 200 && shell.body.includes('XMQR Admin'));
    requireCheck(shell.response.headers.get('content-security-policy').includes("frame-ancestors 'none'"));
    checks.push('public shell security headers');
  } else {
    requireCheck(!status.reachable && !status.ready && Object.keys(status.metrics).length === 0);
    checks.push('stopped broker shown unavailable without invented zeros');
  }
  process.stdout.write(JSON.stringify({ status: 'PASS', checks }) + '\n');
} catch {
  process.stderr.write('Owned Docker HTTP probe failed at ' + phase + '; secret diagnostics suppressed.\n');
  process.exitCode = 1;
}
