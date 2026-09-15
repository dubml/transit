/* OAuth account workspace. Loaded before the shared UI bootstraps. */
let llmManagementToken = '';

const llmProviderInfo = {
  chatgpt: { label: 'OpenAI', oauth: 'codex' },
  anthropic: { label: 'Anthropic', oauth: 'claude' },
  antigravity: { label: 'Antigravity', oauth: 'antigravity' }
};

function llmFamilyForProvider(provider) {
  if (provider === 'antigravity') return 'antigravity';
  return provider === 'claude' ? 'anthropic' : 'chatgpt';
}

const llmProviderLogos = {
  chatgpt: '<svg class="llm-provider-logo llm-provider-logo-chatgpt" viewBox="0 0 24 24" fill="currentColor" fill-rule="evenodd" clip-rule="evenodd" aria-hidden="true" focusable="false" xmlns="http://www.w3.org/2000/svg"><path d="M22.282 9.821a6 6 0 0 0-.516-4.91a6.05 6.05 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a6 6 0 0 0-3.998 2.9a6.05 6.05 0 0 0 .743 7.097a5.98 5.98 0 0 0 .51 4.911a6.05 6.05 0 0 0 6.515 2.9A6 6 0 0 0 13.26 24a6.06 6.06 0 0 0 5.772-4.206a6 6 0 0 0 3.997-2.9a6.06 6.06 0 0 0-.747-7.073M13.26 22.43a4.48 4.48 0 0 1-2.876-1.04l.141-.081l4.779-2.758a.8.8 0 0 0 .392-.681v-6.737l2.02 1.168a.07.07 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494M3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085l4.783 2.759a.77.77 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646M2.34 7.896a4.5 4.5 0 0 1 2.366-1.973V11.6a.77.77 0 0 0 .388.677l5.815 3.354l-2.02 1.168a.08.08 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.872zm16.597 3.855l-5.833-3.387L15.119 7.2a.08.08 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667m2.01-3.023l-.141-.085l-4.774-2.782a.78.78 0 0 0-.785 0L9.409 9.23V6.897a.07.07 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08L8.704 5.46a.8.8 0 0 0-.393.681zm1.097-2.365l2.602-1.5l2.607 1.5v2.999l-2.597 1.5l-2.607-1.5Z"></path></svg>',
  anthropic: '<svg class="llm-provider-logo llm-provider-logo-anthropic" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" focusable="false" xmlns="http://www.w3.org/2000/svg"><path d="M17.304 3.541h-3.672l6.696 16.918H24Zm-10.608 0L0 20.459h3.744l1.37-3.553h7.005l1.369 3.553h3.744L10.536 3.541Zm-.371 10.223L8.616 7.82l2.291 5.945Z"></path></svg>',
  antigravity: '<svg class="llm-provider-logo llm-provider-logo-antigravity" viewBox="0 0 112 114" aria-hidden="true" focusable="false" xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="antigravity-logo-gradient" x1="18" y1="96" x2="97" y2="18" gradientUnits="userSpaceOnUse"><stop stop-color="#3186FF"></stop><stop offset=".34" stop-color="#00B95C"></stop><stop offset=".67" stop-color="#FBBC04"></stop><stop offset="1" stop-color="#FC413D"></stop></linearGradient></defs><path fill="url(#antigravity-logo-gradient)" d="M89.699 93.695c4.667 3.5 11.667 1.167 5.25-5.25C75.699 69.778 79.783 18.445 55.866 18.445S36.033 69.778 16.783 88.445c-7 7 0.583 8.75 5.25 5.25 18.083-12.25 16.917-33.833 33.833-33.833s15.75 21.583 33.833 33.833Z"></path></svg>'
};

function llmProviderMark(family) {
  return llmProviderLogos[family] || '';
}

function llmIcon(name) {
  const paths = {
    login: '<path d="M14 4h6v16h-6M3 12h12m-4-4 4 4-4 4"/>',
    refresh: '<path d="M3 11a9 9 0 0 1 15.4-5.4L21 8M21 3v5h-5M21 13a9 9 0 0 1-15.4 5.4L3 16M3 21v-5h5"/>',
    edit: '<path d="m15 4 5 5M4 20l5-1L20 8a2 2 0 0 0-5-5L4 14z"/>',
    upload: '<path d="M12 16V3m-5 5 5-5 5 5M4 16v5h16v-5"/>',
    download: '<path d="M12 3v12m-5-5 5 5 5-5M3 15v4a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-4"/>',
    models: '<path d="m12 3 9 5-9 5-9-5 9-5ZM3 12l9 5 9-5M3 16l9 5 9-5"/>',
    delete: '<path d="M3 6h18M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M5 6v13a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V6M10 10v7m4-7v7"/>',
    settings: '<g transform="translate(2 2) scale(1.25)" stroke-width="1.44"><circle cx="8" cy="8" r="2.3"/><path d="M6.8 1.5h2.4l.4 1.5c.4.1.7.3 1.1.5l1.4-.7 1.7 1.7-.7 1.4c.2.4.4.7.5 1.1l1.5.4v2.4l-1.5.4c-.1.4-.3.7-.5 1.1l.7 1.4-1.7 1.7-1.4-.7c-.4.2-.7.4-1.1.5l-.4 1.5H6.8l-.4-1.5c-.4-.1-.7-.3-1.1-.5l-1.4.7-1.7-1.7.7-1.4a3.8 3.8 0 0 1-.5-1.1L1 9.2V6.8l1.5-.4c.1-.4.3-.7.5-1.1l-.7-1.4 1.7-1.7 1.4.7c.4-.2.7-.4 1.1-.5l.4-1.5z"/></g>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    codex: '<path d="m6 7 4 5-4 5m7 0h5"/>'
  };
  return '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">' + paths[name] + '</svg>';
}

function llmModal(title, content, saveLabel, onSave) {
  const previous = document.activeElement;
  const dialog = document.createElement('dialog');
  dialog.className = 'llm-dialog';
  dialog.innerHTML = '<form><header><h2>' + esc(title) + '</h2><button type="button" class="quiet-button" data-close aria-label="' + esc(uiText('Close', '关闭')) + '">×</button></header><div class="llm-dialog-body">' + content + '</div><p class="llm-form-error" role="alert"></p><footer><button type="button" class="quiet-button" data-close>' + (saveLabel ? uiText('Cancel', '取消') : uiText('Close', '关闭')) + '</button>' + (saveLabel ? '<button class="quiet-button llm-primary" type="submit">' + esc(saveLabel) + '</button>' : '') + '</footer></form>';
  document.body.appendChild(dialog);
  const close = () => dialog.close();
  dialog.querySelectorAll('[data-close]').forEach(button => button.onclick = close);
  dialog.onclose = () => { dialog.replaceChildren(); dialog.remove(); previous?.focus(); };
  dialog.querySelector('form').onsubmit = async event => {
    event.preventDefault();
    const button = dialog.querySelector('[type=submit]');
    if (button) button.disabled = true;
    dialog.querySelector('[role=alert]').textContent = '';
    try { if (await onSave(dialog) !== false) close(); }
    catch (error) { if (dialog.isConnected) dialog.querySelector('[role=alert]').textContent = error.message; }
    finally { if (button) button.disabled = false; }
  };
  dialog.showModal();
  return dialog;
}

function llmUnlock() {
  return new Promise((resolve, reject) => {
    let accepted = false;
    const dialog = llmModal(uiText('Manage OAuth accounts', '管理 OAuth 账户'),
      '<label>' + uiText('Management token', '管理令牌') + '<input name="token" type="password" autocomplete="off" required minlength="24"></label><p class="muted">' + uiText('Used only in this page session.', '仅在当前页面会话内使用。') + '</p>',
      uiText('Continue', '继续'), async form => {
        llmManagementToken = form.querySelector('[name=token]').value;
        accepted = true; resolve();
      });
    dialog.addEventListener('close', () => { if (!accepted) reject(new Error(uiText('Cancelled', '已取消'))); });
  });
}

