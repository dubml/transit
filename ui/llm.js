/* OAuth account workspace. Loaded before the shared UI bootstraps. */
let llmManagementToken = '';

// ChatGPT mark: Cli-Proxy-API-Management-Center; see ui/THIRD_PARTY_NOTICES.md.
const llmChatGptLogo = '<svg role="img" aria-label="ChatGPT" focusable="false" fill="currentColor" fill-rule="evenodd" height="24" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg"><path d="M21.55 10.004a5.416 5.416 0 00-.478-4.501c-1.217-2.09-3.662-3.166-6.05-2.66A5.59 5.59 0 0010.831 1C8.39.995 6.224 2.546 5.473 4.838A5.553 5.553 0 001.76 7.496a5.487 5.487 0 00.691 6.5 5.416 5.416 0 00.477 4.502c1.217 2.09 3.662 3.165 6.05 2.66A5.586 5.586 0 0013.168 23c2.443.006 4.61-1.546 5.361-3.84a5.553 5.553 0 003.715-2.66 5.488 5.488 0 00-.693-6.497v.001zm-8.381 11.558a4.199 4.199 0 01-2.675-.954c.034-.018.093-.05.132-.074l4.44-2.53a.71.71 0 00.364-.623v-6.176l1.877 1.069c.02.01.033.029.036.05v5.115c-.003 2.274-1.87 4.118-4.174 4.123zM4.192 17.78a4.059 4.059 0 01-.498-2.763c.032.02.09.055.131.078l4.44 2.53c.225.13.504.13.73 0l5.42-3.088v2.138a.068.068 0 01-.027.057L9.9 19.288c-1.999 1.136-4.552.46-5.707-1.51h-.001zM3.023 8.216A4.15 4.15 0 015.198 6.41l-.002.151v5.06a.711.711 0 00.364.624l5.42 3.087-1.876 1.07a.067.067 0 01-.063.005l-4.489-2.559c-1.995-1.14-2.679-3.658-1.53-5.63h.001zm15.417 3.54l-5.42-3.088L14.896 7.6a.067.067 0 01.063-.006l4.489 2.557c1.998 1.14 2.683 3.662 1.529 5.633a4.163 4.163 0 01-2.174 1.807V12.38a.71.71 0 00-.363-.623zm1.867-2.773a6.04 6.04 0 00-.132-.078l-4.44-2.53a.731.731 0 00-.729 0l-5.42 3.088V7.325a.068.068 0 01.027-.057L14.1 4.713c2-1.137 4.555-.46 5.707 1.513.487.833.664 1.809.499 2.757h.001zm-11.741 3.81l-1.877-1.068a.065.065 0 01-.036-.051V6.559c.001-2.277 1.873-4.122 4.181-4.12.976 0 1.92.338 2.671.954-.034.018-.092.05-.131.073l-4.44 2.53a.71.71 0 00-.365.623l-.003 6.173v.002zm1.02-2.168L12 9.25l2.414 1.375v2.75L12 14.75l-2.415-1.375v-2.75z"></path></svg>';

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
    codex: '<path d="m6 7 4 5-4 5m7 0h5"/>'
  };
  return '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">' + paths[name] + '</svg>';
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
    + (managementEnabled ? (localSession ? '' : '<section data-login-management' + (llmManagementToken ? ' hidden' : '') + '><label>' + uiText('Gateway management token', '网关管理令牌') + '<input name="management_token" type="password" autocomplete="off" minlength="24"></label><p class="muted">' + uiText('Use the management key saved in Configuration, or the initial TRANSIT_LLM_ADMIN_TOKEN. It stays in this page session.', '填写 Configuration 中保存的管理密钥，或首次启动时设置的 TRANSIT_LLM_ADMIN_TOKEN；仅用于当前页面会话。') + '</p></section>')
      : '<section class="llm-login-setup" role="note"><strong>' + uiText('OAuth management is not enabled', '当前服务尚未启用 OAuth 管理') + '</strong><p>' + uiText('No authorization link can be generated yet. Configure the account directory and management token, restart the gateway, then reload this page.', '目前无法生成授权链接。请配置账户保存目录和管理令牌，重启网关后刷新此页面。') + '</p><code>TRANSIT_LLM_ACCOUNTS_DIR</code><p>' + uiText('A private directory for authentication files.', '用于保存认证文件的私有目录。') + '</p><code>TRANSIT_LLM_ADMIN_TOKEN</code><p>' + uiText('A management token you create, at least 24 characters long.', '由你设置的管理令牌，至少 24 个字符。') + '</p></section>')
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
  const response = await managementFetch('/debug/llm', { cache: 'no-store', signal: AbortSignal.timeout(8000) });
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
      const family = document.type === 'claude' ? 'anthropic' : 'chatgpt';
      const old = existing ? await (await llmAdminFetch(encodeURIComponent(existing.id))).json() : { id: file.name.replace(/\.json$/i, '').replace(/[^A-Za-z0-9_-]/g, '-'), backend: subscriptions.find(b => b.family === family)?.name || '', revision: 0, models: [] };
      llmFileEditor({ ...old, document }, !existing);
    } catch (error) { notify(error.message); }
  };
  input.click();
}

