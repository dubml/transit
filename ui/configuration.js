/* Gateway access settings. Credentials stay in this page's memory. */
const configurationState = { data: null, draft: null, dirty: false, loading: null, language: '', connected: false, error: '', tab: 'access', initialized: false };
let managementBootstrap = null;

async function managementFetch(url, options = {}) {
  if (!llmManagementToken) {
    if (!managementBootstrap) managementBootstrap = (async () => {
      const status = await fetch('/admin/access/status', { cache: 'no-store', signal: AbortSignal.timeout(8000) });
      if (!status.ok) return;
      const data = await status.json();
      if (data.enabled && data.management_mode === 'local') {
        const session = await fetch('/admin/session', { method: 'POST', cache: 'no-store', headers: { 'Content-Type': 'application/json' }, body: '{}', signal: AbortSignal.timeout(8000) });
        if (session.ok) llmManagementToken = (await session.json()).token;
      }
    })().catch(() => {});
    await managementBootstrap;
  }
  const headers = new Headers(options.headers || {});
  const sentToken = llmManagementToken;
  if (sentToken) headers.set('Authorization', 'Bearer ' + sentToken);
  const response = await fetch(url, { cache: 'no-store', ...options, headers });
  if (response.status === 401 && llmManagementToken === sentToken) { llmManagementToken = ''; managementBootstrap = null; }
  return response;
}

function configurationIcon(name) {
  const paths = {
    key: '<circle cx="8" cy="8" r="5"/><path d="m12 12 9 9m-5-5 3-3m-6 0 3-3"/>',
    copy: '<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V4H4v12h4"/>',
    shield: '<path d="M12 3 3 7v6c0 5 9 9 9 9s9-4 9-9V7l-9-4Z"/><path d="m8 12 3 3 5-6"/>',
    chevron: '<path d="m8 10 4 4 4-4"/>',
    code: '<path d="m8 6-6 6 6 6m8-12 6 6-6 6m-3-16-2 20"/>',
    plus: '<path d="M12 5v14M5 12h14"/>'
  };
  return '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' + (paths[name] || '') + '</svg>';
}