async function llmAdminFetch(path, options = {}, prefix = '/admin/llm/accounts/', renewLocalSession = true) {
  if (!llmManagementToken) {
    if (state.llmData?.management_mode === 'local') await llmLocalSession();
    else await llmUnlock();
  }
  const response = await fetch(prefix + path, {
    ...options, cache: 'no-store', signal: AbortSignal.timeout(path.endsWith('/quota') || path.endsWith('/reset-quota') ? 90000 : 40000),
    headers: { 'Content-Type': 'application/json', Authorization: 'Bearer ' + llmManagementToken }
  });
  if (!response.ok) {
    if (response.status === 401) llmManagementToken = '';
    if (response.status === 401 && renewLocalSession && state.llmData?.management_mode === 'local') {
      await llmLocalSession();
      return llmAdminFetch(path, options, prefix, false);
    }
    const body = await response.json().catch(() => ({}));
    throw new Error(body.error || 'HTTP ' + response.status);
  }
  return response;
}

async function llmLocalSession() {
  const response = await fetch('/admin/llm/session', { method: 'POST', cache: 'no-store', headers: { 'Content-Type': 'application/json' }, body: '{}', signal: AbortSignal.timeout(8000) });
  if (!response.ok) throw new Error(uiText('Could not establish the local management session. Open the UI directly on localhost.', '无法建立本地管理会话，请通过 localhost 或回环地址直接打开 UI。'));
  llmManagementToken = (await response.json()).token;
}

async function llmLoginDialog(requestedFamily) {
  const family = requestedFamily || state.llmFamily || 'chatgpt';
  const info = llmProviderInfo[family] || llmProviderInfo.chatgpt;
  const name = info.label;
  const provider = info.oauth;
  const managementEnabled = state.llmData?.management_enabled === true;
  const localSession = state.llmData?.management_mode === 'local';
  let login = null, expiresAt = 0, saved = false;
  const content = (managementEnabled ? (localSession ? '' : '<section data-login-management' + (llmManagementToken ? ' hidden' : '') + '><label>' + uiText('Gateway management token', '网关管理令牌') + '<input name="management_token" type="password" autocomplete="off" minlength="24"></label><p class="muted">' + uiText('Use the management key saved in Configuration, or the initial TRANSIT_LLM_ADMIN_TOKEN. It stays in this page session.', '填写 Configuration 中保存的管理密钥，或首次启动时设置的 TRANSIT_LLM_ADMIN_TOKEN；仅用于当前页面会话。') + '</p><button type="button" class="quiet-button llm-primary" data-start>' + uiText('Connect', '连接') + '</button></section>')
      : '<section class="llm-login-setup" role="note"><strong>' + uiText('OAuth management is not enabled', '当前服务尚未启用 OAuth 管理') + '</strong><p>' + uiText('No authorization link can be generated yet. Configure the account directory and management token, restart the gateway, then reload this page.', '目前无法生成授权链接。请配置账户保存目录和管理令牌，重启网关后刷新此页面。') + '</p><code>TRANSIT_LLM_ACCOUNTS_DIR</code><p>' + uiText('A private directory for authentication files.', '用于保存认证文件的私有目录。') + '</p><code>TRANSIT_LLM_ADMIN_TOKEN</code><p>' + uiText('A management token you create, at least 24 characters long.', '由你设置的管理令牌，至少 24 个字符。') + '</p></section>')
    + '<section class="llm-login-link" hidden><p data-url></p><div class="llm-heading-actions"><button type="button" class="quiet-button" data-copy>' + uiText('Copy link', '复制链接') + '</button><a class="quiet-button" data-open target="_blank" rel="noopener noreferrer" referrerpolicy="no-referrer">' + uiText('Open link', '打开链接') + '</a></div></section>';
  const credentials = async form => {
    if (localSession) { if (!llmManagementToken) await llmLocalSession(); return; }
    const input = form.querySelector('[name=management_token]');
    if (input?.value) llmManagementToken = input.value;
    if (!llmManagementToken || llmManagementToken.length < 24) {
      llmManagementToken = '';
      const mgmt = form.querySelector('[data-login-management]');
      if (mgmt) mgmt.hidden = false;
      input?.focus();
      throw new Error(uiText('Enter the gateway management token configured at startup (at least 24 characters).', '请填写启动网关时配置的管理令牌（至少 24 个字符）。'));
    }
  };
  const showCredentialError = form => {
    if (!llmManagementToken && form.isConnected && managementEnabled && !localSession) {
      const mgmt = form.querySelector('[data-login-management]');
      if (mgmt) mgmt.hidden = false;
      const input = form.querySelector('[name=management_token]');
      if (input) input.value = '';
    }
  };
  const dialog = llmModal(name + ' OAuth', content, '', async () => {
    await startLogin();
    return false;
  });
  dialog.classList.add('llm-login-dialog');
  const completed = async account => {
    if (saved) return;
    saved = true; login = null;
    if (dialog.isConnected) dialog.close();
    notify(uiText('Signed in. Authentication file saved: ', '登录成功，认证文件已保存：') + account.id);
    await llmReload().catch(() => notify(uiText('Account saved. Refresh the dashboard to see it.', '账户已保存，请刷新仪表盘查看。')));
  };
  const startLogin = async () => {
    const startBtn = dialog.querySelector('[data-start]');
    if (startBtn) startBtn.disabled = true;
    dialog.querySelector('[role=alert]').textContent = '';
    try {
      await credentials(dialog);
      if (login) { await llmAdminFetch(login.id, { method: 'DELETE' }, '/admin/llm/oauth/'); login = null; }
      const response = await llmAdminFetch('start', { method: 'POST', body: JSON.stringify({ provider, backend: '' }) }, '/admin/llm/oauth/');
      if (!dialog.isConnected) return;
      login = await response.json(); expiresAt = Date.now() + login.expires_in * 1000;
      dialog.dataset.callbackMode = login.callback_mode;
      if (!localSession) {
        const input = dialog.querySelector('[name=management_token]');
        if (input) input.value = '';
        const mgmt = dialog.querySelector('[data-login-management]');
        if (mgmt) mgmt.hidden = true;
      }
      dialog.querySelector('[data-url]').textContent = login.authorization_url;
      dialog.querySelector('[data-open]').href = login.authorization_url;
      dialog.querySelector('.llm-login-link').hidden = false;
    } catch (error) {
      showCredentialError(dialog);
      if (dialog.isConnected) dialog.querySelector('[role=alert]').textContent = error.message;
    } finally {
      if (startBtn && dialog.isConnected) startBtn.disabled = false;
    }
  };
  const startBtn = dialog.querySelector('[data-start]');
  if (startBtn) startBtn.onclick = startLogin;
  dialog.querySelector('[data-copy]').onclick = async () => {
    try { await navigator.clipboard.writeText(login.authorization_url); notify(uiText('Link copied', '链接已复制')); }
    catch { dialog.querySelector('[role=alert]').textContent = uiText('Copy unavailable. Select and copy the link above.', '无法自动复制，请选中上方链接手动复制。'); }
  };
  let polling = false;
  const timer = setInterval(async () => {
    if (login && Date.now() >= expiresAt && !saved) {
      login = null;
      dialog.querySelector('[data-open]').removeAttribute('href');
      dialog.querySelector('[role=alert]').textContent = uiText('Login expired. Start a new login.', '登录已过期，请重新开始登录。');
    }
    if (!login || saved || polling || !dialog.isConnected) return;
    const id = login.id;
    polling = true;
    try {
      const response = await llmAdminFetch(id, {}, '/admin/llm/oauth/');
      const result = await response.json();
      if (!dialog.isConnected || login?.id !== id) return;
      if (result.status === 'success') await completed(result.account);
      else if (['error', 'expired', 'cancelled'].includes(result.status)) {
        login = null;
        dialog.querySelector('[role=alert]').textContent = result.error || uiText('Login did not complete. Start a new login.', '登录未完成，请重新开始登录。');
        dialog.querySelector('[data-open]').removeAttribute('href');
      }
    } catch (error) { if (dialog.isConnected) dialog.querySelector('[role=alert]').textContent = error.message; }
    finally { polling = false; }
  }, 1500);
  dialog.addEventListener('close', () => {
    clearInterval(timer);
    if (login && llmManagementToken) fetch('/admin/llm/oauth/' + login.id, { method: 'DELETE', headers: { Authorization: 'Bearer ' + llmManagementToken }, keepalive: true }).catch(() => {});
    login = null;
  });
  if (managementEnabled && (localSession || llmManagementToken)) startLogin();
}

async function llmReload() {
  const response = await managementFetch('/debug/llm', { cache: 'no-store', signal: AbortSignal.timeout(8000) });
  if (!response.ok) throw new Error('LLM dashboard returned ' + response.status);
  state.llmData = await response.json();
  delete state.endpointErrors['/debug/llm'];
  renderLlm();
}

