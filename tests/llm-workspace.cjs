// Real gateway + browser + deterministic local upstream. No provider credentials.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const { existsSync } = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const { spawn } = require('node:child_process');
const { once } = require('node:events');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
let token;
const observed = [];
const quotaRequests = [], resetRequestIds = new Set(), resetCreditIds = [], redeemedCredits = new Set(), resetResults = new Map();
let resetCode;
let quotaFails = false, creditsFail = false, uncertainReset = false, resetCount = 0;
const upstream = http.createServer(async (req, res) => {
  if (req.url.startsWith('/backend-api/codex/models') || req.url.startsWith('/v1/models')) {
    assert.match(req.headers.authorization,/^Bearer (test-access|updated-access|test-unbound-)/);
    res.setHeader('Content-Type','application/json');
    res.end(JSON.stringify({models:['gpt-test','gpt-hidden','gpt-reserve-test'].map(slug=>({slug,display_name:slug}))})); return;
  }

  if (req.url.startsWith('/backend-api/wham/') || req.url.startsWith('/api/oauth/')) {
    quotaRequests.push({path:req.url, headers:req.headers});
    res.setHeader('Content-Type','application/json');
    const expires = days => new Date(Date.now() + days * 86400000).toISOString();
    if (req.url.endsWith('/consume')) {
      let raw = ''; for await (const chunk of req) raw += chunk;
      const {redeem_request_id:id, credit_id:creditId} = JSON.parse(raw);
      assert.ok(['credit-0','credit-1','credit-2'].includes(creditId));
      resetCreditIds.push(creditId);
      assert.match(id, /^[a-f0-9-]{36}$/i);
      let code = resetResults.get(id);
      if (code === 'reset') code = 'already_redeemed';
      if (!code) {
        code = resetCode || 'reset';
        resetRequestIds.add(id);
        if (code !== 'future_code') resetResults.set(id,code);
        if (code === 'reset') { redeemedCredits.add(creditId); resetCount++; }
      }
      if (uncertainReset) { uncertainReset = false; res.statusCode = 502; res.end('{}'); return; }
      res.end(JSON.stringify({code,windows_reset:code === 'reset' ? 2 : 0})); return;
    }
    if (req.url.endsWith('/rate-limit-reset-credits')) {
      if (creditsFail) { res.statusCode = 403; res.end('{}'); return; }
      res.end(JSON.stringify({ available_count:3-resetCount, credits:Array.from({length:3},(_,i) => ({id:'credit-'+i,status:'available',expires_at:expires(11+i*7)})).filter(c => !redeemedCredits.has(c.id)) })); return;
    }
    if (req.url.endsWith('/profile')) { res.end('{"account":{"has_claude_max":true}}'); return; }
    if (quotaFails) { res.statusCode = 503; res.end('{}'); return; }
    if (req.url === '/api/oauth/usage') { res.end(JSON.stringify({five_hour:{utilization:28,resets_at:expires(0.2)},seven_day:{utilization:98,resets_at:expires(5)}})); return; }
    const window = (used,seconds) => ({used_percent:used,limit_window_seconds:seconds,reset_at:Math.floor(Date.now()/1000 + seconds)});
    res.end(JSON.stringify({plan_type:'plus',rate_limit:{primary_window:window(resetCount ? 0 : 28,18000),secondary_window:window(resetCount ? 0 : 98,604800)},additional_rate_limits:[{limit_name:'gpt-reserve',rate_limit:{secondary_window:window(100,604800)}}],rate_limit_reset_credits:{available_count:3-resetCount,applicable_available_count:3-resetCount}})); return;
  }
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
  const temporary = await fs.mkdtemp(path.join(os.tmpdir(), 'transit-llm-workspace-'));
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
        { name: 'oauth', type: 'llm', provider: 'openai', endpoint: upstreamBase, account_type: 'subscription', models: ['gpt-test','gpt-hidden','gpt-reserve-test'] },
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
      const env = { ...process.env, XDG_CONFIG_HOME: storageRoot, TRANSIT_TEST_OAUTH_API_BASE: upstreamBase };
      delete env.TRANSIT_LLM_ADMIN_TOKEN; delete env.TRANSIT_LLM_ACCOUNTS_DIR;
      const bin = process.env.TRANSIT_BINARY || (existsSync(path.join(root, 'target/debug/transit')) ? path.join(root, 'target/debug/transit') : 'transit');
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
    if (process.env.UI_SOURCE_ASSETS === '1') {
      await page.route('**/assets/llm.*', async route => {
        const file = path.basename(new URL(route.request().url()).pathname);
        if (!['llm.js','llm.css'].includes(file)) return route.continue();
        await route.fulfill({contentType:file.endsWith('.js') ? 'text/javascript' : 'text/css',body:await fs.readFile(path.join(root,'ui',file))});
      });
    }

    const errors = []; page.on('pageerror', error => errors.push(error.message));
    await page.goto(base); await page.locator('.nav [data-tab="llm"]').click();
    await page.waitForFunction(() => state.llmData && !state.loading);
    const chooserPromise = page.waitForEvent('filechooser'); await page.locator('#llm-import').click();
    const claims = Buffer.from(JSON.stringify({ 'https://api.openai.com/auth':{chatgpt_plan_type:'plus',chatgpt_subscription_active_until:new Date(Date.now()+3*86400000).toISOString()} })).toString('base64url');
    await (await chooserPromise).setFiles({ name: 'browser-account.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify({ type: 'codex', email:'account@example.test', id_token:'test.'+claims+'.signature', access_token: 'test-access', account_id: 'account-id' })) });
    await page.locator('dialog [name=backend]').selectOption('oauth');
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('dialog').length === 0);
    assert.equal(await page.evaluate(() => state.llmData.management_mode), 'local');
    assert.equal(await page.locator('.llm-account-card').count(), 1);
    await page.waitForFunction(() => state.llmData.accounts[0]?.quota?.windows?.length === 3);
    assert.match(await page.locator('.llm-identity h3').innerText(), /^codex-account@example\.test-[a-f0-9]{7}$/);
    assert.equal(await page.locator('.llm-file-name,.llm-file-meta,.llm-health,.llm-binding').count(),0);
    assert.equal(await page.locator('.llm-quota-panel').evaluate(el => getComputedStyle(el).borderWidth),'0px');
    assert.equal(await page.locator('[data-action=reset]:not([data-credit-id])').count(),0);
    assert.ok(await page.locator('.llm-credit-expirations').evaluate(el => el.previousElementSibling.classList.contains('llm-quota-windows')));
    assert.ok((await page.locator('.llm-account-card').boundingBox()).width <= 360);
    assert.equal(await page.locator('[role=progressbar]').first().getAttribute('aria-valuenow'), '72');
    assert.equal(await page.locator('.llm-credit-expirations > div').count(),3);
    assert.match(await page.locator('.llm-plan-pills').innerText(), /Plus/);
    assert.equal(quotaRequests[0].headers.authorization,'Bearer test-access');
    assert.equal(quotaRequests[0].headers['chatgpt-account-id'],'account-id');
    assert.ok(!JSON.stringify(await page.evaluate(() => state.llmData)).includes('test-access'));
    const quotaPath = base + '/admin/llm/accounts/browser-account/quota';
    assert.equal((await fetch(quotaPath,{method:'POST'})).status,401);
    creditsFail = true;
    await page.locator('[data-action=quota]').click();
    await page.waitForFunction(() => state.llmData.accounts[0]?.quota?.credits_error);
    assert.equal(await page.locator('[role=progressbar]').count(),3);
    creditsFail = false; quotaFails = true;
    await page.locator('[data-action=quota]').click();
    await page.waitForFunction(() => state.llmData.accounts[0]?.quota_error);
    assert.equal(await page.locator('[role=progressbar]').first().getAttribute('aria-valuenow'),'72');
    quotaFails = false;
    await page.locator('[data-action=quota]').click();
    await page.waitForFunction(() => state.llmData.accounts[0]?.quota && !state.llmData.accounts[0].quota_error && !state.llmData.accounts[0].quota.credits_error);
    uncertainReset = true;
    await page.locator('[data-action=reset][data-credit-id=credit-1]').click();
    await page.locator('dialog [type=submit]').click();
    await page.locator('dialog [role=alert]:not(:empty)').waitFor();
    await stopGateway(); await startGateway();
    await page.evaluate(() => sessionStorage.clear());
    await page.reload(); await page.locator('.nav [data-tab=llm]').click();
    await page.waitForFunction(() => state.llmData.accounts[0]?.pending_reset_credit_id === 'credit-1');
    await page.locator('[data-action=reset][data-credit-id=credit-1]').click();
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('dialog').length === 0);
    assert.equal(resetCount,1); assert.equal(resetRequestIds.size,1);
    assert.deepEqual(resetCreditIds,['credit-1','credit-1']);
    assert.equal(await page.locator('[role=progressbar]').first().getAttribute('aria-valuenow'),'100');
    assert.equal(await page.locator('.llm-credit-expirations > div').count(),2);
    if (await page.evaluate(() => state.lang !== 'zh')) await page.locator('#lang-toggle').click();
    if (await page.evaluate(() => state.theme !== 'dark')) await page.locator('#theme-toggle').click();
    for (const width of [1440,390]) {
      await page.setViewportSize({width,height:1000});
      await page.waitForFunction(() => document.documentElement.dataset.theme === 'dark');
      const bounds = await page.locator('.llm-account-card').boundingBox();
      assert.ok(bounds.width <= 360 && bounds.height <= 430,JSON.stringify(bounds));
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
      if (process.env.UI_SCREENSHOT_DIR) await page.locator('.llm-account-card').screenshot({path:path.join(process.env.UI_SCREENSHOT_DIR,'account-card-compact-'+width+'.png')});
    }
    await page.setViewportSize({width:1440,height:1000});
    if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({path:path.join(process.env.UI_SCREENSHOT_DIR,'account-card-verified.png'),fullPage:true});

    for (const code of ['nothing_to_reset','no_credit']) {
      resetCode = code;
      const result = await page.evaluate(async () => (await llmAdminFetch('browser-account/reset-quota',{method:'POST',body:JSON.stringify({redeem_request_id:crypto.randomUUID(),credit_id:'credit-0'})})).json());
      assert.equal(result.code,code); assert.equal(result.reset_applied,false); assert.equal(result.credit_id,'credit-0');
    }
    resetCode = 'future_code';
    const pendingId = crypto.randomUUID();
    const sendReset = (requestId,creditId) => page.evaluate(async ({requestId,creditId}) => {
      try { return await (await llmAdminFetch('browser-account/reset-quota',{method:'POST',body:JSON.stringify({redeem_request_id:requestId,credit_id:creditId})})).json(); }
      catch (e) { return {error:e.message}; }
    },{requestId,creditId});
    assert.match((await sendReset(pendingId,'credit-0')).error,/unrecognized reset result/);
    assert.match((await sendReset(crypto.randomUUID(),'credit-2')).error,/Reset conflict/);
    assert.match((await sendReset(pendingId,'credit-2')).error,/Reset conflict/);
    resetCode = 'nothing_to_reset';
    assert.equal((await sendReset(pendingId,'credit-0')).code,'nothing_to_reset');
    resetCode = undefined;
    assert.equal(resetCount,1);

    await page.locator('[data-action=models]').click();
    await page.locator('dialog [data-exclude-model="gpt-hidden"]').check();
    await page.locator('dialog [name=excluded_patterns]').fill('GpT-ReSeRvE*');
    await page.locator('dialog [data-add-alias]').click();
    await page.locator('dialog [name=model]').fill('gpt-test');
    await page.locator('dialog [name=alias]').fill('friendly');
    assert.equal(await page.locator('dialog [name=keep_original]').isChecked(),false);
    await page.locator('dialog details summary').click();
    await page.locator('dialog [name=effort]').selectOption('high');
    for (const width of [1440,390]) {
      await page.setViewportSize({width,height:1000});
      assert.ok(await page.locator('dialog').evaluate(el=>el.scrollWidth<=el.clientWidth));
      if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({path:path.join(process.env.UI_SCREENSHOT_DIR,'model-management-'+width+'.png'),fullPage:true});
    }
    await page.setViewportSize({width:1440,height:1000});

    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('dialog').length === 0);
    const publicModels = async () => { const response = await fetch(proxy+'/v1/models'); assert.equal(response.status,200); return (await response.json()).data.map(m=>m.id).sort(); };
    assert.deepEqual(await publicModels(),['friendly']);
    const beforeExcluded = observed.length;
    for (const model of ['gpt-test','gpt-hidden','gpt-reserve-test']) {
      const denied=await fetch(proxy+'/v1/chat/completions',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({model,messages:[{role:'user',content:'blocked'}]})});
      assert.ok(!denied.ok); await denied.text();
    }
    assert.equal(observed.length,beforeExcluded);
    const response = await fetch(proxy + '/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ model: 'friendly', messages: [{ role: 'user', content: 'hi' }] }) });
    assert.equal(response.status, 200); await response.text();
    assert.equal(observed.at(-1).body.model, 'gpt-test');
    assert.equal(observed.at(-1).body.reasoning.effort, 'high');
    await page.locator('[data-action=models]').click();
    await page.locator('dialog [name=keep_original]').check();
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(()=>!document.querySelector('dialog'));
    assert.deepEqual(await publicModels(),['friendly','gpt-test']);
    await page.locator('[data-action=models]').click();
    await page.locator('dialog [data-exclude-model="gpt-test"]').check();
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(()=>!document.querySelector('dialog'));
    assert.deepEqual(await publicModels(),[]);
    const beforeDisabledAlias=observed.length;
    const deniedAlias=await fetch(proxy+'/v1/chat/completions',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({model:'friendly',messages:[{role:'user',content:'blocked'}]})});
    assert.ok(!deniedAlias.ok); await deniedAlias.text(); assert.equal(observed.length,beforeDisabledAlias);
    await page.locator('[data-action=models]').click();
    await page.locator('dialog [data-exclude-model="gpt-test"]').uncheck();
    await page.locator('dialog [name=keep_original]').uncheck();
    await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(()=>!document.querySelector('dialog'));
    assert.deepEqual(await publicModels(),['friendly']);

    await page.locator('.nav [data-tab=overview]').click(); await page.locator('.nav [data-tab=llm]').click();
    await page.waitForFunction(() => state.llmData.accounts[0]?.health?.success === 1);
    await page.locator('[data-action=toggle]').click();
    await page.waitForFunction(() => state.llmData.accounts[0]?.disabled === true);
    const beforeDisabled = observed.length;
    const blocked = await fetch(proxy + '/v1/chat/completions',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({model:'friendly',messages:[{role:'user',content:'hi'}]})});
    assert.ok(!blocked.ok); await blocked.text(); assert.equal(observed.length,beforeDisabled);
    await stopGateway(); await startGateway(); await page.reload(); await page.locator('.nav [data-tab=llm]').click();
    await page.waitForFunction(() => state.llmData?.accounts[0]?.disabled === true);
    await page.locator('[data-action=toggle]').click();
    await page.waitForFunction(() => state.llmData.accounts[0]?.disabled === false);
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
    assert.equal(persisted.models.find(m=>m.model==='gpt-test').alias, 'friendly');
    assert.equal(persisted.models.find(m=>m.model==='gpt-test').keep_original,false);
    assert.ok(persisted.models.some(m=>m.model==='GpT-ReSeRvE*' && m.disabled));
    for (const [model, stream] of [['gpt-5.6-sol', false], ['local-test', true]]) {
      const response = await fetch(proxy + '/v1/chat/completions', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ model, stream, messages: [{ role: 'user', content: 'hi' }] }) });
      assert.equal(response.status, 200); await response.text();
    }
    await page.locator('.nav [data-tab=overview]').click(); await page.waitForFunction(() => !state.loading);
    await page.locator('.nav [data-tab=llm]').click(); await page.waitForFunction(() => !state.loading);
    const dashboard = await page.evaluate(async () => (await managementFetch('/debug/llm')).json());
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
      assert.equal(await page.evaluate(provider => state.llmData.accounts.find(a => a.provider === provider).backend,provider),'');
      if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({ path: path.join(process.env.UI_SCREENSHOT_DIR, 'empty-' + provider + '-account.png'), fullPage: true });
    }
    await stopGateway(); await startGateway();
    await page.reload(); await page.locator('.nav [data-tab=llm]').click();
    await page.waitForFunction(() => state.llmData && !state.loading);
    assert.equal(await page.evaluate(() => state.llmData.accounts.length), 2);
    assert.equal(await page.evaluate(() => state.llmData.accounts.every(a => a.backend === '')), true);
    assert.equal(await page.locator('.llm-account-card').count(), 1);
    assert.deepEqual(errors, []);
    await page.locator('#llm-family-trigger').click(); await page.locator('.llm-family-option[data-value=chatgpt]').click();
    await page.locator('[data-select]').check();
    await page.locator('[data-bulk=disable]').click();
    await page.waitForFunction(() => state.llmData.accounts.find(a => a.provider === 'codex')?.disabled === true);
    await page.locator('[data-bulk=enable]').click();
    await page.waitForFunction(() => state.llmData.accounts.find(a => a.provider === 'codex')?.disabled === false);
    await page.locator('[data-bulk=delete]').click(); await page.locator('dialog [type=submit]').click();
    await page.waitForFunction(() => state.llmData.accounts.length === 1);
    await stopGateway(); await startGateway(); await page.reload(); await page.locator('.nav [data-tab=llm]').click();
    await page.waitForFunction(() => state.llmData?.accounts?.length === 1);
    assert.equal(await page.evaluate(() => state.llmData.accounts[0].provider),'claude');
    console.log('PASS account card uses real management/provider HTTP: quota, reset retry idempotency, disable/restart, selection, deletion and credential redaction');
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