function renderConfigurationPage() {
  const root = $('configuration-workspace');
  if (!root) return;
  if (!root.dataset.mounted || configurationState.language !== state.lang) {
    configurationState.openPanels = [...root.querySelectorAll('details[open]')].map(el => el.id);
    configurationState.language = state.lang;
    root.dataset.mounted = 'true';
    root.innerHTML = '<div class="cfg-status"><div class="cfg-product"><span class="cfg-product-name">Transit</span><span id="cfg-version" class="cfg-version">—</span></div><div class="cfg-connection"><span class="cfg-dot"></span><span id="cfg-connection-text">' + uiText('Connecting', '连接中') + '</span><span class="cfg-address">' + esc(location.origin) + '</span></div></div>'
      + '<div class="cfg-heading"><div><h2>' + uiText('Configuration', '配置') + '</h2><p>' + uiText('Access, credentials and secure connections.', '管理接入、认证与安全连接。') + '</p></div><div class="cfg-heading-actions"><button type="button" id="cfg-reload" class="quiet-button">' + llmIcon('refresh') + '<span>' + uiText('Reload', '重新加载') + '</span></button><button type="submit" form="cfg-form" id="cfg-save" class="quiet-button cfg-primary" disabled>' + uiText('Save changes', '保存更改') + '</button></div></div>'
      + '<div class="cfg-tabs" role="tablist" aria-label="' + uiText('Configuration sections', '配置分类') + '"><button type="button" id="cfg-access-tab" role="tab" aria-controls="cfg-access-panel">' + configurationIcon('key') + uiText('Access & authentication', '接入与认证') + '</button><button type="button" id="cfg-runtime-tab" role="tab" aria-controls="cfg-runtime-panel">' + configurationIcon('code') + uiText('Runtime configuration', '运行配置') + '</button></div>'
      + '<p id="cfg-message" class="cfg-message hidden" role="status" aria-live="polite"></p><p id="cfg-error" class="cfg-error hidden" role="alert"></p>'
      + '<div id="cfg-access-panel" role="tabpanel" aria-labelledby="cfg-access-tab"><div id="cfg-auth" class="cfg-auth hidden">' + configurationIcon('shield') + '<div><strong>' + uiText('Management authentication', '管理认证') + '</strong><p>' + uiText('Use the management key for this gateway to view and save settings.', '输入此网关的管理密钥，查看和保存配置。') + '</p></div><button id="cfg-connect" class="quiet-button cfg-primary" type="button">' + uiText('Connect', '连接') + '</button></div><div id="cfg-form-host"></div></div>'
      + '<div id="cfg-runtime-panel" role="tabpanel" aria-labelledby="cfg-runtime-tab" class="hidden"><div class="cfg-json-head"><p>' + uiText('Current routes and backends, supplied by the configuration source.', '配置源提供的当前路由与后端。') + '</p><button id="config-copy-btn" class="quiet-button" type="button">' + configurationIcon('copy') + uiText('Copy JSON', '复制 JSON') + '</button></div><pre id="config-json-viewer"></pre></div>';
    $('cfg-reload').onclick = () => {
      if (!configurationState.dirty) return configurationRefresh(true);
      llmModal(uiText('Reload configuration', '重新加载配置'), '<p>' + uiText('Reloading discards unsaved changes in this form.', '重新加载将放弃表单中尚未保存的更改。') + '</p>', uiText('Reload', '重新加载'), () => configurationRefresh(true));
    };
    $('cfg-connect').onclick = configurationConnect;
    for (const tab of ['access', 'runtime']) $('cfg-' + tab + '-tab').onclick = () => { configurationState.tab = tab; configurationTabs(); };
    root.querySelector('[role=tablist]').onkeydown = event => {
      if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
      event.preventDefault(); configurationState.tab = event.key === 'Home' ? 'access' : event.key === 'End' ? 'runtime' : event.target.id === 'cfg-access-tab' ? 'runtime' : 'access'; configurationTabs(); $('cfg-' + configurationState.tab + '-tab').focus();
    };
    configurationForm();
    const build = configurationState.data?.build;
    if (build) {
      $('cfg-version').textContent = 'v' + build.version;
      root.querySelector('.cfg-product-name').textContent = build.name;
      $('cfg-connection-text').textContent = configurationState.connected ? uiText('Connected', '已连接') : uiText('Disconnected', '连接中断');
      root.querySelector('.cfg-dot').classList.toggle('connected', configurationState.connected);
    }
    if (configurationState.dirty) configurationMessage(uiText('Unsaved changes', '有未保存的更改'));
    else if (configurationState.data?.settings.restart_required) configurationMessage(uiText('Saved. Restart the gateway to apply the address, directory or TLS changes.', '已保存。地址、目录或 TLS 的更改将在网关重启后生效。'));
  }
  configurationTabs();
  $('config-json-viewer').textContent = JSON.stringify(state.config || {}, null, 2);
  $('config-copy-btn').onclick = () => copyText(JSON.stringify(state.config || {}, null, 2));
  if (!configurationState.initialized) configurationRefresh(false);
}

function configurationTabs() {
  for (const tab of ['access', 'runtime']) {
    const active = configurationState.tab === tab;
    $('cfg-' + tab + '-tab').setAttribute('aria-selected', String(active));
    $('cfg-' + tab + '-tab').tabIndex = active ? 0 : -1;
    $('cfg-' + tab + '-panel').classList.toggle('hidden', !active);
  }
  $('cfg-save').classList.toggle('hidden', configurationState.tab !== 'access');
}

function configurationField(name, title, value, type = 'text', hint = '') {
  return '<label class="cfg-field"><span>' + esc(title) + '</span><input id="cfg-' + name + '" name="' + name + '" type="' + type + '" value="' + esc(value) + '"' + (type === 'number' ? ' min="1" max="65535" step="1"' : '') + ' autocomplete="off" spellcheck="false">' + (hint ? '<small>' + esc(hint) + '</small>' : '') + '</label>';
}

