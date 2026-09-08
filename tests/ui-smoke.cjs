// Run with Node.js and Playwright available on NODE_PATH. No npm install or gateway needed.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const demo = JSON.parse(fs.readFileSync(path.join(__dirname, 'demo-config.json'), 'utf8'));
const empty = { listeners: [], clusters: [], backends: [], providers: [], routes: [], policies: [] };
let config = empty;
let costFails = false;
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
    const file = path.join(root, 'logo', path.basename(url.pathname));
    res.setHeader('Content-Type', 'image/svg+xml');
    res.end(fs.readFileSync(file));
  } else {
    res.setHeader('Content-Type', 'application/json');
    if (url.pathname === '/debug/config') res.end(JSON.stringify(config));
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
    assert.match(await page.locator('#llm-accounts-grid').innerText(), /No accounts/);
    assert.deepEqual(errors, []);
    console.log('PASS empty configuration across all 8 pages');

    config = structuredClone(demo);
    config.backends.push({ name: "weekly-only's <account>", type: 'llm', provider: 'custom-provider', account_type: 'subscription', models: ['custom-model'], quota_state: { windows: [{ name: 'weekly', window: '7d', used_percent: 0 }] } });
    await page.locator('#reload-data').click();
    await idle();
    assert.equal(await page.locator('.llm-account-card').count(), config.backends.filter(b => b.type === 'llm').length);
    await page.locator('[data-provider="custom-provider"]').click();
    assert.equal(await page.locator('.llm-account-card').count(), 1);
    assert.equal(await page.locator('.reset-cards-box').count(), 0);
    assert.equal(await page.locator('.quota-window-item').count(), 1);
    assert.match(await page.locator('.llm-account-card').innerText(), /0%/);
    await page.locator('.llm-account-card .action-enable').click();
    assert.match(await page.locator('#ui-toast').innerText(), /not connected/);
    assert.match(await page.locator('.llm-account-card').innerText(), /Configured/);
    await page.locator('#ui-toast button').click();
    await page.locator('[data-provider="all"]').click();
    await page.locator('#search-input').fill('does-not-exist');
    assert.equal(await page.locator('.llm-account-card:visible').count(), 0);
    assert.equal(await page.locator('#page-search-empty').isVisible(), true);
    await page.locator('#search-input').fill('');
    console.log('PASS provider filters, reported-only quotas, safe names and truthful actions');

    await go('mcp');
    assert.equal(await page.locator('#tbody-mcp-target-servers tr[data-mcp-server]').count(), 1);
    assert.equal(await page.locator('#tbody-mcp-capabilities tr').count(), 2);
    assert.equal(await page.locator('#mcp-invocation-detail').isVisible(), false);
    await page.locator('#examples-toggle').click();
    await page.locator('[data-mcp-server="database-mcp"]').click();
    assert.equal(await page.locator('#tbody-mcp-invocations tr').count(), 1);
    assert.match(await page.locator('#tbody-mcp-invocations').innerText(), /database-mcp/);
    await go('a2a');
    await page.locator('[data-a2a-agent="fraud-agent"]').click();
    await page.locator('[data-task-tab="working"]').click();
    assert.match(await page.locator('#tbody-a2a-tasks').innerText(), /No matching/);
    assert.equal(await page.locator('#a2a-task-drawer').isVisible(), false);
    await page.locator('#examples-toggle').click();
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
    costFails = true;
    config.backends.push({ name: 'new-account', type: 'llm', provider: 'new-provider', models: [] });
    await page.locator('#reload-data').click();
    await idle();
    assert.equal(await page.locator('[data-provider="new-provider"]').count(), 1);
    assert.match(await page.locator('#error').innerText(), /503/);
    costFails = false;
    await page.locator('#reload-data').click();
    await idle();
    await page.locator('#poll-toggle').click();
    const count = requests.length;
    await page.waitForTimeout(4200);
    assert.equal(requests.length, count);
    await page.locator('#poll-toggle').click();
    await idle();
    console.log('PASS independent API failure, recovery and paused polling');

    const screenshots = process.env.UI_SCREENSHOT_DIR;
    if (screenshots) {
      await page.screenshot({ path: path.join(screenshots, 'dxgate-ui-llm-desktop.png'), fullPage: true });
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
    const darkTheme = await page.evaluate(() => {
      const card = document.querySelector('.llm-account-card');
      const tab = document.querySelector('.llm-provider-tab');
      const cardStyle = getComputedStyle(card);
      const tabStyle = getComputedStyle(tab);
      return { theme: document.documentElement.dataset.theme, cardBackground: cardStyle.backgroundColor, cardColor: cardStyle.color, tabBackground: tabStyle.backgroundColor, tabColor: tabStyle.color };
    });
    assert.equal(darkTheme.theme, 'dark');
    assert.notEqual(darkTheme.cardBackground, 'rgb(255, 255, 255)');
    assert.notEqual(darkTheme.cardColor, darkTheme.cardBackground);
    assert.notEqual(darkTheme.tabColor, darkTheme.tabBackground);
    if (screenshots) await page.screenshot({ path: path.join(screenshots, 'dxgate-ui-llm-mobile.png'), fullPage: true });
    assert.deepEqual(errors, []);
    console.log('PASS all pages at 1440/1024/768/390px, theme switch and no JavaScript errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
