// Real gateway + browser + deterministic local upstream. No provider credentials.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const { spawn } = require('node:child_process');
const { once } = require('node:events');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
let token;
const observed = [];
const upstream = http.createServer(async (req, res) => {
  let raw = '';
  for await (const chunk of req) raw += chunk;
  const body = JSON.parse(raw);
  observed.push({ headers: req.headers, body, path: req.url });
  if (req.url === '/responses') {
    res.setHeader('Content-Type', 'text/event-stream');
    res.end('data: ' + JSON.stringify({ type: 'response.completed', response: { id: 'resp-test', model: body.model, output: [{ type: 'message', content: [{ type: 'output_text', text: 'hello' }] }], usage: { input_tokens: 10, output_tokens: 5 } } }) + '\n\n');
  } else if (body.stream) {
    res.setHeader('Content-Type', 'text/event-stream');
    res.flushHeaders();
    setTimeout(() => {
      res.write('data: {"choices":[{"delta":{"content":"hello"}}]}\n\n');
      setTimeout(() => res.end('data: {"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":5}}\n\ndata: [DONE]\n\n'), 30);
    }, 25);
  } else {
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ id: 'chat-test', model: body.model, choices: [{ message: { content: 'hello' } }], usage: { prompt_tokens: 10, completion_tokens: 5, prompt_tokens_details: { cached_tokens: 2 }, completion_tokens_details: { reasoning_tokens: 3 } } }));
  }
});

async function unusedPort() {
  const server = http.createServer();
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const port = server.address().port; await new Promise(resolve => server.close(resolve)); return port;
}