function llmFileEditor(account, isNew) {
  const family = llmFamilyForProvider(account.document.type);
  const available = (state.llmData?.backends || []).filter(b => b.mode === 'subscription' && b.family === family);
  const choices = '<option value=""' + (!account.backend ? ' selected' : '') + '>' + uiText('Save account without binding', '暂不绑定，先保存账户') + '</option>' + available.map(b => '<option value="' + esc(b.name) + '"' + (b.name === account.backend ? ' selected' : '') + '>' + esc(b.name) + '</option>').join('') + (account.backend && !available.some(b => b.name === account.backend) ? '<option selected value="' + esc(account.backend) + '">' + esc(account.backend) + ' · ' + uiText('Unavailable', '不可用') + '</option>' : '');
  const content = '<div class="llm-form-pair"><label>' + uiText('Account ID', '账户 ID') + '<input name="id" required pattern="[A-Za-z0-9_-]{1,120}" value="' + esc(account.id || '') + '"' + (isNew ? '' : ' readonly') + '></label><label>' + uiText('Backend binding · optional', '后端绑定 · 可选') + '<select name="backend">' + choices + '</select></label></div><label>OAuth JSON<textarea name="document" rows="16" spellcheck="false" required></textarea></label><p class="muted">' + uiText('Contains credentials. Only explicit download or save sends this document.', '内容包含凭证。仅显式下载或保存会传送此文件。') + '</p>';
  const dialog = llmModal(uiText('OAuth file', 'OAuth 文件'), content, uiText('Save file', '保存文件'), async form => {
    const document = JSON.parse(form.querySelector('[name=document]').value);
    const edited = { ...account, id: form.querySelector('[name=id]').value, backend: form.querySelector('[name=backend]').value, document };
    await llmAdminFetch(encodeURIComponent(edited.id), { method: 'PUT', body: JSON.stringify(edited) });
    await llmReload();
    notify(uiText('OAuth file saved', 'OAuth 文件已保存'));
  });
  dialog.querySelector('textarea').value = JSON.stringify(account.document, null, 2);
  if (!isNew) {
    const tools = document.createElement('div'); tools.className = 'llm-heading-actions';
    tools.innerHTML = '<button type="button" class="quiet-button" data-refresh-token>' + uiText('Refresh OAuth token', '刷新 OAuth token') + '</button><button type="button" class="quiet-button" data-upload-file>' + uiText('Replace OAuth file', '替换 OAuth 文件') + '</button>';
    const refresh = tools.querySelector('[data-refresh-token]');
    refresh.disabled = !account.document.refresh_token;
    refresh.onclick = async () => {
      refresh.disabled = true;
      try { await llmAdminFetch(encodeURIComponent(account.id) + '/refresh', { method: 'POST' }); dialog.close(); await llmReload(); notify(uiText('OAuth token refreshed', 'OAuth token 已刷新')); }
      catch (error) { dialog.querySelector('[role=alert]').textContent = error.message; refresh.disabled = false; }
    };
    tools.querySelector('[data-upload-file]').onclick = () => { dialog.close(); llmChooseFile(account); };
    dialog.querySelector('.llm-dialog-body').appendChild(tools);
  }
}

function llmChooseFile(existing) {
  const subscriptions = (state.llmData?.backends || []).filter(b => b.mode === 'subscription');
  const input = document.createElement('input');
  input.type = 'file'; input.accept = '.json,application/json';
  input.onchange = async () => {
    const file = input.files[0];
    if (!file) return;
    try {
      if (file.size > 1024 * 1024) throw new Error(uiText('OAuth file must be smaller than 1 MiB', 'OAuth 文件不得超过 1 MiB'));
      const document = JSON.parse(await file.text());
      const family = llmFamilyForProvider(document.type);
      const old = existing ? await (await llmAdminFetch(encodeURIComponent(existing.id))).json() : { id: file.name.replace(/\.json$/i, '').replace(/[^A-Za-z0-9_-]/g, '-'), backend: subscriptions.find(b => b.family === family)?.name || '', revision: 0, models: [] };
      llmFileEditor({ ...old, document }, !existing);
    } catch (error) { notify(error.message); }
  };
  input.click();
}