function configurationToggle(name, title, hint, checked) {
  return '<label class="cfg-toggle"><span><strong>' + esc(title) + '</strong><small>' + esc(hint) + '</small></span><input id="cfg-' + name + '" name="' + name + '" type="checkbox" role="switch"' + (checked ? ' checked' : '') + '></label>';
}

function configurationForm() {
  const draft = configurationState.draft;
  const host = $('cfg-form-host');
  if (!host || !draft) return;
  const remote = draft['remote-management'];
  const open = new Set([...host.querySelectorAll('details[open]')].map(el => el.id).concat(configurationState.openPanels || []));
  configurationState.openPanels = [];
  host.innerHTML = '<form id="cfg-form"><fieldset id="cfg-fields"><div class="cfg-card"><div class="cfg-card-title"><h3>' + uiText('Gateway access', '网关接入') + '</h3><span class="cfg-note">' + uiText('Applied after restart', '重启后生效') + '</span></div><div class="cfg-field-grid cfg-address-fields">'
    + configurationField('host', uiText('Host address', '主机地址'), draft.host)
    + configurationField('port', uiText('Port', '端口'), draft.port, 'number') + '</div>'
    + configurationField('auth-dir', uiText('Authentication directory', '认证文件目录'), draft['auth-dir'], 'text', uiText('OAuth account files · supports ~/ · existing files are not moved.', '存放 OAuth 账户文件，支持 ~/；更换目录不会移动已有文件。')) + '</div>'
    + '<div class="cfg-card"><div class="cfg-card-title"><div><h3>' + uiText('API keys', 'API 密钥') + '</h3><p>' + uiText('Authenticate clients calling this gateway.', '用于验证调用此网关的客户端。') + '</p></div><button type="button" id="cfg-add-key" class="quiet-button">' + configurationIcon('plus') + uiText('Add key', '添加密钥') + '</button></div><div id="cfg-keys"></div><p class="cfg-footnote">' + uiText('Without keys, this gateway-level check is off. Route authentication still applies.', '列表为空时不启用网关级密钥校验，路由自身的认证规则仍然有效。') + '</p></div>'
    + '<details id="cfg-tls" class="cfg-card cfg-disclosure"' + (open.has('cfg-tls') ? ' open' : '') + '><summary><span>TLS / SSL <small>' + uiText('HTTPS connection', 'HTTPS 安全连接') + '</small></span>' + configurationIcon('chevron') + '</summary><div class="cfg-disclosure-body">'
    + configurationToggle('tls-enable', uiText('Enable TLS', '启用 TLS'), uiText('Secure both gateway and management listeners. Restart required.', '为网关和管理端口启用 HTTPS，重启后生效。'), draft.tls.enable)
    + '<div id="cfg-tls-fields" class="cfg-field-grid' + (draft.tls.enable ? '' : ' hidden') + '">' + configurationField('tls-cert', uiText('Certificate file (PEM)', '证书文件（PEM）'), draft.tls.cert) + configurationField('tls-key', uiText('Private key file (PEM)', '私钥文件（PEM）'), draft.tls.key) + '</div></div></details>'
    + '<details id="cfg-remote" class="cfg-card cfg-disclosure"' + (open.has('cfg-remote') ? ' open' : '') + '><summary><span>' + uiText('Remote management', '远程管理') + '<small>' + uiText('Access and control panel', '访问权限与控制面板') + '</small></span>' + configurationIcon('chevron') + '</summary><div class="cfg-disclosure-body"><div class="cfg-toggles">'
    + configurationToggle('allow-remote', uiText('Allow remote access', '允许远程访问'), uiText('Requires a management key and an externally reachable UI listener.', '需要管理密钥，管理端口也须允许外部连接。'), remote['allow-remote'])
    + configurationToggle('disable-control-panel', uiText('Disable control panel', '禁用控制面板'), uiText('Hide the web interface. Management APIs remain available.', '关闭网页入口，管理 API 仍可使用。'), remote['disable-control-panel'])
    + configurationToggle('disable-auto-update-panel', uiText('Disable panel auto-update', '禁用面板自动更新'), uiText('Download once if missing; skip later background updates.', '首次缺失时下载，此后停止后台自动更新。'), remote['disable-auto-update-panel'])
    + '</div><div class="cfg-field-grid">' + configurationField('secret', uiText('Management key', '管理密钥'), configurationState.secret || '', 'password', configurationState.data.settings.secret_configured ? uiText('Saved · leave blank to keep the current key.', '已设置；留空保留当前密钥。') : uiText('At least 24 characters. Local access uses a temporary session until set.', '至少 24 个字符。未设置时，本机使用临时管理会话。'))
    + configurationField('panel-repository', uiText('Panel repository', '面板仓库'), remote['panel-github-repository'], 'url', uiText('Optional GitHub release with a compatible management.html; blank uses the bundled panel.', '可选：提供兼容 management.html 的 GitHub 仓库；留空使用内置面板。')) + '</div>'
    + (configurationState.data.settings.secret_configured ? '<label class="cfg-clear-key"><input id="cfg-clear-secret" type="checkbox"' + (configurationState.clearSecret ? ' checked' : '') + '>' + uiText('Clear the saved management key', '清除已保存的管理密钥') + '</label>' : '')
    + '<p id="cfg-panel-status" class="cfg-footnote"></p></div></details></fieldset></form>';
  $('cfg-form').onsubmit = configurationSave;
  $('cfg-form').oninput = () => {
    configurationReadForm();
    configurationState.dirty = true;
    $('cfg-save').disabled = false;
    $('cfg-tls-fields').classList.toggle('hidden', !$('cfg-tls-enable').checked);
    $('cfg-tls-cert').required = $('cfg-tls-key').required = $('cfg-tls-enable').checked;
    configurationMessage(uiText('Unsaved changes', '有未保存的更改'));
  };
  $('cfg-host').required = $('cfg-port').required = $('cfg-auth-dir').required = true;
  $('cfg-tls-cert').required = $('cfg-tls-key').required = draft.tls.enable;
  $('cfg-secret').minLength = 24;
  $('cfg-add-key').onclick = () => configurationEditKey(-1);
  $('cfg-panel-status').textContent = configurationState.data.panel?.error || '';
  configurationKeys();
  $('cfg-save').disabled = !configurationState.dirty;
}