(async () => {
  const temporary = await fs.mkdtemp(path.join(os.tmpdir(), 'xgate-llm-workspace-'));
  let gateway, browser;
  let processOutput = '';
  async function stopGateway() {
    if (!gateway || gateway.exitCode !== null) return;
    const exited = once(gateway, 'exit'); gateway.kill('SIGTERM');
    const timer = setTimeout(() => gateway.kill('SIGKILL'), 5000);
    await exited; clearTimeout(timer);
  }
  try {
    upstream.listen(0, '127.0.0.1'); await once(upstream, 'listening');
    const upstreamBase = 'http://127.0.0.1:' + upstream.address().port;
    const uiPort = await unusedPort(), proxyPort = await unusedPort();
    const base = 'http://127.0.0.1:' + uiPort, proxy = 'http://127.0.0.1:' + proxyPort;
    const config = {
      version: 'llm-workspace-test',
      listeners: [], clusters: [], policies: [],
      providers: [{ name: 'openai', kind: 'open-ai', base_url: upstreamBase + '/v1' }],
      backends: [
        { name: 'oauth', type: 'llm', provider: 'openai', endpoint: upstreamBase, account_type: 'subscription', models: ['gpt-test'] },
        { name: 'api', type: 'llm', provider: 'openai', account_type: 'api-key', models: ['gpt-5.6-sol'] },
        { name: 'local', type: 'llm', provider: 'openai', account_type: 'self-hosted', models: ['local-test'] }
      ],
      routes: [
        { name: 'api', protocol: 'llm', matches: [{ path: { type: 'prefix', value: '/v1/' }, model: 'gpt-5.6-sol' }], weighted_backends: [{ name: 'api', weight: 1 }] },
        { name: 'local', protocol: 'llm', matches: [{ path: { type: 'prefix', value: '/v1/' }, model: 'local-test' }], weighted_backends: [{ name: 'local', weight: 1 }] },
        { name: 'oauth', protocol: 'llm', matches: [{ path: { type: 'prefix', value: '/v1/' } }], weighted_backends: [{ name: 'oauth', weight: 1 }] }
      ]
    };
    const configPath = path.join(temporary, 'config.json');
    let storageRoot = path.join(temporary, 'config');
    await fs.writeFile(configPath, JSON.stringify(config));
    async function startGateway() {
      const env = { ...process.env, XDG_CONFIG_HOME: storageRoot };
      delete env.XGATE_LLM_ADMIN_TOKEN; delete env.XGATE_LLM_ACCOUNTS_DIR;
      delete env.DXGATE_LLM_ADMIN_TOKEN; delete env.DXGATE_LLM_ACCOUNTS_DIR;
      const bin = process.env.XGATE_BINARY || process.env.DXGATE_BINARY || (fs.existsSync(path.join(root, 'target/debug/xgate')) ? path.join(root, 'target/debug/xgate') : path.join(root, 'target/debug/dxgate'));
      gateway = spawn(bin, ['--xds-enabled=false', '--http-addr=127.0.0.1:' + proxyPort, '--ui-addr=127.0.0.1:' + uiPort, '--static-config=' + configPath], { env });
      gateway.stdout.on('data', data => { processOutput = (processOutput + data).slice(-4000); });
      gateway.stderr.on('data', data => { processOutput = (processOutput + data).slice(-4000); });
      for (let i = 0; i < 100; i++) {
        if (gateway.exitCode !== null) throw new Error('Gateway exited: ' + processOutput);
        try { if ((await fetch(base + '/healthz')).ok) return; } catch {}
        await new Promise(resolve => setTimeout(resolve, 50));
      }
      throw new Error('Gateway did not become ready: ' + processOutput);
    }
    await startGateway();
    browser = await chromium.launch({ headless: true, executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    const errors = []; page.on('pageerror', error => errors.push(error.message));
    await page.goto(base); await page.locator('.nav [data-tab="llm"]').click();
    await page.waitForFunction(() => state.llmData && !state.loading);
    const chooserPromise = page.waitForEvent('filechooser'); await page.locator('#llm-import').click();
    await (await chooserPromise).setFiles({ name: 'browser-account.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify({ type: 'codex', access_token: 'test-access', account_id: 'account-id' })) });
    await page.locator('dialog [name=backend]').selectOption('oauth');
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('dialog').length === 0);
    assert.equal(await page.evaluate(() => state.llmData.management_mode), 'local');
    assert.equal(await page.locator('.llm-account-card').count(), 1);
    await page.locator('[data-action=models]').click();
    await page.locator('dialog [name=alias]').fill('friendly');
    await page.locator('dialog [name=effort]').selectOption('high');
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('dialog').length === 0);
    const response = await fetch(proxy + '/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ model: 'friendly', messages: [{ role: 'user', content: 'hi' }] }) });
    assert.equal(response.status, 200); await response.text();
    assert.equal(observed.at(-1).body.model, 'gpt-test');
    assert.equal(observed.at(-1).body.reasoning.effort, 'high');
    const downloadPromise = page.waitForEvent('download'); await page.locator('[data-action=download]').click();
    const download = await downloadPromise, chunks = [];
    for await (const chunk of await download.createReadStream()) chunks.push(chunk);
    assert.equal(JSON.parse(Buffer.concat(chunks).toString()).access_token, 'test-access');
    await page.locator('[data-action=edit]').click();
    const edited = JSON.parse(await page.locator('dialog textarea').inputValue()); edited.access_token = 'updated-access';
    await page.locator('dialog textarea').fill(JSON.stringify(edited)); await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('dialog').length === 0);
    await stopGateway(); await startGateway();
    const session = await fetch(base + '/admin/llm/session', { method: 'POST', headers: { Origin: base, 'Content-Type': 'application/json' }, body: '{}' });
    assert.equal(session.status, 200); token = (await session.json()).token;
    const persisted = await (await fetch(base + '/admin/llm/accounts/browser-account', { headers: { Authorization: 'Bearer ' + token } })).json();
    assert.equal(persisted.document.access_token, 'updated-access');
    assert.equal(persisted.models[0].alias, 'friendly');
    for (const [model, stream] of [['gpt-5.6-sol', false], ['local-test', true]]) {
      const response = await fetch(proxy + '/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ model, stream, messages: [{ role: 'user', content: 'hi' }] }) });
      assert.equal(response.status, 200); await response.text();
    }
    await page.locator('.nav [data-tab=overview]').click(); await page.waitForFunction(() => !state.loading);
    await page.locator('.nav [data-tab=llm]').click(); await page.waitForFunction(() => !state.loading);
    const dashboard = await (await fetch(base + '/debug/llm')).json();
    const api = dashboard.backends.find(b => b.name === 'api'), local = dashboard.backends.find(b => b.name === 'local');
    assert.equal(api.input_tokens, 10); assert.equal(api.output_tokens, 5); assert.equal(api.cached_input_tokens, 2); assert.equal(api.reasoning_tokens, 3);
    assert.ok(api.estimated_usd > 0); assert.ok(local.ttft_ms >= 20); assert.ok(local.tokens_per_second > 0); assert.ok(local.requests_per_second > 0); assert.equal(local.context, 15);
    for (const mode of ['subscription', 'api', 'local']) {
      await page.locator('[data-mode=' + mode + ']').click();
      assert.equal(await page.locator('.llm-account-card').count(), 1);
      if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({ path: path.join(process.env.UI_SCREENSHOT_DIR, 'llm-' + mode + '.png'), fullPage: true });
    }
    await page.locator('[data-mode=subscription]').click();
    if (await page.evaluate(() => state.lang !== 'zh')) await page.locator('#lang-toggle').click();
    const loginButton = await page.locator('#llm-login').boundingBox();
    const uploadButton = await page.locator('#llm-import').boundingBox();
    assert.ok(loginButton.x < uploadButton.x, 'OAuth login is left of upload');
    await page.locator('#llm-login').click();
    await page.locator('.llm-login-link:not([hidden])').waitFor();
    assert.equal(await page.locator('[name=token], [name=management_token]').count(), 0);
    const authorization = new URL(await page.locator('[data-open]').getAttribute('href'));
    assert.equal(authorization.origin, 'https://auth.openai.com');
    assert.equal(authorization.searchParams.get('code_challenge_method'), 'S256');
    assert.equal(authorization.searchParams.get('redirect_uri'), 'http://localhost:1455/auth/callback');
    assert.equal(await page.locator('[data-open]').getAttribute('rel'), 'noopener noreferrer');
    for (const width of [1440, 390]) {
      await page.setViewportSize({ width, height: 1000 });
      assert.equal(await page.locator('.llm-login-dialog').evaluate(el => el.scrollWidth <= el.clientWidth + 1), true);
      if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({ path: path.join(process.env.UI_SCREENSHOT_DIR, 'oauth-login-' + width + '.png'), fullPage: true });
    }
    await page.setViewportSize({ width: 1440, height: 1000 });
    await page.evaluate(() => document.documentElement.dataset.theme = 'dark');
    const panelColors = await page.locator('.llm-login-dialog').evaluate(el => ({ background: getComputedStyle(el).backgroundColor, color: getComputedStyle(el).color }));
    assert.notEqual(panelColors.background, 'rgb(255, 255, 255)');
    assert.notEqual(panelColors.background, panelColors.color);
    if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({ path: path.join(process.env.UI_SCREENSHOT_DIR, 'oauth-login-dark.png'), fullPage: true });
    await page.locator('[name=callback]').fill('http://localhost:1455/auth/callback?code=test&state=wrong');
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelector('dialog [role=alert]').textContent.includes('state mismatch'));
    assert.equal(await page.locator('dialog').count(), 1);
    // Browser success rendering is deterministic; provider token exchange is covered by Rust HTTP tests.
    await page.route('**/admin/llm/oauth/*/callback', route => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ id: 'browser-login-test' }) }));
    await page.locator('[name=callback]').fill('http://localhost:1455/auth/callback?code=test&state=' + authorization.searchParams.get('state'));
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelector('dialog [role=status]').textContent.includes('browser-login-test'));
    assert.equal(await page.locator('[name=callback]').inputValue(), '');
    assert.equal(await page.locator('[data-open]').getAttribute('href'), null);
    await page.locator('dialog header [data-close]').click();
    // Polling success must update the same panel without a manual callback submission.
    await page.locator('#llm-login').click();
    await page.locator('.llm-login-link:not([hidden])').waitFor();
    await page.route(/\/admin\/llm\/oauth\/[^/]+$/, route => {
      if (route.request().method() !== 'GET') return route.continue();
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ status: 'success', account: { id: 'polled-login-test' } }) });
    });
    await page.waitForFunction(() => document.querySelector('dialog [role=status]').textContent.includes('polled-login-test'));
    await page.unroute(/\/admin\/llm\/oauth\/[^/]+$/);
    await page.locator('dialog header [data-close]').click();
    // Real loopback callback denial must propagate through the status endpoint to the browser.
    await page.locator('#llm-login').click();
    await page.locator('.llm-login-link:not([hidden])').waitFor();
    const nextAuthorization = new URL(await page.locator('[data-open]').getAttribute('href'));
    if (await page.locator('dialog').getAttribute('data-callback-mode') === 'automatic') {
      const deniedCallback = 'http://127.0.0.1:1455/auth/callback?error=access_denied&state=' + nextAuthorization.searchParams.get('state');
      const denied = await fetch(deniedCallback);
      assert.equal(denied.status, 400);
      await page.waitForFunction(() => document.querySelector('dialog [role=alert]').textContent.includes('Authorization was denied'));
    } else {
      assert.match(await page.locator('dialog [role=status]').innerText(), /回调端口不可用|Callback port is unavailable/);
    }
    await page.locator('dialog header [data-close]').click();
    assert.deepEqual(errors, []);
    await stopGateway();
    storageRoot = path.join(temporary, 'empty-account-storage');
    await fs.writeFile(configPath, JSON.stringify({ version: 'empty-oauth-test', listeners: [], clusters: [], providers: [], backends: [], routes: [], policies: [] }));
    await startGateway();
    await page.reload(); await page.locator('.nav [data-tab=llm]').click();
    await page.waitForFunction(() => state.llmData && !state.loading);
    assert.equal(await page.evaluate(() => state.llmData.backends.length), 0);
    assert.equal(await page.locator('.llm-account-card').count(), 0);
    for (const [family, provider, domain] of [['chatgpt', 'codex', 'auth.openai.com'], ['anthropic', 'claude', 'claude.ai']]) {
      await page.locator('#llm-family-trigger').click();
      await page.locator('.llm-family-option[data-value=' + family + ']').click();
      await page.locator('#llm-login').click();
      await page.locator('.llm-login-link:not([hidden])').waitFor();
      assert.equal(new URL(await page.locator('[data-open]').getAttribute('href')).hostname, domain);
      assert.equal(await page.locator('dialog [name=backend]').inputValue(), '');
      assert.equal(await page.locator('[name=token],[name=management_token]').count(), 0);
      if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({ path: path.join(process.env.UI_SCREENSHOT_DIR, 'empty-' + provider + '-login.png'), fullPage: true });
      await page.locator('dialog header [data-close]').click();
      const choose = page.waitForEvent('filechooser'); await page.locator('#llm-import').click();
      await (await choose).setFiles({ name: provider + '.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify({ type: provider, access_token: 'test-unbound-' + provider })) });
      assert.equal(await page.locator('dialog [name=backend]').inputValue(), '');
      await page.locator('dialog [type=submit]').click();
      await page.waitForFunction(() => document.querySelectorAll('dialog').length === 0);
      assert.equal(await page.locator('.llm-account-card').count(), 1);
      assert.match(await page.locator('.llm-account-card .llm-stat-row').first().innerText(), /Not bound|未绑定/);
      if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({ path: path.join(process.env.UI_SCREENSHOT_DIR, 'empty-' + provider + '-account.png'), fullPage: true });
    }
    await stopGateway(); await startGateway();
    await page.reload(); await page.locator('.nav [data-tab=llm]').click();
    await page.waitForFunction(() => state.llmData && !state.loading);
    assert.equal(await page.evaluate(() => state.llmData.accounts.length), 2);
    assert.equal(await page.evaluate(() => state.llmData.accounts.every(a => a.backend === '')), true);
    assert.equal(await page.locator('.llm-account-card').count(), 1);
    assert.deepEqual(errors, []);
    console.log('PASS real browser upload/edit/download/model rules, proxy protocol, restart persistence, API and local KPIs');
    console.log('PASS zero-configuration local session, automatic authorization link, callback availability, responsive panel and simulated manual/polled success rendering');
    console.log('PASS completely empty configuration: both OAuth providers generate links, import and display unbound accounts, persist across restart');
  } finally {
    if (browser) await browser.close();
    await stopGateway();
    await new Promise(resolve => upstream.close(resolve));
    await fs.rm(temporary, { recursive: true, force: true });
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