async function llmEditModels(summary) {
  let account = await (await llmAdminFetch(encodeURIComponent(summary.id))).json();
  const backend = state.llmData.backends.find(b => b.name === account.backend);
  const excluded = new Set(account.models.filter(r => r.disabled && !r.model.includes('*')).map(r => r.model));
  const patterns = account.models.filter(r => r.disabled && r.model.includes('*')).map(r => r.model);
  const known = new Map([...(backend?.models || []), ...account.models.map(r => r.model).filter(m => !m.includes('*'))].map(id => [id,id]));
  const content = '<p class="llm-model-scope"><strong>' + esc(account.document.type === 'codex' ? 'Codex' : 'Claude') + '</strong><span>' + esc(summary.email || summary.id) + '</span></p>'
    + '<p class="muted">' + uiText('Applies to this account. Excluded models cannot be routed through their original names or aliases.', '仅作用于当前账户。禁用后，原模型及其别名都不会路由到此账户。') + '</p>'
    + '<div class="llm-model-sections"><section class="llm-model-section"><header><h3>' + uiText('OAuth model exclusions', 'OAuth 模型禁用') + '</h3><span data-excluded-count></span></header><p class="muted">' + uiText('Select models you do not want to use.', '勾选不想使用的模型。') + '</p><input type="search" data-model-search aria-label="' + uiText('Search models', '搜索模型') + '" placeholder="' + uiText('Search models', '搜索模型') + '"><p class="llm-catalog-status" role="status">' + uiText('Loading available models…', '正在加载可用模型…') + '</p><div class="llm-model-options"></div><label>' + uiText('Exclusion rules', '禁用规则') + '<textarea name="excluded_patterns" rows="3" placeholder="gpt-5-*">' + esc(patterns.join('\n')) + '</textarea></label><p class="muted">' + uiText('One model name or pattern per line. * matches any characters; exclusions are case-insensitive.', '每行一个模型名或规则；* 匹配任意字符，禁用匹配不区分大小写。') + '</p></section>'
    + '<section class="llm-model-section"><header><h3>' + uiText('OAuth model aliases', 'OAuth 模型别名') + '</h3><button class="quiet-button" type="button" data-add-alias>+ ' + uiText('Add alias', '添加别名') + '</button></header><p class="muted">' + uiText('Clients call the alias; requests use the original model.', '客户端使用别名调用，请求发送给原模型。') + '</p><datalist id="llm-model-catalog"></datalist><div class="llm-model-rules"></div><p class="muted" data-alias-empty>' + uiText('No aliases. Add one to rename a model.', '暂无别名，点击“添加别名”修改模型名称。') + '</p></section></div>';
  const dialog = llmModal(uiText('Model management', '模型管理'), content, uiText('Save rules', '保存规则'), async form => {
    const rules = new Map(account.models.filter(r => !r.model.includes('*')).map(r => [r.model,{...r,disabled:excluded.has(r.model),alias:'',keep_original:true}]));
    const ensure = model => { if (!rules.has(model)) rules.set(model,{model,disabled:false,alias:'',keep_original:true,reasoning_effort:''}); return rules.get(model); };
    for (const model of excluded) ensure(model).disabled = true;
    const sources = new Set();
    for (const row of form.querySelectorAll('.llm-alias-row')) {
      const model = row.querySelector('[name=model]').value.trim(), alias = row.querySelector('[name=alias]').value.trim();
      if (!model || !alias) throw new Error(uiText('Original model and alias are required.', '原模型名称和别名不能为空。'));
      if (sources.has(model)) throw new Error(uiText('Each original model can have one alias.', '同一个原模型只能设置一条别名。'));
      sources.add(model);
      Object.assign(ensure(model),{alias,keep_original:row.querySelector('[name=keep_original]').checked,reasoning_effort:row.querySelector('[name=effort]').value});
    }
    for (const model of form.querySelector('[name=excluded_patterns]').value.split('\n').map(s => s.trim()).filter(Boolean)) ensure(model).disabled = true;
    const next = {...account,models:[...rules.values()].filter(r => r.disabled || r.alias || r.reasoning_effort)};
    await llmAdminFetch(encodeURIComponent(account.id), { method: 'PUT', body: JSON.stringify(next) });
    await llmReload(); notify(uiText('Model rules saved', '模型规则已保存'));
  });
  dialog.classList.add('llm-model-dialog');
  function matchesPattern(pattern, model) {
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
  function updateCount() {
    const rules = [...excluded,...manualRules()];
    const count = [...known.keys()].filter(id=>rules.some(rule=>matchesPattern(rule,id))).length;
    dialog.querySelector('[data-excluded-count]').textContent = uiText('Excluded ', '已禁用 ') + count + ' / ' + known.size;
  }
  function renderCatalog() {
    const query = dialog.querySelector('[data-model-search]').value.trim().toLowerCase();
    const items = [...known].sort(([a],[b]) => a.localeCompare(b));
    dialog.querySelector('.llm-model-options').innerHTML = items.filter(([id,label]) => (id+' '+label).toLowerCase().includes(query)).map(([id,label]) => { const byRule=manualRules().some(rule=>matchesPattern(rule,id)); return '<label class="llm-model-option"><input type="checkbox" data-exclude-model="' + esc(id) + '"' + (byRule || [...excluded].some(rule=>matchesPattern(rule,id)) ? ' checked' : '') + (byRule ? ' disabled title="' + uiText('Excluded by a rule below', '已被下方规则禁用') + '"' : '') + '><span title="' + esc(label) + '">' + esc(id) + '</span></label>'; }).join('') || '<p class="muted">' + uiText('No matching models. You can enter a rule below.', '暂无匹配模型，可以在下方手动输入规则。') + '</p>';
    dialog.querySelector('#llm-model-catalog').innerHTML = items.map(([id,label]) => '<option value="' + esc(id) + '">' + esc(label) + '</option>').join('');
    dialog.querySelectorAll('[data-exclude-model]').forEach(input => input.onchange = () => { if (input.checked) excluded.add(input.dataset.excludeModel); else excluded.delete(input.dataset.excludeModel); updateCount(); });
    updateCount();
  }
  function addAlias(rule = {}) {
    const row = document.createElement('div'); row.className = 'llm-alias-row';
    row.innerHTML = '<label>' + uiText('Original model', '原模型名称') + '<input name="model" list="llm-model-catalog" required autocomplete="off" value="' + esc(rule.model || '') + '"></label><span class="llm-alias-arrow" aria-hidden="true">→</span><label>' + uiText('Alias', '别名') + '<input name="alias" required autocomplete="off" value="' + esc(rule.alias || '') + '"></label><div class="llm-alias-options"><label class="llm-check"><input name="keep_original" type="checkbox"' + (rule.keep_original ? ' checked' : '') + '>' + uiText('Keep original name', '保留原名') + '</label><button class="quiet-button llm-delete" type="button" data-remove-alias aria-label="' + uiText('Remove alias', '删除别名') + '">' + llmIcon('delete') + '</button></div><details><summary>' + uiText('Default reasoning', '默认推理等级') + '</summary><select name="effort" aria-label="' + uiText('Default reasoning', '默认推理等级') + '">' + ['', 'none', 'minimal', 'low', 'medium', 'high', 'xhigh'].map(e => '<option value="' + e + '"' + (e === rule.reasoning_effort ? ' selected' : '') + '>' + (e || uiText('Inherit', '继承')) + '</option>').join('') + '</select></details>';
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
      for (const model of catalog.models) known.set(model.id,model.display_name || model.id);
      renderCatalog();
      dialog.querySelector('.llm-catalog-status').textContent = uiText('Loaded ', '已加载 ') + catalog.models.length + uiText(' provider models', ' 个供应商模型');
    } catch (error) {
      if (dialog.isConnected) dialog.querySelector('.llm-catalog-status').textContent = uiText('Model list unavailable; manual rules still work. ', '模型列表加载失败，仍可手动填写规则。') + error.message;
    }
  }
  void loadCatalog();
}