function configurationReadForm() {
  const draft = configurationState.draft;
  if (!draft || !$('cfg-form')) return;
  draft.host = $('cfg-host').value;
  draft.port = Number($('cfg-port').value);
  draft['auth-dir'] = $('cfg-auth-dir').value;
  draft.tls = { enable: $('cfg-tls-enable').checked, cert: $('cfg-tls-cert').value, key: $('cfg-tls-key').value };
  for (const name of ['allow-remote', 'disable-control-panel', 'disable-auto-update-panel']) draft['remote-management'][name] = $('cfg-' + name).checked;
  draft['remote-management']['panel-github-repository'] = $('cfg-panel-repository').value;
  configurationState.secret = $('cfg-secret').value;
  configurationState.clearSecret = Boolean($('cfg-clear-secret')?.checked);
}

function configurationKeys() {
  const root = $('cfg-keys');
  const keys = configurationState.draft['api-keys'];
  root.innerHTML = keys.length ? keys.map((key, i) => '<div class="cfg-key"><span class="cfg-key-index">' + String(i + 1).padStart(2, '0') + '</span><code>' + esc(key.slice(0, 2) + '••••••••' + key.slice(-2)) + '</code><div class="cfg-key-actions">' + ['copy', 'edit', 'delete'].map(action => '<button type="button" class="quiet-button' + (action === 'delete' ? ' cfg-danger' : '') + '" data-key="' + i + '" data-action="' + action + '" aria-label="' + esc(({copy:uiText('Copy key', '复制密钥'),edit:uiText('Edit key', '编辑密钥'),delete:uiText('Delete key', '删除密钥')})[action] + ' ' + (i + 1)) + '" title="' + esc(({copy:uiText('Copy', '复制'),edit:uiText('Edit', '编辑'),delete:uiText('Delete', '删除')})[action]) + '">' + (action === 'copy' ? configurationIcon('copy') : llmIcon(action)) + '</button>').join('') + '</div></div>').join('') : '<p class="cfg-empty">' + uiText('No API keys yet', '尚未添加 API 密钥') + '</p>';
  root.querySelectorAll('[data-action]').forEach(button => button.onclick = async () => {
    const index = Number(button.dataset.key);
    if (button.dataset.action === 'copy') { try { await navigator.clipboard.writeText(keys[index]); notify(uiText('Key copied', '密钥已复制')); } catch (_) { notify(uiText('Clipboard unavailable', '剪贴板不可用')); } return; }
    if (button.dataset.action === 'edit') return configurationEditKey(index);
    keys.splice(index, 1); configurationChanged(); configurationKeys(); $('cfg-add-key').focus();
  });
}