async function llmEditModels(summary) {
  let account = await (await llmAdminFetch(encodeURIComponent(summary.id))).json();
  const isPattern = model => model.includes('*') || model.startsWith('re:') || /^\/.+\/$/.test(model);
  const excluded = new Set(account.models.filter(r => r.disabled && !isPattern(r.model)).map(r => r.model));
  const patterns = account.models.filter(r => r.disabled && isPattern(r.model)).map(r => r.model);
  const exactRules = account.models.filter(r => !patterns.includes(r.model));
  const existingAliasModels = new Set(account.models.filter(r => r.alias).map(r => r.model));
  const known = new Map();
  let catalogLoaded = false;
  const content = '<p class="llm-model-scope"><strong>' + esc(llmProviderInfo[llmFamilyForProvider(account.document.type)].label) + '</strong><span>' + esc(summary.email || summary.id) + '</span></p>'
    + '<p class="muted">' + uiText('Applies to this account. Excluded models cannot be routed through their original names or aliases.', '仅作用于当前账户。禁用后，原模型及其别名都不会路由到此账户。') + '</p>'
    + '<div class="llm-model-sections"><section class="llm-model-section"><header><h3>' + uiText('OAuth model exclusions', 'OAuth 模型禁用') + '</h3><span data-excluded-count></span></header><p class="muted">' + uiText('Select models you do not want to use.', '勾选不想使用的模型。') + '</p><input type="search" data-model-search aria-label="' + uiText('Search models', '搜索模型') + '" placeholder="' + uiText('Search or /regular expression/', '搜索或输入 /正则表达式/') + '"><p class="llm-catalog-status" role="status">' + uiText('Loading current models…', '正在加载当前模型…') + '</p><div class="llm-model-options"></div><label>' + uiText('Exclusion rules', '禁用规则') + '<textarea name="excluded_patterns" rows="2" placeholder="gpt-5-*\nre:^gemini-3\\.">' + esc(patterns.join('\n')) + '</textarea></label><p class="muted">' + uiText('One rule per line. Use * as a wildcard, or re:expression / /expression/ for regular expressions. Matching is case-insensitive.', '每行一条规则。* 是通配符；re:表达式 或 /表达式/ 使用正则，匹配不区分大小写。') + '</p></section>'
    + '<section class="llm-model-section"><header><h3>' + uiText('OAuth model aliases', 'OAuth 模型别名') + '</h3><button class="quiet-button" type="button" data-add-alias>+ ' + uiText('Add alias', '添加别名') + '</button></header><p class="muted">' + uiText('Choose a current provider model, then expose it under an alias.', '选择供应商当前模型，再为它设置别名。') + '</p><div class="llm-model-rules"></div><p class="muted" data-alias-empty>' + uiText('No aliases. Add one to rename a model.', '暂无别名，点击“添加别名”修改模型名称。') + '</p></section></div>';
  const dialog = llmModal(uiText('Model management', '模型管理'), content, uiText('Save rules', '保存规则'), async form => {
    const rules = new Map(exactRules.map(r => [r.model,{...r,disabled:excluded.has(r.model),alias:'',keep_original:true}]));
    const ensure = model => { if (!rules.has(model)) rules.set(model,{model,disabled:false,alias:'',keep_original:true,reasoning_effort:''}); return rules.get(model); };
    for (const model of excluded) ensure(model).disabled = true;
    const sources = new Set();
    for (const row of form.querySelectorAll('.llm-alias-row')) {
      const model = row.querySelector('[name=model]').value.trim(), alias = row.querySelector('[name=alias]').value.trim();
      if (!model || !alias) throw new Error(uiText('Original model and alias are required.', '原模型名称和别名不能为空。'));
      if (catalogLoaded && !known.has(model) && !existingAliasModels.has(model)) throw new Error(uiText('Choose a model from the current provider list.', '请选择供应商当前模型列表中的模型。'));
      if (sources.has(model)) throw new Error(uiText('Each original model can have one alias.', '同一个原模型只能设置一条别名。'));
      sources.add(model);
      Object.assign(ensure(model),{alias,keep_original:row.querySelector('[name=keep_original]').checked});
    }
    const manual = form.querySelector('[name=excluded_patterns]').value.split('\n').map(s => s.trim()).filter(Boolean);
    for (const model of manual) {
      const expression = regexExpression(model);
      if (expression !== null) {
        try { new RegExp(expression,'i'); }
        catch (_) { throw new Error(uiText('Invalid exclusion regular expression: ', '禁用正则表达式无效：') + model); }
      }
      ensure(model).disabled = true;
    }
    const next = {...account,models:[...rules.values()].filter(r => r.disabled || r.alias || r.reasoning_effort)};
    await llmAdminFetch(encodeURIComponent(account.id), { method: 'PUT', body: JSON.stringify(next) });
    await llmReload(); notify(uiText('Model rules saved', '模型规则已保存'));
  });
  dialog.classList.add('llm-model-dialog');
  function regexExpression(value) {
    if (value.startsWith('re:')) return value.slice(3);
    if (value.length > 2 && value.startsWith('/') && value.endsWith('/')) return value.slice(1,-1);
    return null;
  }
  function matchesPattern(pattern, model) {
    const expression = regexExpression(pattern);
    if (expression !== null) {
      try { return new RegExp(expression,'i').test(model); }
      catch (_) { return false; }
    }
    pattern = pattern.toLowerCase(); model = model.toLowerCase();
    let i=0,j=0,star=-1,retry=0;
    while (j<model.length) {
      if (pattern[i] !== '*' && pattern[i] === model[j]) { i++; j++; }
      else if (pattern[i] === '*') { star=i++; retry=j; }
      else if (star>=0) { i=star+1; j=++retry; }
      else return false;
    }
    while (pattern[i] === '*') i++;
    return i === pattern.length;
  }
  function manualRules() { return dialog.querySelector('[name=excluded_patterns]').value.split('\n').map(s=>s.trim()).filter(Boolean); }
  function matchesQuery(query, id, label) {
    if (!query) return true;
    const expression = regexExpression(query);
    if (expression !== null) {
      try { return new RegExp(expression,'i').test(id + ' ' + label); }
      catch (_) { return false; }
    }
    return (id + ' ' + label).toLowerCase().includes(query.toLowerCase());
  }
  function updateCount() {
    const rules = [...excluded,...manualRules()];
    const count = [...known.keys()].filter(id=>rules.some(rule=>matchesPattern(rule,id))).length;
    dialog.querySelector('[data-excluded-count]').textContent = uiText('Excluded ', '已禁用 ') + count + ' / ' + known.size;
  }
  function renderCatalog() {
    const query = dialog.querySelector('[data-model-search]').value.trim();
    const items = [...known].sort(([a],[b]) => a.localeCompare(b));
    dialog.querySelector('.llm-model-options').innerHTML = items.filter(([id,label]) => matchesQuery(query,id,label)).map(([id,label]) => { const byRule=manualRules().some(rule=>matchesPattern(rule,id)); return '<label class="llm-model-option"><input type="checkbox" data-exclude-model="' + esc(id) + '"' + (byRule || excluded.has(id) ? ' checked' : '') + (byRule ? ' disabled title="' + uiText('Excluded by a rule below', '已被下方规则禁用') + '"' : '') + '><span title="' + esc(label) + '">' + esc(id) + '</span><small>' + esc(label === id ? '' : label) + '</small></label>'; }).join('') || '<p class="muted">' + uiText('No current models match.', '当前模型中没有匹配项。') + '</p>';
    dialog.querySelectorAll('[data-exclude-model]').forEach(input => input.onchange = () => { if (input.checked) excluded.add(input.dataset.excludeModel); else excluded.delete(input.dataset.excludeModel); updateCount(); });
    dialog.querySelectorAll('.llm-alias-row').forEach(renderAliasSuggestions);
    updateCount();
  }
  function renderAliasSuggestions(row) {
    const input = row.querySelector('[name=model]'), suggestions = row.querySelector('.llm-model-suggestions');
    if (document.activeElement !== input || !known.size) { suggestions.hidden = true; return; }
    const matches = [...known].filter(([id,label]) => matchesQuery(input.value.trim(),id,label)).slice(0,6);
    suggestions.innerHTML = matches.map(([id,label]) => '<button type="button" data-model="' + esc(id) + '"><strong>' + esc(id) + '</strong>' + (label === id ? '' : '<span>' + esc(label) + '</span>') + '</button>').join('') || '<span class="muted">' + uiText('No current model matches', '当前模型中没有匹配项') + '</span>';
    suggestions.hidden = false;
    suggestions.querySelectorAll('[data-model]').forEach(button => {
      button.onmousedown = event => event.preventDefault();
      button.onclick = () => { input.value = button.dataset.model; suggestions.hidden = true; input.focus(); };
    });
  }
  function addAlias(rule = {}) {
    const row = document.createElement('div'); row.className = 'llm-alias-row';
    row.innerHTML = '<div class="llm-alias-model"><label>' + uiText('Original model', '原模型名称') + '<input name="model" required autocomplete="off" spellcheck="false" value="' + esc(rule.model || '') + '" placeholder="' + uiText('Search current models', '搜索当前模型') + '"></label><div class="llm-model-suggestions" hidden></div></div><span class="llm-alias-arrow" aria-hidden="true">→</span><label>' + uiText('Alias', '别名') + '<input name="alias" required autocomplete="off" spellcheck="false" value="' + esc(rule.alias || '') + '"></label><div class="llm-alias-options"><label class="llm-check"><input name="keep_original" type="checkbox"' + (rule.keep_original ? ' checked' : '') + '>' + uiText('Keep original name', '保留原名') + '</label><button class="quiet-button llm-delete" type="button" data-remove-alias title="' + uiText('Remove alias', '删除别名') + '" aria-label="' + uiText('Remove alias', '删除别名') + '">' + llmIcon('delete') + '</button></div>';
    const modelInput = row.querySelector('[name=model]');
    modelInput.onfocus = () => renderAliasSuggestions(row);
    modelInput.oninput = () => renderAliasSuggestions(row);
    modelInput.onblur = () => setTimeout(() => { if (row.isConnected) row.querySelector('.llm-model-suggestions').hidden = true; },100);
    row.querySelector('[data-remove-alias]').onclick = () => { row.remove(); dialog.querySelector('[data-alias-empty]').hidden = !!dialog.querySelector('.llm-alias-row'); };
    dialog.querySelector('.llm-model-rules').appendChild(row);
    dialog.querySelector('[data-alias-empty]').hidden = true;
  }
  account.models.filter(r => r.alias).forEach(addAlias);
  dialog.querySelector('[data-add-alias]').onclick = () => { addAlias(); dialog.querySelector('.llm-alias-row:last-child [name=model]').focus(); };
  dialog.querySelector('[data-model-search]').oninput = renderCatalog;
  dialog.querySelector('[name=excluded_patterns]').oninput = renderCatalog;
  renderCatalog();
  async function loadCatalog() {
    try {
      const catalog = await (await llmAdminFetch(encodeURIComponent(account.id) + '/models')).json();
      if (!dialog.isConnected) return;
      if (catalog.revision !== account.revision) {
        const latest = await (await llmAdminFetch(encodeURIComponent(account.id))).json();
        if (JSON.stringify(latest.models) === JSON.stringify(account.models) && latest.backend === account.backend) account = latest;
      }
      known.clear();
      for (const model of catalog.models) known.set(model.id,model.display_name || model.id);
      catalogLoaded = true;
      renderCatalog();
      dialog.querySelector('.llm-catalog-status').textContent = uiText('Current provider models · ', '供应商当前模型 · ') + catalog.models.length;
    } catch (error) {
      if (dialog.isConnected) dialog.querySelector('.llm-catalog-status').textContent = uiText('Model list unavailable; manual rules still work. ', '模型列表加载失败，仍可手动填写规则。') + error.message;
    }
  }
  void loadCatalog();
}

async function llmAccountAction(action, account, button) {
  if (action === 'upload') { llmChooseFile(account); return; }
  if (action === 'view-credits') { llmCreditDialog(account); return; }
  button.disabled = true;
  try {
    const id = encodeURIComponent(account.id);
    if (action === 'quota') { await llmRefreshQuota(account.id, true); }
    else if (action === 'reset') { llmResetQuota(account, button.dataset.creditId); }
    else if (action === 'delete') { llmDeleteAccounts([account]); }
    else if (action === 'toggle') {
      await llmAdminFetch(id + '/status', { method: 'PATCH', body: JSON.stringify({ revision: account.revision, disabled: !account.disabled }) });
      await llmReload();
    } else if (action === 'refresh') {
      await llmAdminFetch(id + '/refresh', { method: 'POST' }); await llmReload();
      notify(uiText('OAuth token refreshed', 'OAuth token 已刷新'));
    } else if (action === 'edit') llmFileEditor(await (await llmAdminFetch(id)).json(), false);
    else if (action === 'models') await llmEditModels(account);
    else if (action === 'download') {
      const blob = await (await llmAdminFetch(id + '/download')).blob();
      const url = URL.createObjectURL(blob), link = document.createElement('a');
      link.href = url; link.download = account.id + '.json'; link.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    }
  } catch (error) { if (action === 'toggle') button.checked = !account.disabled; notify(error.message); }
  finally { button.disabled = false; }
}

