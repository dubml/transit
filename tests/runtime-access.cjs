// Real process checks for default ports and deployment-owned access settings.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const https = require('node:https');
const { spawn, execFileSync } = require('node:child_process');
const { once } = require('node:events');
const { chromium } = require('playwright');
const root = path.resolve(__dirname, '..');
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function request(url, options = {}) {
  return new Promise((resolve, reject) => {
    const req = (url.startsWith('https:') ? https : http).request(url, {rejectUnauthorized: false, ...options}, res => {
      const chunks = [];
      res.on('data', chunk => chunks.push(chunk));
      res.on('end', () => resolve({status: res.statusCode, body: Buffer.concat(chunks).toString()}));
    });
    req.on('error', reject); req.setTimeout(3000, () => req.destroy(new Error('Request timeout'))); req.end(options.body);
  });
}
(async () => {
  const temporary = await fs.mkdtemp(path.join(os.tmpdir(), 'transit-runtime-access-'));
  let gateway, browser;
  const upstream = http.createServer((req, res) => res.end('gateway-api-ok'));
  const binary = process.env.TRANSIT_BINARY || path.join(root, 'target/debug/transit');
  const accessPath = path.join(temporary, 'transit/access.json');
  const configPath = path.join(temporary, 'routes.json');
  let scheme = 'http';
  const base = () => scheme + '://127.0.0.1:26021';
  async function stop() {
    if (gateway && gateway.exitCode === null) { const exit = once(gateway, 'exit'); gateway.kill('SIGTERM'); await exit; }
  }
  async function start(kubernetes = false) {
    const env = {...process.env, XDG_CONFIG_HOME: temporary, KUBECONFIG: path.join(temporary, 'no-kubeconfig')};
    for (const key of Object.keys(env)) if (key.startsWith('TRANSIT_')) delete env[key];
    const args = kubernetes
      ? ['--mode=kubernetes', '--access-config=' + accessPath, '--xds-address=http://127.0.0.1:' + upstream.address().port]
      : ['--mode=standalone', '--static-config=' + configPath];
    gateway = spawn(binary, args, {env});
    let output = '';
    gateway.stdout.on('data', value => output += value); gateway.stderr.on('data', value => output += value);
    for (let attempt = 0; attempt < 150; attempt++) {
      if (gateway.exitCode !== null) throw new Error('Gateway exited: ' + output);
      try { if ((await request(base() + '/healthz')).status === 200) return; } catch {}
      await delay(50);
    }
    throw new Error('Gateway did not start: ' + output);
  }
  async function session() {
    const result = await request(base() + '/admin/session', {method: 'POST', headers: {Origin: base()}});
    assert.equal(result.status, 200); return JSON.parse(result.body).token;
  }
  const admin = (token, options = {}) => request(base() + '/admin/access', {...options, headers: {Authorization: 'Bearer ' + token, 'Content-Type': 'application/json'}});
  try {
    // Fail without touching another running gateway if its default ports are in use.
    for (const port of [26080, 26443, 26021]) {
      const probe = http.createServer(); probe.listen(port, '0.0.0.0'); await once(probe, 'listening'); await new Promise(resolve => probe.close(resolve));
    }
    upstream.listen(0, '127.0.0.1'); await once(upstream, 'listening');
    await fs.writeFile(configPath, JSON.stringify({version: 'runtime-access', listeners: [], clusters: [], providers: [],
      backends: [{name: 'echo', type: 'http', endpoint: 'http://127.0.0.1:' + upstream.address().port}],
      routes: [{name: 'echo', protocol: 'http', matches: [], weighted_backends: [{name: 'echo', weight: 1}], policies: []}], policies: []}));
    await start();
    assert.equal((await request('http://127.0.0.1:26080/echo')).body, 'gateway-api-ok');
    let token = await session();
    let view = JSON.parse((await admin(token)).body);
    assert.equal(view.settings.active.port, 26080); assert(view.settings.writable);
    assert.match(view.management_address, /:26021$/);
    const cert = path.join(temporary, 'cert.pem'), key = path.join(temporary, 'key.pem');
    execFileSync('openssl', ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-keyout', key, '-out', cert, '-days', '1', '-subj', '/CN=localhost'], {stdio: 'ignore'});
    view.settings.config.tls = {enable: true, cert, key}; delete view.settings.config.port;
    let saved = await admin(token, {method: 'PUT', body: JSON.stringify({revision: view.settings.revision, config: view.settings.config})});
    assert.equal(saved.status, 200); assert.equal(JSON.parse(saved.body).settings.config.port, 26443);
    await stop(); scheme = 'https'; await start();
    assert.equal((await request('https://127.0.0.1:26443/echo')).body, 'gateway-api-ok');
    await assert.rejects(request('http://127.0.0.1:26080/echo'));
    token = await session(); view = JSON.parse((await admin(token)).body);
    assert.equal(view.settings.active.port, 26443); assert.equal(view.settings.active.tls, true);
    view.settings.config.tls.enable = false; delete view.settings.config.port;
    saved = await admin(token, {method: 'PUT', body: JSON.stringify({revision: view.settings.revision, config: view.settings.config})});
    assert.equal(saved.status, 200); assert.equal(JSON.parse(saved.body).settings.config.port, 26080);
    await stop(); scheme = 'http'; await start();
    assert.equal((await request('http://127.0.0.1:26080/echo')).body, 'gateway-api-ok');
    await stop();
    const deployment = JSON.parse(await fs.readFile(accessPath, 'utf8'));
    deployment.config['api-keys'] = ['runtime-access-client-key'];
    await fs.writeFile(accessPath, JSON.stringify(deployment));
    const original = await fs.readFile(accessPath, 'utf8');
    await start(true);
    token = await session(); view = JSON.parse((await admin(token)).body);
    assert.equal(view.runtime.mode, 'kubernetes'); assert.equal(view.settings.writable, false);
    assert.equal((await admin(token, {method: 'PUT', body: JSON.stringify({revision: view.settings.revision, config: view.settings.config})})).status, 409);
    browser = await chromium.launch({headless: true, executablePath: process.env.CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'});
    const context = await browser.newContext({permissions: ['clipboard-read', 'clipboard-write'], viewport: {width: 1100, height: 900}});
    const page = await context.newPage(); const errors = []; page.on('pageerror', error => errors.push(error.message));
    await page.goto(base() + '/#configuration'); await page.locator('#cfg-form').waitFor();
    assert.match(await page.locator('#cfg-mode').textContent(), /Kubernetes/);
    for (const selector of ['#cfg-port', '#cfg-host', '#cfg-save', '#cfg-add-key', '[data-action=edit]', '[data-action=delete]']) assert(await page.locator(selector).isDisabled(), selector);
    await page.locator('[data-action=copy]').click(); assert.equal(await page.evaluate(() => navigator.clipboard.readText()), 'runtime-access-client-key');
    await page.locator('#lang-toggle').click(); assert(await page.locator('#cfg-port').isDisabled());
    await page.evaluate(async () => { await configurationRefresh(false); await configurationSave({preventDefault() {}}); });
    assert(await page.locator('#cfg-save').isDisabled()); assert.equal(await fs.readFile(accessPath, 'utf8'), original);
    if (process.env.CONFIGURATION_ARTIFACTS) {
      await fs.mkdir(process.env.CONFIGURATION_ARTIFACTS, {recursive: true});
      await page.screenshot({path: path.join(process.env.CONFIGURATION_ARTIFACTS, 'configuration-kubernetes.png')});
    }
    assert.deepEqual(errors, []);
    console.log('PASS: HTTP 26080, HTTPS 26443, management 26021, restart persistence, actual TLS requests, Kubernetes read-only API and browser, copy access, language refresh, unchanged deployment file.');
  } finally {
    if (browser) await browser.close(); await stop();
    if (upstream.listening) await new Promise(resolve => upstream.close(resolve));
    await fs.rm(temporary, {recursive: true, force: true});
  }
})().catch(error => { console.error(error.stack); process.exitCode = 1; });