function configurationChanged() {
  configurationState.dirty = true; $('cfg-save').disabled = false;
  configurationMessage(uiText('Unsaved changes', '有未保存的更改'));
}

function configurationEditKey(index) {
  const keys = configurationState.draft['api-keys'];
  const generate = () => 'tr-' + [...crypto.getRandomValues(new Uint8Array(24))].map(byte => byte.toString(16).padStart(2, '0')).join('');
  const dialog = llmModal(index < 0 ? uiText('Add API key', '添加 API 密钥') : uiText('Edit API key', '编辑 API 密钥'), '<label>' + uiText('API key', 'API 密钥') + '<input name="key" type="password" autocomplete="off" required minlength="8" maxlength="4096" value="' + esc(index < 0 ? generate() : keys[index]) + '"></label><button type="button" class="quiet-button" data-generate>' + uiText('Generate a new key', '生成新密钥') + '</button>', uiText('Apply', '应用'), form => {
    const value = form.querySelector('[name=key]').value;
    if (keys.some((key, i) => key === value && i !== index)) throw new Error(uiText('This key already exists.', '此密钥已存在。'));
    if (index < 0) keys.push(value); else keys[index] = value;
    configurationChanged(); configurationKeys();
  });
  dialog.querySelector('[data-generate]').onclick = () => { dialog.querySelector('[name=key]').value = generate(); };
}

function configurationMessage(message, error = false) {
  const target = $(error ? 'cfg-error' : 'cfg-message');
  if (target) { target.textContent = message; target.classList.toggle('hidden', !message); }
}

