// Run with Node.js and Playwright available on NODE_PATH. No npm install or gateway needed.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const demo = JSON.parse(fs.readFileSync(path.join(__dirname, 'ui-fake.json'), 'utf8'));
const empty = { listeners: [], clusters: [], backends: [], providers: [], routes: [], policies: [] };
let config = empty;
let costFails = false;
let llmFails = false;
let managementEnabled = true;
let traces = [];
let securityEvents = [];
const requests = [];
const server = http.createServer((req, res) => {
  const url = new URL(req.url, 'http://localhost');
  requests.push(url.pathname);
  if (url.pathname === '/' || url.pathname === '/ui') {
    res.setHeader('Content-Type', 'text/html');
    res.end(fs.readFileSync(path.join(root, 'ui/ui.html')));
  } else if (url.pathname.startsWith('/assets/')) {
    if (['/assets/llm.js', '/assets/llm.css', '/assets/configuration.js', '/assets/configuration.css'].includes(url.pathname)) {
      res.setHeader('Content-Type', url.pathname.endsWith('.js') ? 'text/javascript' : 'text/css');
      res.end(fs.readFileSync(path.join(root, 'ui', path.basename(url.pathname))));
      return;
    }
    const file = path.join(root, 'logo', path.basename(url.pathname));
    res.setHeader('Content-Type', 'image/svg+xml');
    res.end(fs.readFileSync(file));
  } else {
    res.setHeader('Content-Type', 'application/json');
    if (url.pathname === '/debug/config') res.end(JSON.stringify(config));
    else if (url.pathname === '/debug/llm') {
      res.statusCode = llmFails ? 503 : 200;
      const backends = config.backends.filter(b => b.type === 'llm').map(b => ({
        ...b, family: b.provider === 'anthropic' ? 'anthropic' : 'chatgpt',
        mode: b.account_type === 'subscription' ? 'subscription' : b.account_type === 'self-hosted' ? 'local' : 'api',
        quota: b.quota_state
      }));
      const accounts = backends.filter(b => b.mode === 'subscription').map(b => ({ id: b.name, backend: b.name, provider: b.family === 'anthropic' ? 'claude' : 'codex', revision: 1, models: [], can_refresh: true, quota: b.quota_state ? { observed_at:new Date().toISOString(), windows:b.quota_state.windows.map(w => ({ name:w.name,window_seconds:w.window === '7d' ? 604800 : w.window === '30d' ? 2592000 : 18000,used_percent:w.used_percent,reset_at:w.reset_at })) } : null }));
      res.end(JSON.stringify({ backends, accounts, management_enabled: managementEnabled }));
    }
    else if (url.pathname === '/debug/cost') {
      res.statusCode = costFails ? 503 : 200;
      res.end(JSON.stringify({ rate_card: [], usage: [], events: [], api_usd: '$0', chatgpt_credits: '0' }));
    } else if (url.pathname === '/debug/observability') res.end(JSON.stringify({ telemetry: {}, kpis: {}, traces }));
    else if (url.pathname === '/debug/security/events') res.end(JSON.stringify(securityEvents));
    else if (url.pathname === '/debug/security/identities') res.end('[]');
    else if (url.pathname === '/debug/security/posture') res.end('{"policy_default":"deny"}');
    else if (url.pathname === '/debug/services') { res.statusCode = 503; res.end('{}'); }
    else { res.statusCode = 404; res.end('{}'); }
  }
});

