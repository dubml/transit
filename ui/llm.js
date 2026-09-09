/* OAuth account workspace. Loaded before the shared UI bootstraps. */
let llmManagementToken = '';

function llmIcon(name) {
  const paths = {
    login: '<path d="M14 4h6v16h-6M3 12h12m-4-4 4 4-4 4"/>',
    refresh: '<path d="M20 7v5h-5M4 17v-5h5M6.1 7a7 7 0 0 1 11.5-2L20 8M4 16l2.4 3A7 7 0 0 0 18 17"/>',
    edit: '<path d="m15 4 5 5M4 20l5-1L20 8a2 2 0 0 0-5-5L4 14z"/>',
    upload: '<path d="M12 16V3m-5 5 5-5 5 5M4 16v5h16v-5"/>',
    download: '<path d="M12 3v13m-5-5 5 5 5-5M4 16v5h16v-5"/>',
    models: '<path d="M4 6h16M4 12h16M4 18h16"/><circle cx="8" cy="6" r="2"/><circle cx="16" cy="12" r="2"/><circle cx="10" cy="18" r="2"/>'
  };
  return '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' + paths[name] + '</svg>';
}

function llmModal(title, content, saveLabel, onSave) {
  const previous = document.activeElement;
  const dialog = document.createElement('dialog');
  dialog.className = 'llm-dialog';
  dialog.innerHTML = '<form><header><h2>' + esc(title) + '</h2><button type="button" class="quiet-button" data-close aria-label="' + esc(uiText('Close', '关闭')) + '">×</button></header><div class="llm-dialog-body">' + content + '</div><p class="llm-form-error" role="alert"></p><footer><button type="button" class="quiet-button" data-close>' + uiText('Cancel', '取消') + '</button><button class="quiet-button llm-primary" type="submit">' + esc(saveLabel) + '</button></footer></form>';
  document.body.appendChild(dialog);
  const close = () => dialog.close();
  dialog.querySelectorAll('[data-close]').forEach(button => button.onclick = close);
  dialog.onclose = () => { dialog.replaceChildren(); dialog.remove(); previous?.focus(); };
  dialog.querySelector('form').onsubmit = async event => {
    event.preventDefault();
    const button = dialog.querySelector('[type=submit]');
    button.disabled = true;
    dialog.querySelector('[role=alert]').textContent = '';
    try { if (await onSave(dialog) !== false) close(); }
    catch (error) { if (dialog.isConnected) dialog.querySelector('[role=alert]').textContent = error.message; }
    finally { button.disabled = false; }
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
    ...options, cache: 'no-store', signal: AbortSignal.timeout(40000),
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

async function llmLoginDialog() {
  const family = state.llmFamily || 'chatgpt';
  const name = family === 'anthropic' ? 'Anthropic' : 'Codex';
  const provider = family === 'anthropic' ? 'claude' : 'codex';
  const backends = (state.llmData?.backends || []).filter(b => b.mode === 'subscription' && b.family === family);
  const managementEnabled = state.llmData?.management_enabled === true;
  const localSession = state.llmData?.management_mode === 'local';
  let login = null, expiresAt = 0, saved = false;
  const content = '<div class="llm-login-heading"><p>' + uiText('Sign in through OAuth to automatically obtain and save the authentication file.', '通过 OAuth 流程登录，自动获取并保存认证文件。') + '</p><button type="button" class="quiet-button llm-primary" data-start>' + uiText('Start ' + name + ' login', '开始 ' + name + ' 登录') + '</button></div>'
    + '<label>' + uiText('Backend binding · optional', '后端绑定 · 可选') + '<select name="backend">' + backends.map(b => '<option value="' + esc(b.name) + '">' + esc(b.name) + '</option>').join('') + '<option value="">' + uiText('Save account without binding', '暂不绑定，先保存账户') + '</option></select></label>'
    + (managementEnabled ? (localSession ? '' : '<section data-login-management' + (llmManagementToken ? ' hidden' : '') + '><label>' + uiText('Gateway management token', '网关管理令牌') + '<input name="management_token" type="password" autocomplete="off" minlength="24"></label><p class="muted">' + uiText('Enter the XGATE_LLM_ADMIN_TOKEN (or DXGATE_LLM_ADMIN_TOKEN) configured when starting this gateway, then start login to generate the authorization link. This is a gateway credential, not your OpenAI or Anthropic password. It stays in this page session.', '填写启动此网关时设置的 XGATE_LLM_ADMIN_TOKEN（或 DXGATE_LLM_ADMIN_TOKEN），再点击开始登录生成授权链接。这是网关管理凭证，无需填写 OpenAI 或 Anthropic 密码；仅在当前页面会话内使用。') + '</p></section>')
      : '<section class="llm-login-setup" role="note"><strong>' + uiText('OAuth management is not enabled', '当前服务尚未启用 OAuth 管理') + '</strong><p>' + uiText('No authorization link can be generated yet. Configure the account directory and management token, restart the gateway, then reload this page.', '目前无法生成授权链接。请配置账户保存目录和管理令牌，重启网关后刷新此页面。') + '</p><code>XGATE_LLM_ACCOUNTS_DIR</code> (or <code>DXGATE_LLM_ACCOUNTS_DIR</code>)<p>' + uiText('A private directory for authentication files.', '用于保存认证文件的私有目录。') + '</p><code>XGATE_LLM_ADMIN_TOKEN</code> (or <code>DXGATE_LLM_ADMIN_TOKEN</code>)<p>' + uiText('A management token you create, at least 24 characters long.', '由你设置的管理令牌，至少 24 个字符。') + '</p></section>')
    + '<section class="llm-login-link" hidden><span class="muted">' + uiText('Authorization link', '授权链接') + '</span><p data-url></p><div class="llm-heading-actions"><button type="button" class="quiet-button" data-copy>' + uiText('Copy link', '复制链接') + '</button><a class="quiet-button" data-open target="_blank" rel="noopener noreferrer" referrerpolicy="no-referrer">' + uiText('Open link', '打开链接') + '</a></div></section>'
    + '<label class="llm-login-callback" hidden>' + uiText('Callback URL', '回调 URL') + '<input name="callback" type="url" autocomplete="off" spellcheck="false" maxlength="16384"><span>' + uiText('After authorization redirects to localhost, copy the full URL from the address bar and submit it here, even if the page cannot connect.', '授权跳转到 localhost 后，即使页面无法连接，也请复制地址栏中的完整 URL 并提交到这里。') + '</span></label>'
    + '<p class="llm-login-status" role="status" aria-live="polite">' + (managementEnabled ? uiText('Start login to generate your authorization link.', '点击开始登录后生成授权链接。') : uiText('Waiting for gateway configuration', '等待启用网关 OAuth 管理')) + '</p>';
  const credentials = async form => {
    if (localSession) { if (!llmManagementToken) await llmLocalSession(); return; }
    const input = form.querySelector('[name=management_token]');
    if (input?.value) llmManagementToken = input.value;
    if (!llmManagementToken || llmManagementToken.length < 24) {
      llmManagementToken = '';
      form.querySelector('[data-login-management]').hidden = false;
      input.focus();
      throw new Error(uiText('Enter the gateway management token configured at startup (at least 24 characters).', '请填写启动网关时配置的管理令牌（至少 24 个字符）。'));
    }
  };
  const showCredentialError = form => {
    if (!llmManagementToken && form.isConnected && managementEnabled && !localSession) {
      form.querySelector('[data-login-management]').hidden = false;
      form.querySelector('[name=management_token]').value = '';
    }
  };
  const dialog = llmModal(name + ' OAuth', content, uiText('Submit callback URL', '提交回调 URL'), async form => {
    if (!login || Date.now() >= expiresAt) throw new Error(uiText('Login expired. Start a new login.', '登录已过期，请重新开始登录。'));
    const status = form.querySelector('[role=status]');
    form.querySelector('[data-start]').disabled = true;
    status.textContent = uiText('Authenticating and saving…', '正在认证并保存…');
    try {
      await credentials(form);
      const response = await llmAdminFetch(login.id + '/callback', { method: 'POST', body: JSON.stringify({ callback_url: form.querySelector('[name=callback]').value.trim() }) }, '/admin/llm/oauth/');
      await completed(await response.json());
    } catch (error) { showCredentialError(form); status.textContent = uiText('Not completed. Check the error or restart login.', '尚未完成，请检查错误或重新开始登录。'); throw error; }
    finally { if (form.isConnected) form.querySelector('[data-start]').disabled = false; }
    return false;
  });
  dialog.classList.add('llm-login-dialog');
  const submit = dialog.querySelector('[type=submit]');
  submit.hidden = true;
  const completed = async account => {
    if (saved) return;
    saved = true; login = null;
    if (dialog.isConnected) {
      dialog.querySelector('[name=callback]').value = '';
      for (const selector of ['.llm-login-link', '.llm-login-callback', '[type=submit]', '[data-start]']) dialog.querySelector(selector).hidden = true;
      dialog.querySelector('[data-url]').textContent = '';
      dialog.querySelector('[data-open]').removeAttribute('href');
      dialog.querySelector('[role=status]').textContent = uiText('Signed in. Authentication file saved: ', '登录成功，认证文件已保存：') + account.id;
      dialog.querySelector('footer [data-close]').textContent = uiText('Done', '完成');
    }
    await llmReload().catch(() => notify(uiText('Account saved. Refresh the dashboard to see it.', '账户已保存，请刷新仪表盘查看。')));
  };
  const start = dialog.querySelector('[data-start]');
  start.disabled = !managementEnabled;
  start.onclick = async () => {
    start.disabled = true; submit.disabled = true;
    const status = dialog.querySelector('[role=status]');
    dialog.querySelector('[role=alert]').textContent = '';
    status.textContent = uiText('Creating authorization link…', '正在生成授权链接…');
    try {
      await credentials(dialog);
      if (login) { await llmAdminFetch(login.id, { method: 'DELETE' }, '/admin/llm/oauth/'); login = null; }
      const response = await llmAdminFetch('start', { method: 'POST', body: JSON.stringify({ provider, backend: dialog.querySelector('[name=backend]').value }) }, '/admin/llm/oauth/');
      if (!dialog.isConnected) return;
      login = await response.json(); expiresAt = Date.now() + login.expires_in * 1000;
      dialog.dataset.callbackMode = login.callback_mode;
      if (!localSession) { dialog.querySelector('[name=management_token]').value = ''; dialog.querySelector('[data-login-management]').hidden = true; }
      dialog.querySelector('[name=backend]').disabled = true;
      dialog.querySelector('[data-url]').textContent = login.authorization_url;
      dialog.querySelector('[data-open]').href = login.authorization_url;
      dialog.querySelector('[name=callback]').placeholder = login.redirect_uri + '?code=…&state=…';
      dialog.querySelector('[name=callback]').value = '';
      dialog.querySelector('[name=callback]').required = true;
      dialog.querySelector('.llm-login-link').hidden = false;
      dialog.querySelector('.llm-login-callback').hidden = false;
      submit.hidden = false;
      status.textContent = login.callback_mode === 'automatic' ? uiText('Waiting for authorization… Same-machine browsers complete automatically; remote browsers can submit the callback URL below.', '等待认证中… 同机浏览器授权后将自动保存；远程浏览器可在下方提交回调 URL。') : uiText('Callback port is unavailable. Authorize, then paste the full callback URL below. Link expires in 10 minutes.', '回调端口不可用。请完成授权后在下方粘贴完整回调 URL；链接 10 分钟内有效。');
      start.textContent = uiText('Restart login', '重新开始登录');
    } catch (error) { showCredentialError(dialog); if (dialog.isConnected) dialog.querySelector('[role=alert]').textContent = error.message; status.textContent = uiText('Could not start login', '登录未能启动'); }
    finally { start.disabled = false; submit.disabled = !login; }
  };
  dialog.querySelector('[data-copy]').onclick = async () => {
    try { await navigator.clipboard.writeText(login.authorization_url); notify(uiText('Link copied', '链接已复制')); }
    catch { dialog.querySelector('[role=alert]').textContent = uiText('Copy unavailable. Select and copy the link above.', '无法自动复制，请选中上方链接手动复制。'); }
  };
  let polling = false;
  const timer = setInterval(async () => {
    if (login && Date.now() >= expiresAt && !saved) {
      login = null; submit.disabled = true;
      dialog.querySelector('[data-open]').removeAttribute('href');
      dialog.querySelector('[role=status]').textContent = uiText('Login expired. Start a new login.', '登录已过期，请重新开始登录。');
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
        login = null; submit.disabled = true;
        dialog.querySelector('[role=status]').textContent = uiText('Login did not complete. Start a new login.', '登录未完成，请重新开始登录。');
        dialog.querySelector('[role=alert]').textContent = result.error || '';
        dialog.querySelector('[data-open]').removeAttribute('href');
      } else if (result.status === 'exchanging') dialog.querySelector('[role=status]').textContent = uiText('Authenticating and saving…', '正在认证并保存…');
    } catch (error) { if (dialog.isConnected) dialog.querySelector('[role=alert]').textContent = error.message; }
    finally { polling = false; }
  }, 1500);
  dialog.addEventListener('close', () => {
    clearInterval(timer);
    if (login && llmManagementToken) fetch('/admin/llm/oauth/' + login.id, { method: 'DELETE', headers: { Authorization: 'Bearer ' + llmManagementToken }, keepalive: true }).catch(() => {});
    login = null;
  });
  if (localSession && managementEnabled && backends.length <= 1) start.onclick();
}

async function llmReload() {
  const response = await fetch('/debug/llm', { cache: 'no-store', signal: AbortSignal.timeout(8000) });
  if (!response.ok) throw new Error('LLM dashboard returned ' + response.status);
  state.llmData = await response.json();
  delete state.endpointErrors['/debug/llm'];
  renderLlm();
}

function llmFileEditor(account, isNew) {
  const family = account.document.type === 'claude' ? 'anthropic' : 'chatgpt';
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
      const family = document.type === 'claude' ? 'anthropic' : 'chatgpt';
      const old = existing ? await (await llmAdminFetch(encodeURIComponent(existing.id))).json() : { id: file.name.replace(/\.json$/i, '').replace(/[^A-Za-z0-9_-]/g, '-'), backend: subscriptions.find(b => b.family === family)?.name || '', revision: 0, models: [] };
      llmFileEditor({ ...old, document }, !existing);
    } catch (error) { notify(error.message); }
  };
  input.click();
}