async function llmAccountAction(action, account, button) {
  if (action === 'upload') { llmChooseFile(account); return; }
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

function llmQuotaTime(value) {
  const [absolute, relative] = llmDate(value).split(' · ');
  const soon = Date.parse(value) - Date.now();
  return '<time title="' + esc(value || '') + '">' + esc(absolute) + (relative ? ' · <span' + (soon > 0 && soon < 3600000 ? ' class="llm-time-soon"' : '') + '>' + esc(relative) + '</span>' : '') + '</time>';
}

function llmQuotaMarkup(account) {
  const q = account.quota;
  const rawPlan = q?.plan_type || account.plan_type;
  const plan = rawPlan ? rawPlan[0].toUpperCase() + rawPlan.slice(1) : '—';
  const renewal = q?.renewal_at || account.renewal_at;
  const credits = q?.reset_credits;
  const loading = llmQuotaJobs.get(account.id)?.loading;
  const windows = (q?.windows || []).map(w => {
    const seconds = Number(w.window_seconds);
    const period = seconds === 18000 ? uiText('5-hour limit', '5 小时限额') : seconds === 604800 ? uiText('Weekly limit', '周限额') : seconds >= 2419200 && seconds <= 2678400 ? uiText('Monthly limit', '月限额') : uiText(Math.round(seconds / 3600) + '-hour limit', Math.round(seconds / 3600) + ' 小时限额');
    const prefix = ({ five_hour: '', seven_day: '', seven_day_opus: 'Opus', seven_day_sonnet: 'Sonnet', iguana_necktie: 'Fable', 'code-review': uiText('Code review', '代码审查') })[w.name] ?? w.name;
    const remaining = typeof w.used_percent === 'number' ? Math.max(0,Math.min(100,100-w.used_percent)) : null;
    const label = [prefix,period].filter(Boolean).join(' ');
    return '<div class="quota-window-item"><div class="quota-window-label-row"><strong>' + esc(label) + '</strong><span><b title="' + uiText('Remaining quota', '剩余额度') + '">' + (remaining === null ? '—' : Math.round(remaining) + '%') + '</b> ' + llmQuotaTime(w.reset_at) + '</span></div><div class="llm-quota-track"' + (remaining === null ? ' aria-label="' + uiText('Quota not reported', '额度未返回') + '"' : ' role="progressbar" aria-label="' + esc(label) + '" aria-valuemin="0" aria-valuemax="100" aria-valuenow="' + remaining + '"') + '><span style="width:' + (remaining ?? 0) + '%" class="' + (remaining !== null && remaining <= 20 ? 'low' : '') + '"></span></div></div>';
  }).join('');
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
  return '<section class="llm-quota-panel"><div class="llm-plan-pills"><span>' + uiText('Plan: ', '套餐：') + '<b>' + esc(plan) + '</b></span><span>' + uiText('Renewal ', '续费时间 ') + llmQuotaTime(renewal) + '</span>' + (account.provider === 'codex' ? '<span>' + uiText('Reset credits ', '主动重置次数 ') + '<b>' + esc(credits?.available_count ?? '—') + '</b></span>' : '') + '</div>'
    + '<div class="llm-quota-windows">' + (windows || '<p class="muted">' + (loading ? uiText('Loading provider quota…', '正在查询供应商额度…') : uiText('Quota not reported. Refresh to query the provider.', '尚无额度数据，请刷新查询供应商。')) + '</p>') + '</div>'
    + (creditRows ? '<div class="llm-credit-expirations"><strong>' + uiText('Reset credit expiration', '主动重置过期时间') + ' (' + esc(timezone) + ')</strong>' + creditRows + '</div>' : '')
    + ([account.quota_error,q?.credits_error,q?.profile_error].filter(Boolean).map(message => '<p class="llm-quota-error" role="status">' + esc(message) + (q?.observed_at ? ' · ' + uiText('Last observation retained', '保留上次观测') : '') + '</p>').join('')) + '</section>';
}

function llmSubscriptionCard(account) {
  const card = document.createElement('article'); card.className = 'llm-account-card llm-subscription-card'; card.dataset.accountId = account.id;
  card.dataset.search = [account.id, account.backend, account.email, account.provider].filter(Boolean).join(' ');
  let hash = 2166136261;
  for (const character of account.id) hash = Math.imul(hash ^ character.charCodeAt(0), 16777619);
  const name = [account.provider, account.email || uiText('Unknown email', '未知邮箱'), (hash >>> 0).toString(16).padStart(8,'0').slice(0,7)].join('-');
  card.innerHTML = '<header class="llm-identity"><span class="llm-provider-mark">' + (account.provider === 'codex' ? llmChatGptLogo : llmIcon('codex')) + '</span><h3 title="' + esc(name) + '">' + esc(name) + '</h3></header>' + llmQuotaMarkup(account)
    + '<footer class="llm-account-footer"><input type="checkbox" data-select aria-label="' + esc(uiText('Select ', '选择 ') + (account.email || account.id)) + '"' + (llmSelectedAccounts.has(account.id) ? ' checked' : '') + '><div class="llm-account-actions"></div><label class="llm-account-toggle"><span>' + uiText('Enabled', '启用') + '</span><input type="checkbox" role="switch" data-action="toggle" aria-label="' + esc(uiText('Enable ', '启用 ') + (account.email || account.id)) + '"' + (account.disabled ? '' : ' checked') + '></label></footer>';
  const labels = { models: uiText('Models', '模型'), quota: uiText('Refresh quota', '刷新额度'), download: uiText('Download OAuth file', '下载 OAuth 文件'), edit: uiText('Account settings', '账户设置'), delete: uiText('Delete account', '删除账户') };
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