(async () => {
  let browser;
  try {
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
    browser = await chromium.launch({ headless: true, ...(fs.existsSync(chromePath) ? { executablePath: chromePath } : {}) });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    await page.addInitScript(() => {
      Object.defineProperty(navigator, 'clipboard', { value: { writeText: async value => { window.copiedText = value; } } });
    });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('dialog', async dialog => { errors.push('Unexpected native dialog: ' + dialog.message()); await dialog.dismiss(); });
    const base = `http://127.0.0.1:${server.address().port}`;
    const idle = () => page.waitForFunction(() => state.started && !state.loading);
    const go = async tab => { await page.locator(`.nav button[data-tab="${tab}"]`).click(); await idle(); };
    await page.goto(base);
    await idle();
    for (const tab of ['llm', 'mcp', 'a2a', 'services', 'observability', 'security', 'cost-control', 'overview']) await go(tab);
    assert.equal(await page.locator('#metric-llm').innerText(), '0');
    await go('llm');
    assert.equal(await page.locator('.llm-account-card').count(), 0);
    assert.match(await page.locator('#llm-accounts-grid').innerText(), /No accounts|No available information|暂无可用信息/);
    assert.deepEqual(errors, []);
    console.log('PASS empty configuration across all 8 pages');

    config = structuredClone(demo);
    config.backends.push({ name: "weekly-only's <account>", type: 'llm', provider: 'custom-provider', account_type: 'subscription', models: ['custom-model'], quota_state: { windows: [{ name: 'weekly', window: '7d', used_percent: 0 }] } });
    await go('overview'); await go('llm');
    await idle();
    assert.equal(await page.locator('.llm-account-card').count(), 2);
    await page.locator('#search-input').fill('weekly-only');
    assert.equal(await page.locator('.llm-account-card:visible').count(), 1);
    assert.equal(await page.locator('.reset-cards-box').count(), 0);
    assert.equal(await page.locator('.llm-account-card:visible .quota-window-item').count(), 1);
    assert.match(await page.locator('.llm-account-card:visible').innerText(), /100%/);
    assert.equal(await page.locator('.llm-account-card:visible [role=progressbar]').getAttribute('aria-valuenow'), '100');
    assert.equal(await page.locator('.llm-account-card:visible [data-action]').count(), 6);
    config.backends.find(b => b.name.startsWith('weekly-only')).quota_state.windows = [{ name:'',window:'30d',used_percent:99 }];
    await go('overview'); await go('llm'); await idle();
    assert.match(await page.locator('.llm-account-card:visible').innerText(), /Monthly limit|月限额/);
    assert.equal(await page.locator('.llm-account-card:visible [role=progressbar]').getAttribute('aria-valuenow'),'1');
    await page.locator('#search-input').fill('does-not-exist');
    assert.equal(await page.locator('.llm-account-card:visible').count(), 0);
    assert.equal(await page.locator('#page-search-empty').isVisible(), true);
    await page.locator('#search-input').fill('');
    console.log('PASS provider filters, reported-only quotas, safe names and truthful actions');

    managementEnabled = false;
    await go('overview'); await go('llm'); await idle();
    await page.locator('#llm-login').click();
    assert.equal(await page.locator('dialog').count(), 1);
    assert.equal(await page.locator('dialog h2').innerText(), 'Codex OAuth');
    assert.equal(await page.locator('[name=token]').count(), 0);
    assert.equal(await page.locator('[name=management_token]').count(), 0);
    assert.equal(await page.locator('[data-start]').isDisabled(), true);
    assert.match(await page.locator('.llm-login-setup').innerText(), /TRANSIT_LLM_ACCOUNTS_DIR/);
    assert.match(await page.locator('.llm-login-setup').innerText(), /TRANSIT_LLM_ADMIN_TOKEN/);
    if (process.env.UI_SCREENSHOT_DIR) await page.screenshot({ path: path.join(process.env.UI_SCREENSHOT_DIR, 'oauth-not-configured.png'), fullPage: true });
    await page.locator('dialog header [data-close]').click();
    managementEnabled = true;
    await go('overview'); await go('llm'); await idle();
    await page.locator('#llm-login').click();
    assert.equal(await page.locator('dialog h2').innerText(), 'Codex OAuth');
    assert.equal(await page.locator('[name=management_token]').isVisible(), true);
    await page.locator('[data-start]').click();
    assert.match(await page.locator('dialog [role=alert]').innerText(), /at least 24/);
    assert.equal(await page.locator('dialog').count(), 1);
    await page.route('**/admin/llm/oauth/start', route => {
      if (route.request().headers().authorization !== 'Bearer test-management-token-long-enough') return route.fulfill({ status: 401, contentType: 'application/json', body: '{"error":"LLM management authentication required"}' });
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ id: 'ui-only-session', authorization_url: 'https://auth.openai.com/oauth/authorize?state=ui-test', redirect_uri: 'http://localhost:1455/auth/callback', expires_in: 600 }) });
    });
    await page.locator('[name=management_token]').fill('incorrect-management-token-long-enough');
    await page.locator('[data-start]').click();
    await page.waitForFunction(() => document.querySelector('dialog [role=alert]').textContent.includes('authentication required'));
    assert.equal(await page.locator('[name=management_token]').isVisible(), true);
    assert.equal(await page.locator('[name=management_token]').inputValue(), '');
    await page.locator('[name=management_token]').fill('test-management-token-long-enough');
    await page.locator('[data-start]').click();
    await page.locator('.llm-login-link:not([hidden])').waitFor();
    assert.equal(await page.locator('[data-open]').getAttribute('href'), 'https://auth.openai.com/oauth/authorize?state=ui-test');
    assert.equal(await page.locator('[name=management_token]').inputValue(), '');
    assert.equal(await page.locator('dialog').count(), 1);
    await page.locator('dialog header [data-close]').click();
    await page.unroute('**/admin/llm/oauth/start');
    console.log('PASS OAuth panel shows missing setup, accepts management token inline and displays authorization link without nested dialogs');

    await go('mcp');
    assert.equal(await page.locator('#tbody-mcp-target-servers tr[data-mcp-server]').count(), 1);
    assert.equal(await page.locator('#tbody-mcp-capabilities tr').count(), 2);
    assert.equal(await page.locator('#mcp-invocation-detail').isVisible(), false);
    await page.locator('#tbody-mcp-target-servers tr[data-mcp-server]').first().click();
    assert.equal(await page.locator('#tbody-mcp-invocations tr').count(), 1);
    await go('a2a');
    await page.locator('#tbody-a2a-registry tr[data-a2a-agent]').first().click();
    await page.locator('[data-task-tab="working"]').click();
    assert.match(await page.locator('#tbody-a2a-tasks').innerText(), /No matching/);
    assert.equal(await page.locator('#a2a-task-drawer').isVisible(), false);
    assert.equal(await page.locator('#tbody-a2a-registry tr[data-a2a-agent]').count(), 1);
    assert.equal(await page.locator('#a2a-task-drawer').isVisible(), false);
    console.log('PASS MCP selection and A2A task scoping without fallback records');

    traces = ['first-trace', 'second-trace'].map((trace_id, i) => ({ trace_id, req_id: 'req-' + i, timestamp: '2026-09-07T10:00:00Z', protocol: 'http', route: 'route-' + i, backend: 'backend-' + i, service: 'service-' + i, method: 'GET', path: '/orders', status_code: 200, duration_ms: 12, status: 'Success', error: '', spans: [{ name: 'Gateway', service: 'gateway', duration_ms: 12, offset_ms: 0, status: 'Success' }] }));
    securityEvents = [{ event_id: 'decision-1', trace_id: 'first-trace', timestamp: '2026-09-07T10:00:00Z', protocol: 'http', route: 'route-0', backend: 'backend-0', principal: 'user:test', actor: 'test-agent', decision: 'denied', enforcement_point: 'authorization', policy_id: 'deny-policy', reason_code: 'auth.denied', status_code: 403, latency_ms: 2 }];
    await go('services');
    await page.locator('#services-table [data-svc-id]').first().focus();
    await page.keyboard.press('Enter');
    await page.waitForFunction(() => document.activeElement.closest('#svc-drawer'));
    assert.equal(await page.locator('#svc-drawer').getAttribute('aria-modal'), 'true');
    await page.keyboard.press('Escape');
    assert.equal(await page.locator('#svc-drawer').getAttribute('aria-hidden'), 'true');
    assert.equal(await page.evaluate(() => document.activeElement.matches('[data-svc-id]')), true);
    await go('observability');
    await page.locator('[data-trace-idx="0"]').click();
    await page.locator('#act-copy-trace-id').click();
    assert.equal(await page.evaluate(() => window.copiedText), 'first-trace');
    await page.keyboard.press('Escape');
    await page.locator('[data-trace-idx="1"]').click();
    await page.locator('#act-copy-trace-id').click();
    assert.equal(await page.evaluate(() => window.copiedText), 'second-trace');
    await page.keyboard.press('Escape');
    assert.equal(await page.locator('#obs-drawer-backdrop').getAttribute('class'), 'obs-drawer-backdrop');
    await go('security');
    await page.locator('[data-decision-idx="0"]').focus();
    await page.keyboard.press('Enter');
    await page.waitForFunction(() => document.activeElement.closest('#sec-drawer'));
    await page.keyboard.press('Shift+Tab');
    assert.equal(await page.evaluate(() => !!document.activeElement.closest('#sec-drawer')), true);
    await page.keyboard.press('Escape');
    console.log('PASS keyboard drawers, focus return, Escape and current-record clipboard');

    await go('llm');
    assert.equal(await page.locator('#error').isVisible(), false, 'Errors must stay on their relevant page');
    llmFails = true;
    config.backends.push({ name: 'new-account', type: 'llm', provider: 'new-provider', models: [] });
    await go('overview'); await go('llm');
    await idle();
    assert.match(await page.locator('#tab-llm [role="alert"]').innerText(), /503/);
    llmFails = false;
    await go('overview'); await go('llm');
    await idle();
    const count = requests.length;
    await page.waitForTimeout(4500);
    assert.ok(requests.length > count);
    await idle();
    console.log('PASS independent API failure, recovery and automatic polling');

    const screenshots = process.env.UI_SCREENSHOT_DIR;
    if (screenshots) {
      await page.screenshot({ path: path.join(screenshots, 'transit-ui-llm-desktop.png'), fullPage: true });
    }
    for (const width of [1440, 1024, 768, 390]) {
      await page.setViewportSize({ width, height: 1000 });
      for (const tab of ['overview', 'services', 'llm', 'mcp', 'a2a', 'observability', 'security', 'cost-control']) {
        await go(tab);
        const sizes = await page.evaluate(() => ({ width: innerWidth, scroll: document.documentElement.scrollWidth }));
        if (sizes.scroll > sizes.width + 1) {
          const offenders = await page.evaluate(() => Array.from(document.querySelectorAll('body *')).filter(el => el.getBoundingClientRect().right > innerWidth + 1 || el.getBoundingClientRect().left < -1).slice(0, 8).map(el => ({ tag: el.tagName, id: el.id, cls: el.className, right: Math.round(el.getBoundingClientRect().right), width: Math.round(el.getBoundingClientRect().width) })));
          console.log('OVERFLOW', tab, width, sizes, offenders);
        }
        assert.ok(sizes.scroll <= sizes.width + 1, `${tab} overflows at ${width}px: ${sizes.scroll}`);
      }
    }
    await go('llm');
    await page.locator('#theme-toggle').click();
    await idle();
    const darkTheme = await page.evaluate(() => ({
      theme: document.documentElement.dataset.theme,
      cardBackground: getComputedStyle(document.querySelector('.card') || document.body).backgroundColor,
      cardColor: getComputedStyle(document.querySelector('.card') || document.body).color,
      tabBackground: getComputedStyle(document.querySelector('.nav button.active') || document.body).backgroundColor,
      tabColor: getComputedStyle(document.querySelector('.nav button.active') || document.body).color
    }));
    assert.equal(darkTheme.theme, 'dark');
    assert.notEqual(darkTheme.cardBackground, 'rgb(255, 255, 255)');
    assert.notEqual(darkTheme.cardColor, darkTheme.cardBackground);
    assert.notEqual(darkTheme.tabColor, darkTheme.tabBackground);
    if (screenshots) await page.screenshot({ path: path.join(screenshots, 'transit-ui-llm-mobile.png'), fullPage: true });
    assert.deepEqual(errors, []);
    console.log('PASS all pages at 1440/1024/768/390px, theme switch and no JavaScript errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