async function llmEditModels(summary) {
  const account = await (await llmAdminFetch(encodeURIComponent(summary.id))).json();
  const backend = state.llmData.backends.find(b => b.name === account.backend);
  const rules = account.models.length ? account.models : (backend?.models || []).map(model => ({ model, disabled: false, alias: '', reasoning_effort: '' }));
  const content = '<p class="muted">' + uiText('Rules apply to this account. Empty reasoning keeps the caller setting. Disabled models cannot be selected through their names or aliases.', '规则作用于此账户。留空保留调用方推理设置；禁用模型后，原名与别名均不可选。') + '</p><div class="llm-model-rules"></div><button type="button" class="quiet-button" data-add>' + uiText('Add model', '添加模型') + '</button>';
  const dialog = llmModal(uiText('Model settings', '模型设置'), content, uiText('Save rules', '保存规则'), async form => {
    account.models = [...form.querySelectorAll('.llm-model-row')].map(row => ({
      model: row.querySelector('[name=model]').value.trim(), alias: row.querySelector('[name=alias]').value.trim(),
      disabled: row.querySelector('[name=disabled]').checked, reasoning_effort: row.querySelector('[name=effort]').value
    }));
    await llmAdminFetch(encodeURIComponent(account.id), { method: 'PUT', body: JSON.stringify(account) });
    await llmReload(); notify(uiText('Model rules saved', '模型规则已保存'));
  });
  function add(rule = {}) {
    const row = document.createElement('div'); row.className = 'llm-model-row';
    row.innerHTML = '<label>' + uiText('Model', '模型') + '<input name="model" required value="' + esc(rule.model || '') + '"></label><label>' + uiText('Alias', '别名') + '<input name="alias" value="' + esc(rule.alias || '') + '"></label><label>' + uiText('Default reasoning', '默认推理等级') + '<select name="effort">' + ['', 'none', 'minimal', 'low', 'medium', 'high', 'xhigh'].map(e => '<option value="' + e + '"' + (e === rule.reasoning_effort ? ' selected' : '') + '>' + (e || uiText('Inherit', '继承')) + '</option>').join('') + '</select></label><label class="llm-check"><input name="disabled" type="checkbox"' + (rule.disabled ? ' checked' : '') + '>' + uiText('Disabled', '禁用') + '</label><button class="quiet-button" type="button" aria-label="' + uiText('Remove model rule', '移除模型规则') + '">×</button>';
    row.querySelector('button').onclick = () => row.remove();
    dialog.querySelector('.llm-model-rules').appendChild(row);
  }
  rules.forEach(add);
  dialog.querySelector('[data-add]').onclick = () => add();
}