function llmProviderTabs(data, mode, family, actions = '') {
  const source = mode === 'subscription' ? data.accounts : data.backends.filter(backend => backend.mode === mode);
  const counts = Object.fromEntries(Object.keys(llmProviderInfo).map(id => [id, 0]));
  for (const item of source) {
    const id = mode === 'subscription' ? llmFamilyForProvider(item.provider) : item.family;
    if (id in counts) counts[id]++;
  }
  const tabs = [['all', uiText('All', '全部')], ...Object.entries(llmProviderInfo).map(([id, info]) => [id, info.label])];
  return '<nav class="llm-provider-tabs" id="llm-provider-tabs" aria-label="' + uiText('Provider filter', '供应商筛选') + '">'
    + '<div class="llm-provider-tab-list">'
    + tabs.map(([id, label]) => {
      const active = family === id;
      const count = id === 'all' ? source.length : counts[id];
      const content = id === 'all'
        ? '<span>' + esc(label) + '</span>'
        : '<span class="llm-provider-mark">' + llmProviderMark(id) + '</span>';
      return '<button type="button" class="llm-provider-tab' + (active ? ' active' : '') + '" data-family="' + id + '" title="' + esc(label) + '" aria-label="' + esc(label) + '" aria-pressed="' + active + '">'
        + content + '<span class="llm-provider-count">' + count + '</span></button>';
    }).join('')
    + '</div>'
    + actions
    + '</nav>';
}

const llmSelectedAccounts = new Set();
const llmQuotaJobs = new Map();
let llmQuotaWorkers = 0;