async function configurationRefresh(discard = false) {
  if (configurationState.saving) return;
  if (configurationState.loading) return configurationState.loading;
  configurationState.initialized = true;
  configurationState.loading = (async () => {
    $('cfg-reload').disabled = true;
    try {
      const health = await fetch('/healthz', { cache: 'no-store', signal: AbortSignal.timeout(8000) });
      if (!health.ok) throw new Error('HTTP ' + health.status);
      const build = await health.json();
      $('cfg-version').textContent = build.version ? 'v' + build.version : '—';
      $('configuration-workspace').querySelector('.cfg-product-name').textContent = build.name || 'Transit';
      const response = await managementFetch('/admin/access', { signal: AbortSignal.timeout(10000) });
      const body = await response.json().catch(() => ({}));
      if (!response.ok) {
        const locked = response.status === 401;
        $('cfg-auth').classList.toggle('hidden', !locked);
        $('cfg-form-host').classList.add('hidden');
        configurationState.connected = false;
        $('cfg-connection-text').textContent = locked ? uiText('Authentication required', '等待认证') : uiText('Unavailable', '不可用');
        $('configuration-workspace').querySelector('.cfg-dot').classList.remove('connected');
        if (!locked) throw new Error(body.error || 'HTTP ' + response.status);
        return;
      }
      const unchanged = configurationState.data?.settings.revision === body.settings.revision;
      configurationState.data = body;
      configurationState.connected = true;
      $('cfg-connection-text').textContent = uiText('Connected', '已连接');
      $('configuration-workspace').querySelector('.cfg-dot').classList.add('connected');
      $('cfg-auth').classList.add('hidden'); $('cfg-form-host').classList.remove('hidden');
      configurationMessage('', true);
      if ((!configurationState.dirty && (!unchanged || !configurationState.draft)) || discard) {
        configurationState.draft = structuredClone(body.settings.config);
        configurationState.baseRevision = body.settings.revision;
        configurationState.dirty = false; configurationState.secret = ''; configurationState.clearSecret = false;
        configurationForm();
        configurationMessage(body.settings.restart_required ? uiText('Saved. Restart the gateway to apply the address, directory or TLS changes.', '已保存。地址、目录或 TLS 的更改将在网关重启后生效。') : '');
      }
    } catch (error) {
      configurationState.connected = false;
      $('configuration-workspace').querySelector('.cfg-dot').classList.remove('connected');
      $('cfg-connection-text').textContent = uiText('Disconnected', '连接中断');
      configurationMessage(error.message, true);
    } finally {
      $('cfg-reload').disabled = false;
      if ($('cfg-fields')) $('cfg-fields').disabled = !configurationState.connected;
      $('cfg-save').disabled = !configurationState.connected || !configurationState.dirty;
      configurationState.loading = null;
    }
  })();
  return configurationState.loading;
}

function configurationConnect() {
  llmModal(uiText('Management authentication', '管理认证'), '<label>' + uiText('Management key', '管理密钥') + '<input name="key" type="password" required minlength="24" autocomplete="off"></label>', uiText('Connect', '连接'), async dialog => {
    const key = dialog.querySelector('[name=key]').value;
    const response = await fetch('/admin/access', { headers: { Authorization: 'Bearer ' + key }, cache: 'no-store', signal: AbortSignal.timeout(10000) });
    if (!response.ok) throw new Error((await response.json().catch(() => ({}))).error || 'HTTP ' + response.status);
    llmManagementToken = key;
    await configurationRefresh(false);
    load();
  });
}

async function configurationSave(event) {
  event.preventDefault();
  if (configurationState.loading) await configurationState.loading;
  if (!configurationState.connected) return;
  configurationState.saving = true;
  configurationReadForm();
  const request = { revision: configurationState.baseRevision, config: configurationState.draft };
  if (configurationState.clearSecret) request.secret = '';
  else if (configurationState.secret) request.secret = configurationState.secret;
  $('cfg-save').disabled = true; $('cfg-fields').disabled = true;
  configurationMessage('', true);
  try {
    const response = await managementFetch('/admin/access', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(request), signal: AbortSignal.timeout(20000) });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.error || 'HTTP ' + response.status);
    if (request.secret !== undefined) { llmManagementToken = request.secret; managementBootstrap = null; }
    configurationState.data = body; configurationState.draft = structuredClone(body.settings.config); configurationState.baseRevision = body.settings.revision; configurationState.dirty = false; configurationState.secret = ''; configurationState.clearSecret = false;
    configurationForm();
    configurationMessage(body.settings.restart_required ? uiText('Saved. Address, directory and TLS changes take effect after restart.', '已保存。地址、目录和 TLS 的更改将在重启后生效。') : uiText('Saved and applied.', '已保存并生效。'));
    if (body.settings.config['remote-management']['disable-control-panel']) configurationMessage(uiText('Saved. The control panel is disabled on the next page load. Re-enable it through the management API or access configuration file.', '已保存，下次加载将关闭控制面板。可通过管理 API 或接入配置文件重新启用。'));
  } catch (error) { configurationMessage(error.message, true); }
  finally { configurationState.saving = false; $('cfg-fields').disabled = false; $('cfg-save').disabled = !configurationState.dirty; }
}

setInterval(() => { if (typeof state !== 'undefined' && state.tab === 'configuration' && !document.hidden && !state.paused) configurationRefresh(false); }, 10000);