async function llmAccountAction(action, account, button) {
  if (action === 'upload') { llmChooseFile(account); return; }
  button.disabled = true;
  try {
    const id = encodeURIComponent(account.id);
    if (action === 'refresh') {
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
  } catch (error) { notify(error.message); }
  finally { button.disabled = false; }
}

function renderFamilyDropdown(family) {
  const actions = $('page-actions');
  if (!actions) return;
  const current = family === 'anthropic' ? 'Anthropic' : 'ChatGPT';
  actions.innerHTML = '<div class="llm-family-dropdown" id="llm-family-dropdown">'
    + '<button type="button" class="llm-family-trigger" id="llm-family-trigger" aria-haspopup="listbox" aria-expanded="false" aria-label="' + uiText('Provider', '供应商') + '">'
    + '<span class="llm-family-val">' + current + '</span>'
    + '<svg class="llm-family-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>'
    + '</button>'
    + '<div class="llm-family-menu" id="llm-family-menu" role="listbox" hidden>'
    + '<button type="button" role="option" class="llm-family-option' + (family === 'chatgpt' ? ' active' : '') + '" data-value="chatgpt" aria-selected="' + (family === 'chatgpt') + '"><span>ChatGPT</span><svg class="llm-family-check" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg></button>'
    + '<button type="button" role="option" class="llm-family-option' + (family === 'anthropic' ? ' active' : '') + '" data-value="anthropic" aria-selected="' + (family === 'anthropic') + '"><span>Anthropic</span><svg class="llm-family-check" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg></button>'
    + '</div>'
    + '<select id="llm-family" class="hidden" aria-hidden="true" tabindex="-1"><option value="chatgpt"' + (family === 'chatgpt' ? ' selected' : '') + '>ChatGPT</option><option value="anthropic"' + (family === 'anthropic' ? ' selected' : '') + '>Anthropic</option></select>'
    + '</div>';

  const trigger = $('llm-family-trigger');
  const menu = $('llm-family-menu');
  if (!trigger || !menu) return;

  function closeMenu() {
    menu.hidden = true;
    trigger.setAttribute('aria-expanded', 'false');
    document.removeEventListener('click', onDocClick);
    document.removeEventListener('keydown', onDocKey);
  }
  function onDocClick(e) {
    if (!trigger.contains(e.target) && !menu.contains(e.target)) closeMenu();
  }
  function onDocKey(e) {
    if (e.key === 'Escape') closeMenu();
  }

  trigger.onclick = (e) => {
    e.stopPropagation();
    if (menu.hidden) {
      menu.hidden = false;
      trigger.setAttribute('aria-expanded', 'true');
      setTimeout(() => {
        document.addEventListener('click', onDocClick);
        document.addEventListener('keydown', onDocKey);
      }, 0);
    } else {
      closeMenu();
    }
  };

  menu.querySelectorAll('.llm-family-option').forEach(btn => {
    btn.onclick = (e) => {
      e.stopPropagation();
      const val = btn.dataset.value;
      closeMenu();
      if (val !== state.llmFamily) {
        state.llmFamily = val;
        renderLlm();
      }
    };
  });
}

function renderLlmWorkspace() {
  const host = $('tab-llm');
  const data = state.llmData || { accounts: [], backends: [] };
  const mode = state.llmMode || 'subscription', family = state.llmFamily || 'chatgpt';
  const modes = [['subscription', uiText('Account subscription', '账户订阅')], ['api', uiText('API', 'API')], ['local', uiText('Local open weights', '本地开源权重')]];
  const selected = data.backends.filter(b => b.mode === mode && (mode === 'local' || b.family === family));
  const error = state.endpointErrors['/debug/llm'];
  host.innerHTML = '<div class="llm-workspace"><div class="llm-workspace-toolbar"><div class="llm-mode-tabs" role="group" aria-label="' + uiText('LLM mode', 'LLM 模式') + '">' + modes.map(([id, label]) => '<button type="button" data-mode="' + id + '" aria-pressed="' + (mode === id) + '">' + label + '</button>').join('') + '</div></div>'
    + (error ? '<div class="data-notice" role="alert">' + esc(error) + ' · ' + uiText('Displayed data may be stale.', '当前显示的数据可能已过期。') + '</div>' : '')
    + '<div class="llm-section-heading"><h3>' + (mode === 'subscription' ? uiText('Accounts', '账户') : uiText('Backends', '后端')) + '</h3>' + (mode === 'subscription' ? '<div class="llm-heading-actions"><button id="llm-login" type="button" class="quiet-button llm-icon-button" title="' + uiText('OAuth login', 'OAuth 登录') + '" aria-label="' + uiText('OAuth login', 'OAuth 登录') + '">' + llmIcon('login') + '</button><button id="llm-import" type="button" class="quiet-button llm-icon-button" title="' + uiText('Upload OAuth file', '上传 OAuth 文件') + '" aria-label="' + uiText('Upload OAuth file', '上传 OAuth 文件') + '">' + llmIcon('upload') + '</button></div>' : '') + '</div><div class="llm-accounts-grid" id="llm-accounts-grid"></div></div>';
  if (mode !== 'local') {
    renderFamilyDropdown(family);
  } else {
    const actions = $('page-actions');
    if (actions) actions.innerHTML = '';
  }
  host.querySelectorAll('[data-mode]').forEach(button => button.onclick = () => { state.llmMode = button.dataset.mode; renderLlm(); });
  if ($('llm-import')) $('llm-import').onclick = () => llmChooseFile();
  if ($('llm-login')) $('llm-login').onclick = () => llmLoginDialog();
  const query = (state.queries?.llm || state.query || '').toLowerCase();
  const accounts = mode === 'subscription' ? data.accounts.filter(a => (a.provider === 'claude' ? 'anthropic' : 'chatgpt') === family).map(a => ({ ...a, detail: data.backends.find(b => b.name === a.backend) || { name: uiText('Unbound account', '未绑定账户'), models: (a.models || []).map(rule => rule.model), provider: a.provider } })) : selected.map(b => ({ id: b.name, backend: b.name, detail: b }));
  const visible = accounts.filter(a => JSON.stringify([a.id, a.email, a.backend, a.detail.models]).toLowerCase().includes(query));
  const grid = $('llm-accounts-grid');
  for (const account of visible) {
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
}