function llmDate(value) {
  if (!value) return '—';
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return '—';
  const locale = uiText('en-US', 'zh-CN');
  const absolute = new Intl.DateTimeFormat(locale, { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', hour12: false }).format(date);
  const seconds = (date.getTime() - Date.now()) / 1000;
  const [unit, divisor] = Math.abs(seconds) >= 86400 ? ['day',86400] : Math.abs(seconds) >= 3600 ? ['hour',3600] : ['minute',60];
  return absolute + ' · ' + new Intl.RelativeTimeFormat(locale, { numeric: 'auto' }).format(Math.round(seconds / divisor),unit);
}

async function llmRefreshQuota(id, explicit = false) {
  const prior = llmQuotaJobs.get(id);
  if (prior?.loading) return;
  llmQuotaJobs.set(id, { loading: true, retryAt: Date.now() + 60000 });
  try {
    await llmAdminFetch(encodeURIComponent(id) + '/quota', { method: 'POST' });
  } catch (error) {
    if (explicit) notify(error.message);
  } finally {
    llmQuotaJobs.set(id, { loading: false, retryAt: Date.now() + 60000 });
    await llmReload();
  }
}

function llmLoadQuotas(accounts, data) {
  if (!data.management_enabled || (data.management_mode !== 'local' && !llmManagementToken)) return;
  for (const account of accounts) {
    const job = llmQuotaJobs.get(account.id);
    const observed = Date.parse(account.quota?.observed_at || '') || 0;
    if (llmQuotaWorkers >= 2) break;
    if (account.disabled || job?.loading || Date.now() < (job?.retryAt || 0) || Date.now() - observed < 60000) continue;
    llmQuotaWorkers++;
    llmRefreshQuota(account.id).catch(error => notify(error.message)).finally(() => { llmQuotaWorkers--; });
  }
}

function llmDeleteAccounts(accounts) {
  llmModal(uiText('Delete OAuth accounts', '删除 OAuth 账户'),
    '<p>' + uiText('Delete these saved credentials? This cannot be undone. Already running requests may finish.', '删除以下账户的已保存凭证？此操作无法撤销，已开始的请求可能继续完成。') + '</p><ul>' + accounts.map(a => '<li>' + esc(a.email || a.id) + '</li>').join('') + '</ul>',
    uiText('Delete', '删除'), async form => {
      const failures = [];
      for (const account of accounts) {
        try { await llmAdminFetch(encodeURIComponent(account.id), { method: 'DELETE', body: JSON.stringify({ revision: account.revision }) }); llmSelectedAccounts.delete(account.id); }
        catch (error) { failures.push(account.id + ': ' + error.message); }
      }
      await llmReload();
      if (failures.length) { form.querySelector('[type=submit]').hidden = true; throw new Error(failures.join('\n')); }
    });
}

function llmResetQuota(account, creditId) {
  const credit = account.quota?.reset_credits?.credits?.find(c => c.id === creditId);
  if (!creditId) return;
  const key = 'transit-quota-reset:' + account.id + ':' + creditId;
  llmModal(uiText('Use reset credit', '使用重置卡'),
    '<p>' + esc(account.email || account.id) + '</p><p>' + uiText('Use the selected reset credit? Expires: ', '使用选中的这张重置卡？过期时间：') + esc(llmDate(credit?.expires_at)) + '</p>',
    uiText('Use reset', '使用重置'), async () => {
      let requestId = sessionStorage.getItem(key);
      if (!requestId) { requestId = crypto.randomUUID(); sessionStorage.setItem(key, requestId); }
      const result = await (await llmAdminFetch(encodeURIComponent(account.id) + '/reset-quota', { method: 'POST', body: JSON.stringify({ redeem_request_id: requestId, credit_id: creditId }) })).json();
      sessionStorage.removeItem(key);
      llmQuotaJobs.delete(account.id);
      await llmReload();
      const message = {
        reset: uiText('Quota reset completed', '额度已重置'),
        already_redeemed: uiText('This request was already redeemed', '这次请求已兑换过'),
        nothing_to_reset: uiText('No quota needs resetting; the credit was not used', '当前无需重置，重置卡未消耗'),
        no_credit: uiText('This reset credit is no longer available', '这张重置卡已不可用')
      }[result.code] || uiText('Reset result could not be confirmed', '无法确认重置结果');
      notify(message + (result.refresh_error ? uiText('; quota refresh failed. Refresh to check.', '；额度刷新失败，请刷新查看。') : ''));
    });
}

function llmCreditDialog(account) {
  const q = account.quota;
  const credits = q?.reset_credits;
  const loading = llmQuotaJobs.get(account.id)?.loading;
  const timezone = new Intl.DateTimeFormat('en', { timeZoneName: 'shortOffset' }).formatToParts(new Date()).find(p => p.type === 'timeZoneName')?.value || '';
  const expirations = (credits?.credits || []).slice().sort((a,b) => (Date.parse(a.expires_at) || Infinity) - (Date.parse(b.expires_at) || Infinity));
  const pendingCredit = account.pending_reset_credit_id;
  if (pendingCredit && !expirations.some(c => c.id === pendingCredit)) expirations.push({ id: pendingCredit });
  const creditRows = expirations.map((credit,i) => {
    const expired = credit.expires_at && Date.parse(credit.expires_at) <= Date.now();
    const pending = credit.id === pendingCredit;
    const disabled = !credit.id || (pendingCredit && !pending) || (!pending && (expired || loading || account.quota_error || q?.credits_error));
    return '<div class="llm-credit-row"><div><span>' + uiText('Reset ', '第 ') + (i+1) + uiText('', ' 次') + '</span>' + llmQuotaTime(credit.expires_at) + '</div><button type="button" class="quiet-button" data-action="reset" data-credit-id="' + esc(credit.id || '') + '" aria-label="' + esc(uiText('Use reset ', '使用第 ') + (i+1) + uiText('', ' 次重置')) + '"' + (disabled ? ' disabled' : '') + '>' + (pending ? uiText('Retry', '重试确认') : uiText('Use reset', '使用重置')) + '</button></div>';
  }).join('');
  const availableCount = credits?.available_count ?? expirations.length;
  const content = '<div class="llm-credit-dialog-content">'
    + '<header class="llm-credit-header"><strong>' + uiText('Reset credit expiration', '到期时间') + (timezone ? ' (' + esc(timezone) + ')' : '') + '</strong><span>' + uiText('Available: ', '可用：') + '<b>' + esc(availableCount) + '</b></span></header>'
    + (creditRows ? '<div class="llm-credit-list">' + creditRows + '</div>' : '<p class="muted">' + uiText('No reset credits available', '暂无可用的重置卡') + '</p>')
    + ([account.quota_error,q?.credits_error,q?.profile_error].filter(Boolean).map(message => '<p class="llm-quota-error" role="status">' + esc(message) + (q?.observed_at ? ' · ' + uiText('Last observation retained', '保留上次观测') : '') + '</p>').join(''))
    + '</div>';
  const dialog = llmModal(uiText('Reset credits', '重置卡'), content, '', () => {});
  dialog.classList.add('llm-credit-dialog');
  dialog.querySelectorAll('[data-action=reset]').forEach(button => {
    button.onclick = () => {
      dialog.close();
      llmResetQuota(account, button.dataset.creditId);
    };
  });
  return dialog;
}

function llmQuotaTime(value) {
  const [absolute, relative] = llmDate(value).split(' · ');
  const soon = Date.parse(value) - Date.now();
  return '<time title="' + esc(value || '') + '">' + esc(absolute) + (relative ? ' · <span' + (soon > 0 && soon < 3600000 ? ' class="llm-time-soon"' : '') + '>' + esc(relative) + '</span>' : '') + '</time>';
}

function llmQuotaWindowMarkup(w) {
  const seconds = Number(w.window_seconds);
  const period = !(seconds > 0) ? '' : seconds === 18000 ? uiText('5-hour limit', '5 小时限额') : seconds === 604800 ? uiText('Weekly limit', '周限额') : seconds >= 2419200 && seconds <= 2678400 ? uiText('Monthly limit', '月限额') : uiText(Math.round(seconds / 3600) + '-hour limit', Math.round(seconds / 3600) + ' 小时限额');
  const prefix = ({ five_hour: '', seven_day: '', seven_day_opus: 'Opus', seven_day_sonnet: 'Sonnet', iguana_necktie: 'Fable', 'code-review': uiText('Code review', '代码审查') })[w.name] ?? w.name;
  const remaining = typeof w.used_percent === 'number' ? Math.max(0,Math.min(100,100-w.used_percent)) : null;
  const label = w.window_label || [prefix,period].filter(Boolean).join(' ');
  return '<div class="quota-window-item"><div class="quota-window-label-row"><strong>' + esc(label) + '</strong><span><b title="' + uiText('Remaining quota', '剩余额度') + '">' + (remaining === null ? '—' : Math.round(remaining) + '%') + '</b> ' + llmQuotaTime(w.reset_at) + '</span></div><div class="llm-quota-track"' + (remaining === null ? ' aria-label="' + uiText('Quota not reported', '额度未返回') + '"' : ' role="progressbar" aria-label="' + esc(label) + '" aria-valuemin="0" aria-valuemax="100" aria-valuenow="' + remaining + '"') + '><span style="width:' + (remaining ?? 0) + '%" class="' + (remaining !== null && remaining <= 20 ? 'low' : '') + '"></span></div></div>';
}

function llmPlanMarkup(account) {
  const q = account.quota;
  const rawPlan = q?.plan_type || account.plan_type;
  const renewal = q?.renewal_at || account.renewal_at;
  const credits = q?.reset_credits;
  const parts = [];
  if (rawPlan) parts.push('<span>' + uiText('Plan: ', '套餐：') + '<b>' + esc(rawPlan[0].toUpperCase() + rawPlan.slice(1)) + '</b></span>');
  if (renewal) parts.push('<span>' + uiText('Renewal ', '续费时间 ') + llmQuotaTime(renewal) + '</span>');
  if (account.provider === 'codex' && !(credits?.credits?.length || account.pending_reset_credit_id)) {
    parts.push('<span>' + uiText('Reset credits ', '重置卡 ') + '<b>' + esc(credits?.available_count ?? '0') + '</b>' + uiText('', ' 次') + '</span>');
  }
  return parts.length ? '<div class="llm-plan-pills">' + parts.join('') + '</div>' : '';
}

function llmQuotaMarkup(account) {
  const q = account.quota;
  const credits = q?.reset_credits;
  const loading = llmQuotaJobs.get(account.id)?.loading;
  const summaryGroups = new Map();
  const modelWindows = [];
  for (const window of q?.windows || []) {
    if (window.window !== 'summary') {
      if (account.provider !== 'antigravity') modelWindows.push(window);
      continue;
    }
    const name = window.name || uiText('Antigravity', 'Antigravity');
    let group = summaryGroups.get(name);
    if (!group) {
      group = { name, description: window.group_description || '', windows: [] };
      summaryGroups.set(name, group);
    } else if (!group.description && window.group_description) {
      group.description = window.group_description;
    }
    group.windows.push(window);
  }
  const summary = summaryGroups.size ? '<div class="llm-quota-summary-groups">' + [...summaryGroups.values()].map(group => '<section class="llm-quota-group"><header><strong>' + esc(group.name) + '</strong>' + (group.description ? '<span>' + esc(group.description) + '</span>' : '') + '</header><div class="llm-quota-group-windows">' + group.windows.map(llmQuotaWindowMarkup).join('') + '</div></section>').join('') + '</div>' : '';
  const windows = summary + modelWindows.map(llmQuotaWindowMarkup).join('');
  const timezone = new Intl.DateTimeFormat('en', { timeZoneName: 'shortOffset' }).formatToParts(new Date()).find(p => p.type === 'timeZoneName')?.value || '';
  const expirations = (credits?.credits || []).slice().sort((a,b) => (Date.parse(a.expires_at) || Infinity) - (Date.parse(b.expires_at) || Infinity));
  const pendingCredit = account.pending_reset_credit_id;
  if (pendingCredit && !expirations.some(c => c.id === pendingCredit)) expirations.push({ id: pendingCredit });
  const availableCount = credits?.available_count ?? expirations.length;
  const hasCredits = expirations.length > 0 || Boolean(pendingCredit);
  const creditBadge = hasCredits ? '<div class="llm-credit-expirations">'
    + '<button type="button" class="quiet-button llm-credit-badge' + (pendingCredit ? ' has-pending' : '') + '" data-action="view-credits" aria-label="' + esc(uiText('Reset credits ', '重置卡 ') + availableCount) + '">'
    + '<span class="llm-credit-label">' + uiText('Reset credits ', '重置卡 ') + '</span><b class="llm-credit-count">' + esc(availableCount) + '</b></button>'
    + '</div>' : '';
  return '<section class="llm-quota-panel"><div class="llm-quota-windows">' + (windows || '<p class="muted">' + (loading ? uiText('Loading provider quota…', '正在查询供应商额度…') : uiText('Quota not reported. Refresh to query the provider.', '尚无额度数据，请刷新查询供应商。')) + '</p>') + '</div>'
    + creditBadge
    + ([account.quota_error,q?.credits_error,q?.profile_error].filter(Boolean).map(message => '<p class="llm-quota-error" role="status">' + esc(message) + (q?.observed_at ? ' · ' + uiText('Last observation retained', '保留上次观测') : '') + '</p>').join('')) + '</section>';
}

function llmSubscriptionCard(account) {
  const card = document.createElement('article'); card.className = 'llm-account-card llm-subscription-card'; card.dataset.accountId = account.id;
  let hash = 2166136261;
  for (const character of account.id) hash = Math.imul(hash ^ character.charCodeAt(0), 16777619);
  const providerName = account.provider === 'codex' ? 'OpenAI' : account.provider;
  const name = [providerName, account.email || uiText('Unknown email', '未知邮箱'), (hash >>> 0).toString(16).padStart(8,'0').slice(0,7)].join('-');
  card.dataset.search = [account.id, account.backend, account.email, account.provider, providerName, name].filter(Boolean).join(' ');
  card.innerHTML = '<header class="llm-identity"><div class="llm-identity-main"><span class="llm-provider-mark">' + llmProviderMark(llmFamilyForProvider(account.provider)) + '</span><div class="llm-identity-meta"><h3 title="' + esc(name) + '">' + esc(name) + '</h3>' + llmPlanMarkup(account) + '</div></div></header>' + llmQuotaMarkup(account)
    + '<div class="llm-account-actions" role="group" aria-label="' + esc(uiText('Account actions', '账户操作')) + '"></div><label class="llm-account-toggle"><span>' + uiText('Enabled', '启用') + '</span><input type="checkbox" role="switch" data-action="toggle" aria-label="' + esc(uiText('Enable ', '启用 ') + (account.email || account.id)) + '"' + (account.disabled ? '' : ' checked') + '></label><input type="checkbox" data-select aria-label="' + esc(uiText('Select ', '选择 ') + (account.email || account.id)) + '"' + (llmSelectedAccounts.has(account.id) ? ' checked' : '') + '>';
  const labels = { models: uiText('Models', '模型'), download: uiText('Download OAuth file', '下载 OAuth 文件'), edit: uiText('Account settings', '账户设置'), delete: uiText('Delete account', '删除账户') };
  labels.quota = uiText('Refresh quota', '刷新额度');
  for (const [action,label] of Object.entries(labels)) {
    const button = document.createElement('button'); button.type = 'button'; button.className = 'quiet-button llm-icon-button' + (action === 'delete' ? ' llm-delete' : '');
    button.title = button.ariaLabel = label; button.dataset.action = action;
    button.innerHTML = llmIcon(action === 'quota' ? 'refresh' : action === 'edit' ? 'settings' : action);
    button.disabled = action === 'quota' && !!llmQuotaJobs.get(account.id)?.loading;
    card.querySelector('.llm-account-actions').appendChild(button);
  }
  card.querySelectorAll('[data-action]').forEach(button => button.onclick = () => llmAccountAction(button.dataset.action,account,button));
  card.querySelector('[data-select]').onchange = event => { if (event.target.checked) llmSelectedAccounts.add(account.id); else llmSelectedAccounts.delete(account.id); renderLlm(); };
  return card;
}

function llmPlatformRow(account, providers) {
  const b = account.detail;
  const row = document.createElement('article');
  row.className = 'llm-platform-row';
  row.dataset.accountId = account.id;
  const provider = providers.get(b.provider) || { name: b.provider, family: b.family, authentication: 'unset' };
  const models = (b.models || []).join(', ') || '*';
  row.innerHTML = '<div class="llm-platform-cell llm-platform-name"><strong>' + esc(b.name) + '</strong><small>' + esc(models) + '</small></div>'
    + '<div class="llm-platform-cell"><span class="llm-platform-provider"><span class="llm-provider-mark">' + llmProviderMark(provider.family) + '</span><strong>' + esc(provider.name) + '</strong><small>' + esc(provider.authentication === 'api-key' ? uiText('API key', 'API Key') : uiText('Unset', '未设置')) + '</small></span></div>'
    + '<div class="llm-platform-cell"><span class="llm-platform-outgoing">' + uiText('Incoming model', '传入模型') + '</span></div>'
    + '<div class="llm-platform-cell"><span class="type-badge">' + uiText('None', '无') + '</span></div>';
  return row;
}

function llmCreateProvider() {
  const defaults = { chatgpt: 'openai', anthropic: 'anthropic', antigravity: 'antigravity' };
  const options = Object.entries(llmProviderInfo).map(([id, info]) => '<option value="' + id + '">' + esc(info.label) + '</option>').join('');
  const content = '<label>' + uiText('Provider name', 'Provider 名称') + '<input name="name" required pattern="[A-Za-z0-9._-]{1,80}" maxlength="80" spellcheck="false"></label>'
    + '<div class="llm-form-pair"><label>' + uiText('Provider', '供应商') + '<select name="family">' + options + '</select></label><label>' + uiText('Base URL · optional', 'Base URL · 可选') + '<input name="base_url" type="url" placeholder="https://…" spellcheck="false"></label></div>'
    + '<fieldset class="llm-auth-mode"><legend>' + uiText('Provider API key', 'Provider API Key') + '</legend><label><input type="radio" name="authentication" value="unset" checked>' + uiText('Unset', '未设置') + '</label><label><input type="radio" name="authentication" value="api-key">' + uiText('API key', 'API Key') + '</label></fieldset>'
    + '<label data-api-key hidden>' + uiText('API key', 'API Key') + '<div class="llm-secret-input"><input name="api_key" type="password" autocomplete="new-password" spellcheck="false"><button class="quiet-button" type="button" data-reveal-key aria-label="' + esc(uiText('Show API key', '显示 API Key')) + '">' + llmIcon('edit') + '</button></div></label><p class="muted">' + uiText('The key is stored separately from the runtime configuration with local-owner-only permissions.', '密钥与运行配置分开保存，并限制为本地所有者可读。') + '</p>';
  const dialog = llmModal(uiText('Create provider', '创建 Provider'), content, uiText('Create provider', '创建 Provider'), async form => {
    const family = form.querySelector('[name=family]').value;
    const name = form.querySelector('[name=name]').value.trim();
    await llmAdminFetch('platform/providers', { method: 'POST', body: JSON.stringify({
      expected_version: state.llmData?.config_version || '', family, name,
      authentication: form.querySelector('[name=authentication]:checked').value,
      api_key: form.querySelector('[name=api_key]').value,
      base_url: form.querySelector('[name=base_url]').value.trim()
    }) }, '/admin/llm/');
    for (let attempt = 0; attempt < 6; attempt++) {
      await new Promise(resolve => setTimeout(resolve, 200));
      await llmReload();
      if ((state.llmData.providers || []).some(provider => provider.name === name)) break;
    }
    state.llmMode = 'api'; state.llmFamily = family; renderLlm();
    notify(uiText('Provider created', 'Provider 已创建'));
  });
  const family = dialog.querySelector('[name=family]'), name = dialog.querySelector('[name=name]');
  const applyDefaults = () => { name.value = defaults[family.value]; };
  family.onchange = applyDefaults;
  dialog.querySelectorAll('[name=authentication]').forEach(input => input.onchange = () => { dialog.querySelector('[data-api-key]').hidden = dialog.querySelector('[name=authentication]:checked').value !== 'api-key'; });
  dialog.querySelector('[data-reveal-key]').onclick = () => { const input = dialog.querySelector('[name=api_key]'); input.type = input.type === 'password' ? 'text' : 'password'; };
  applyDefaults();
}

function llmAddModel() {
  const providers = state.llmData?.providers || [];
  const choices = providers.map(provider => '<option value="' + esc(provider.name) + '">' + esc(provider.name) + ' · ' + esc(llmProviderInfo[provider.family]?.label || provider.family) + '</option>').join('');
  const content = '<label>' + uiText('Incoming model match', '传入模型匹配') + '<input name="model" required maxlength="200" value="*" spellcheck="false"></label><p class="muted">' + uiText('Use * to accept every model, or enter one exact model ID.', '使用 * 匹配全部模型，或输入一个精确模型 ID。') + '</p><label>' + uiText('Provider', 'Provider') + '<select name="provider" required>' + choices + '</select></label>';
  llmModal(uiText('Add model', '添加模型'), content, uiText('Add model', '添加模型'), async form => {
    if (!providers.length) throw new Error(uiText('Create a provider before adding a model.', '请先创建 Provider，再添加模型。'));
    const provider = form.querySelector('[name=provider]').value;
    const model = form.querySelector('[name=model]').value.trim();
    await llmAdminFetch('platform/models', { method: 'POST', body: JSON.stringify({ expected_version: state.llmData?.config_version || '', provider, model }) }, '/admin/llm/');
    for (let attempt = 0; attempt < 6; attempt++) {
      await new Promise(resolve => setTimeout(resolve, 200));
      await llmReload();
      if (state.llmData.backends.some(backend => backend.name === provider + '/' + model)) break;
    }
    state.llmMode = 'api'; renderLlm();
    notify(uiText('Model added', '模型已添加'));
  });
}

function renderLlmWorkspace() {
  const host = $('tab-llm');
  const data = state.llmData || { accounts: [], providers: [], backends: [] };
  if (state.llmMode === 'local') state.llmMode = 'subscription';
  const mode = state.llmMode || 'subscription', family = state.llmFamily || 'all';
  const modes = [['subscription', uiText('Account subscription', '账户订阅')], ['api', uiText('Platform', '平台')]];
  const selected = data.backends.filter(b => b.mode === mode && (family === 'all' || b.family === family));
  const error = state.endpointErrors['/debug/llm'];
  const subscriptionActions = mode === 'subscription' ? '<div class="llm-heading-actions">'
    + (family === 'all' ? '' : '<button id="llm-login" type="button" class="quiet-button llm-icon-button" title="' + uiText('OAuth login', 'OAuth 登录') + '" aria-label="' + uiText('OAuth login', 'OAuth 登录') + '">' + llmIcon('login') + '</button>')
    + '<button id="llm-import" type="button" class="quiet-button llm-icon-button" title="' + uiText('Upload OAuth file', '上传 OAuth 文件') + '" aria-label="' + uiText('Upload OAuth file', '上传 OAuth 文件') + '">' + llmIcon('upload') + '</button></div>' : '';
  const platformActions = mode === 'api' ? '<div class="llm-heading-actions"><button id="llm-add-model" type="button" class="quiet-button llm-primary"' + (data.platform_writable && (data.providers || []).length ? '' : ' disabled') + '>' + uiText('Add model', '添加模型') + '</button><button id="llm-create-provider" type="button" class="quiet-button" title="' + esc(data.platform_writable ? uiText('Create Providers', '创建 Provider') : uiText('A writable local runtime configuration is required', '需要可写的本地运行配置')) + '"' + (data.platform_writable ? '' : ' disabled') + '>' + uiText('Create Providers', '创建 Provider') + '</button></div>' : '';
  const platformHeader = mode === 'api' ? '<div class="llm-platform-table-head"><span>' + uiText('Name', '名称') + '</span><span>' + uiText('Provider', 'Provider') + '</span><span>' + uiText('Outgoing model', '输出模型') + '</span><span>' + uiText('Policy state', '策略状态') + '</span></div>' : '';
  host.innerHTML = '<div class="llm-workspace"><div class="llm-workspace-toolbar"><div class="llm-mode-tabs" role="group" aria-label="' + uiText('LLM mode', 'LLM 模式') + '">' + modes.map(([id, label]) => '<button type="button" data-mode="' + id + '" aria-pressed="' + (mode === id) + '">' + label + '</button>').join('') + '</div></div>'
    + llmProviderTabs(data, mode, family, subscriptionActions + platformActions)
    + (error ? '<div class="data-notice" role="alert">' + esc(error) + ' · ' + uiText('Displayed data may be stale.', '当前显示的数据可能已过期。') + '</div>' : '')
    + platformHeader + '<div class="llm-accounts-grid' + (mode === 'api' ? ' llm-platform-list' : '') + '" id="llm-accounts-grid"></div></div>';
  const actions = $('page-actions');
  if (actions) actions.innerHTML = '';
  host.querySelectorAll('[data-mode]').forEach(button => button.onclick = () => { state.llmMode = button.dataset.mode; renderLlm(); });
  host.querySelectorAll('.llm-provider-tab').forEach(button => button.onclick = () => { state.llmFamily = button.dataset.family; renderLlm(); });
  if ($('llm-import')) $('llm-import').onclick = () => llmChooseFile();
  if ($('llm-login')) $('llm-login').onclick = () => llmLoginDialog(family);
  if ($('llm-create-provider')) $('llm-create-provider').onclick = () => llmCreateProvider();
  if ($('llm-add-model')) $('llm-add-model').onclick = () => llmAddModel();
  const query = (state.queries?.llm || state.query || '').toLowerCase();
  const accounts = mode === 'subscription' ? data.accounts.filter(a => family === 'all' || llmFamilyForProvider(a.provider) === family).map(a => ({ ...a, detail: data.backends.find(b => b.name === a.backend) || { name: uiText('Unbound account', '未绑定账户'), models: (a.models || []).map(rule => rule.model), provider: a.provider } })) : selected.map(b => ({ id: b.name, backend: b.name, detail: b }));
  const visible = accounts.filter(a => JSON.stringify([a.id, a.email, a.backend, a.detail.models]).toLowerCase().includes(query));
  const grid = $('llm-accounts-grid');
  const providers = new Map((data.providers || []).map(provider => [provider.name, provider]));
  for (const id of llmSelectedAccounts) if (!data.accounts.some(a => a.id === id)) llmSelectedAccounts.delete(id);
  if (mode === 'subscription') {
    const checked = visible.filter(a => llmSelectedAccounts.has(a.id));
    if (checked.length) {
      const bulk = document.createElement('div'); bulk.className = 'llm-bulk-actions';
      bulk.innerHTML = '<span>' + uiText('Selected ', '已选择 ') + checked.length + '</span><button type="button" class="quiet-button" data-bulk="enable">' + uiText('Enable', '启用') + '</button><button type="button" class="quiet-button" data-bulk="disable">' + uiText('Disable', '禁用') + '</button><button type="button" class="quiet-button llm-delete" data-bulk="delete">' + uiText('Delete', '删除') + '</button>';
      bulk.querySelectorAll('[data-bulk]').forEach(button => button.onclick = async () => {
        if (button.dataset.bulk === 'delete') { llmDeleteAccounts(checked); return; }
        button.disabled = true; const failures = [];
        for (const a of checked) {
          try { await llmAdminFetch(encodeURIComponent(a.id) + '/status', { method:'PATCH', body:JSON.stringify({ revision:a.revision, disabled:button.dataset.bulk === 'disable' }) }); }
          catch (error) { failures.push(a.id + ': ' + error.message); }
        }
        await llmReload(); if (failures.length) notify(failures.join('\n'));
      });
      grid.before(bulk);
    }
  }
  for (const account of visible) {
    if (mode === 'subscription') { grid.appendChild(llmSubscriptionCard(account)); continue; }
    if (mode === 'api') { grid.appendChild(llmPlatformRow(account, providers)); continue; }
    const b = account.detail, card = document.createElement('article'); card.className = 'llm-account-card';
    const quota = (b.quota?.windows || []).map(w => '<div class="quota-window-item"><div class="quota-window-label-row"><span>' + esc(w.name + ' · ' + w.window) + '</span><span>' + esc(w.used_percent) + '% ' + uiText('used', '已用') + '</span></div><progress max="100" value="' + Math.max(0, Math.min(100, Number(w.used_percent) || 0)) + '"></progress><small>' + esc(w.reset_at || '') + '</small></div>').join('');
    card.innerHTML = '<div class="llm-account-header"><div class="llm-account-header-top"><strong>' + esc(account.id) + '</strong><span class="type-badge">' + esc(mode === 'subscription' ? 'OAuth' : b.provider) + '</span></div><div class="llm-account-header-sub">' + esc(account.email || b.endpoint || b.name) + '</div></div><div class="llm-account-body"><div class="llm-stat-row"><span>' + uiText('Backend', '后端') + '</span><span class="code">' + esc(account.backend || uiText('Not bound', '未绑定')) + '</span></div><div class="llm-stat-row"><span>' + uiText('Models', '模型') + '</span><span class="llm-stat-v">' + esc((b.models || []).join(', ') || uiText('Unrestricted', '未限定')) + '</span></div>' + (mode === 'subscription' ? '<div class="llm-stat-row"><span>' + uiText('Token expires', 'Token 到期') + '</span><span class="llm-stat-v">' + esc(account.expires_at || '—') + '</span></div>' + (quota || '<p class="muted">' + uiText('Quota not reported', '尚无额度观测') + '</p>') : '') + '</div><div class="llm-account-footer"><span class="muted">' + (mode === 'subscription' ? uiText('File revision ', '文件版本 ') + account.revision : uiText('Gateway configuration', '网关配置')) + '</span><div class="llm-account-actions"></div></div>';
    if (mode === 'subscription') {
      const labels = { refresh: uiText('Refresh OAuth token', '刷新 OAuth token'), edit: uiText('Edit OAuth file', '编辑 OAuth 文件'), upload: uiText('Upload OAuth file', '上传 OAuth 文件'), download: uiText('Download OAuth file', '下载 OAuth 文件'), models: uiText('Edit models', '编辑模型') };
      for (const action of Object.keys(labels)) {
        const button = document.createElement('button'); button.type = 'button'; button.className = 'quiet-button llm-icon-button';
        button.title = button.ariaLabel = labels[action]; button.dataset.action = action; button.innerHTML = llmIcon(action);
        button.disabled = action === 'refresh' && !account.can_refresh;
        button.onclick = () => llmAccountAction(action, account, button); card.querySelector('.llm-account-actions').appendChild(button);
      }
    }
    grid.appendChild(card);
  }
  if (!visible.length) grid.innerHTML = '<div class="ui-empty">' + uiText('No available information', '暂无可用信息') + '</div>';
  if (mode === 'subscription') llmLoadQuotas(visible, data);
}
