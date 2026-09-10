// Real gateway process + browser + upstream. All settings and credentials use a temporary directory.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const https = require('node:https');
const crypto = require('node:crypto');
const { spawn, execFileSync } = require('node:child_process');
const { once } = require('node:events');
const { chromium } = require('playwright');
const root = path.resolve(__dirname, '..');
const artifact = process.env.CONFIGURATION_ARTIFACTS;
async function port() { const server = http.createServer(); server.listen(0, '127.0.0.1'); await once(server, 'listening'); const value = server.address().port; await new Promise(resolve => server.close(resolve)); return value; }
async function request(url, options = {}) {
  return new Promise((resolve,reject) => {
    const req = (url.startsWith('https:') ? https : http).request(url,{...options,rejectUnauthorized:false},res => {
      const chunks=[]; res.on('data',chunk=>chunks.push(chunk)); res.on('end',()=>resolve({status:res.statusCode,body:Buffer.concat(chunks).toString(),headers:res.headers}));
    }); req.on('error',reject); req.setTimeout(10000,()=>req.destroy(new Error('Request timeout'))); req.end(options.body);
  });
}
(async()=> {
  const temporary = await fs.mkdtemp(path.join(os.tmpdir(),'transit-configuration-'));
  let gateway, browser, page, processOutput='';
  const observed=[];
  const upstream=http.createServer((req,res)=>{observed.push({url:req.url,headers:req.headers});res.setHeader('Content-Type','application/json');res.end('{"ok":true}');});
  try {
    upstream.listen(0,'127.0.0.1'); await once(upstream,'listening');
    let proxyPort=await port(); const initialProxyPort=proxyPort, uiPort=await port(); let scheme='http';
    const configPath=path.join(temporary,'routes.json'), accessPath=path.join(temporary,'access.json');
    const authDir=path.join(temporary,'accounts');
    await fs.writeFile(configPath,JSON.stringify({version:'configuration-test',listeners:[],clusters:[],providers:[],
      backends:[{name:'echo',type:'http',endpoint:'http://127.0.0.1:'+upstream.address().port}],
      routes:[{name:'shared',protocol:'http',matches:[{path:{type:'prefix',value:'/shared'}}],weighted_backends:[{name:'echo',weight:1}],policies:['shared-auth']},{name:'echo',protocol:'http',matches:[{path:{type:'prefix',value:'/'}}],weighted_backends:[{name:'echo',weight:1}],policies:['client-auth']}],
      policies:[{name:'client-auth',auth:{type:'api-key',header:'x-route-key',values:['route-secret']}},{name:'shared-auth',auth:{type:'api-key',header:'authorization',values:['Bearer gateway-client-updated-5678']}}]}));
    const apiBase=()=>scheme+'://127.0.0.1:'+uiPort;
    async function start() {
      const env={...process.env,XDG_CONFIG_HOME:temporary};
      delete env.TRANSIT_LLM_ADMIN_TOKEN; delete env.TRANSIT_ACCESS_CONFIG; delete env.TRANSIT_LLM_ACCOUNTS_DIR;
      gateway=spawn(process.env.TRANSIT_BINARY || path.join(root,'target/debug/transit'),['--xds-enabled=false','--http-addr=127.0.0.1:'+initialProxyPort,'--ui-addr=127.0.0.1:'+uiPort,'--llm-accounts-dir='+authDir,'--access-config='+accessPath,'--static-config='+configPath],{env});
      gateway.stdout.on('data',d=>processOutput=(processOutput+d).slice(-5000)); gateway.stderr.on('data',d=>processOutput=(processOutput+d).slice(-5000));
      for(let i=0;i<150;i++) { if(gateway.exitCode!==null) throw new Error('Gateway exited: '+processOutput); try { if((await request(apiBase()+'/healthz')).status===200) return; }catch{} await new Promise(r=>setTimeout(r,50)); }
      throw new Error('Gateway did not start: '+processOutput);
    }
    async function stop() { if(gateway && gateway.exitCode===null) {const exit=once(gateway,'exit');gateway.kill('SIGTERM');await exit;} }
    await start();
    browser=await chromium.launch({headless:true,executablePath:process.env.CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'});
    const context=await browser.newContext({viewport:{width:1440,height:1000},ignoreHTTPSErrors:true,permissions:['clipboard-read','clipboard-write']});
    page=await context.newPage(); const errors=[];page.on('pageerror',error=>errors.push(error.message));
    await page.goto(apiBase()+'/#configuration');
    await page.locator('#cfg-form').waitFor();
    assert.equal(await page.locator('#cfg-port').inputValue(),String(proxyPort));
    assert.match(await page.locator('#cfg-version').textContent(),/^v\d/);
    assert.equal(await page.locator('#cfg-save').isDisabled(),true);
    await page.locator('#cfg-runtime-tab').focus();await page.keyboard.press('ArrowLeft');assert.equal(await page.locator('#cfg-access-tab').getAttribute('aria-selected'),'true');
    assert.equal((await request(apiBase()+'/admin/access')).status,401);
    assert.equal((await request(apiBase()+'/debug/config')).status,401);
    const token=await page.evaluate(()=>llmManagementToken);
    const admin=(suffix,options={})=>request(apiBase()+suffix,{...options,headers:{Authorization:'Bearer '+token,...options.headers}});
    let baseline=JSON.parse((await admin('/admin/access')).body);
    await page.locator('#cfg-add-key').click();await page.locator('dialog input[name=key]').fill('gateway-client-secret-1234');await page.locator('dialog button[type=submit]').click();
    assert.equal(await page.locator('#cfg-keys .cfg-key').count(),1);
    assert.equal((await fs.stat(accessPath).catch(()=>null)),null);
    await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving && !configurationState.dirty);
    assert.equal((await request('http://127.0.0.1:'+proxyPort+'/echo')).status,401);
    assert.equal((await request('http://127.0.0.1:'+proxyPort+'/echo',{headers:{Authorization:'Bearer gateway-client-secret-1234'}})).status,401);
    assert.equal((await request('http://127.0.0.1:'+proxyPort+'/echo',{headers:{Authorization:'Bearer gateway-client-secret-1234','x-route-key':'route-secret'}})).status,200);
    assert.equal(observed.at(-1).headers.authorization,undefined);
    assert.equal((await request('http://127.0.0.1:'+proxyPort+'/echo?key=gateway-client-secret-1234&keep=1',{headers:{'x-route-key':'route-secret'}})).status,200);
    assert.equal(observed.at(-1).url,'/echo?keep=1');
    await page.locator('[data-action=copy]').click();assert.equal(await page.evaluate(()=>navigator.clipboard.readText()),'gateway-client-secret-1234');
    await page.locator('[data-action=edit]').click();await page.locator('dialog input[name=key]').fill('gateway-client-updated-5678');await page.locator('dialog button[type=submit]').click();await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving && !configurationState.dirty);
    assert.equal((await request('http://127.0.0.1:'+proxyPort+'/echo',{headers:{Authorization:'Bearer gateway-client-secret-1234','x-route-key':'route-secret'}})).status,401);
    assert.equal((await request('http://127.0.0.1:'+proxyPort+'/echo',{headers:{'x-api-key':'gateway-client-updated-5678','x-route-key':'route-secret'}})).status,200);
    assert.equal(observed.at(-1).headers['x-api-key'],undefined);
    assert.equal((await request('http://127.0.0.1:'+proxyPort+'/shared',{headers:{Authorization:'Bearer gateway-client-updated-5678'}})).status,200);
    assert.equal(observed.at(-1).headers.authorization,undefined);
    const conflict=await admin('/admin/access',{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({revision:baseline.settings.revision,config:baseline.settings.config})});assert.equal(conflict.status,409);
    await page.locator('#cfg-host').fill('127.0.0.2');
    const external=JSON.parse((await admin('/admin/access')).body).settings;
    external.config['remote-management']['disable-auto-update-panel']=true;
    const externalSaved=JSON.parse((await admin('/admin/access',{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({revision:external.revision,config:external.config})})).body).settings;
    await page.evaluate(async()=>{if(configurationState.loading)await configurationState.loading;await configurationRefresh(false);});
    assert.equal(await page.evaluate(()=>configurationState.data.settings.revision),externalSaved.revision);
    await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving);
    assert.match(await page.locator('#cfg-error').textContent(),/changed elsewhere/);
    assert.equal(JSON.parse((await admin('/admin/access')).body).settings.config.host,'127.0.0.1');
    await page.reload();await page.locator('#cfg-form').waitFor();
    await page.locator('#cfg-remote summary').click();
    await page.locator('#cfg-allow-remote').check();await page.locator('#cfg-save').click();await page.locator('#cfg-error:not(.hidden)').waitFor();assert.match(await page.locator('#cfg-error').textContent(),/management key/i);
    const managementKey='management-key-for-configuration-test-1234';await page.locator('#cfg-secret').fill(managementKey);await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving && !configurationState.dirty);
    const repository='https://github.com/fixture/transit-panel';
    const packagePath=path.join(temporary,'panels',crypto.createHash('sha256').update(repository).digest('hex')+'.html');
    execFileSync('python3',[path.join(root,'scripts/package-management.py'),packagePath],{stdio:'ignore'});
    await page.locator('#cfg-panel-repository').fill(repository);await page.locator('#cfg-disable-auto-update-panel').check();await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving && !configurationState.dirty);
    assert.equal((await admin('/admin/access')).status,401);
    const withKey=(suffix,options={})=>request(apiBase()+suffix,{...options,headers:{Authorization:'Bearer '+managementKey,...options.headers}});
    assert.equal((await withKey('/admin/access')).status,200);
    const saved=await fs.readFile(accessPath,'utf8');assert(!saved.includes(managementKey));assert(saved.includes('pbkdf2$'));
    assert.equal((await withKey('/admin/llm/session',{method:'POST',headers:{Origin:apiBase()}})).status,403);
    assert.equal(JSON.parse((await withKey('/debug/llm')).body).management_mode,'token');
    await page.locator('#cfg-tls summary').click();await page.locator('#cfg-tls-enable').check();
    assert.equal(await page.locator('#cfg-tls-fields').isVisible(),true);
    const cert=path.join(temporary,'cert.pem'),key=path.join(temporary,'key.pem');
    execFileSync('openssl',['req','-x509','-newkey','rsa:2048','-nodes','-keyout',key,'-out',cert,'-days','1','-subj','/CN=localhost'],{stdio:'ignore'});
    await page.locator('#cfg-tls-cert').fill(cert);await page.locator('#cfg-tls-key').fill(key);
    const newPort=await port(), newDirectory=path.join(temporary,'accounts-new');
    await page.locator('#cfg-port').fill(String(newPort));await page.locator('#cfg-auth-dir').fill(newDirectory);await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving && !configurationState.dirty);
    const pending=JSON.parse((await withKey('/admin/access')).body);assert(pending.settings.restart_required);assert.equal(pending.settings.active.port,proxyPort);assert.equal(pending.settings.active.tls,false);
    await page.locator('#cfg-host').fill('127.0.0.2');await page.locator('#lang-toggle').click();assert.equal(await page.locator('#cfg-host').inputValue(),'127.0.0.2');assert.match(await page.locator('#cfg-version').textContent(),/^v\d/);assert.equal(await page.locator('.cfg-dot.connected').count(),1);
    await page.locator('#cfg-host').fill('127.0.0.1');
    if(artifact) {
      await fs.mkdir(artifact,{recursive:true});
      await page.locator('#configuration-workspace').screenshot({path:path.join(artifact,'configuration-desktop.png')});
      await page.locator('#theme-toggle').click();await page.locator('#configuration-workspace').screenshot({path:path.join(artifact,'configuration-other-theme.png')});
      await page.setViewportSize({width:390,height:1000});await page.locator('#configuration-workspace').screenshot({path:path.join(artifact,'configuration-mobile.png')});
    }
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
    await page.setViewportSize({width:1440,height:1000});
    await stop();scheme='https';proxyPort=newPort;await start();
    assert.equal((await withKey('/admin/access')).status,200);
    const restarted=JSON.parse((await withKey('/admin/access')).body);assert.equal(restarted.settings.restart_required,false);assert.equal(restarted.settings.active.tls,true);assert.equal(restarted.settings.active.auth_dir,newDirectory);assert((await fs.stat(newDirectory)).isDirectory());assert((await fs.stat(authDir)).isDirectory());
    const panel=await request(apiBase()+'/');assert(panel.body.includes('name="transit-management-api" content="1"'));assert(!panel.body.includes('src="/assets/configuration.js"'));
    assert.equal((await request('https://127.0.0.1:'+proxyPort+'/echo',{headers:{'x-goog-api-key':'gateway-client-updated-5678','x-route-key':'route-secret'}})).status,200);
    await page.goto(apiBase()+'/#configuration');await page.locator('#cfg-connect').waitFor({state:'visible'});await page.locator('#cfg-connect').click();await page.locator('dialog input[name=key]').fill(managementKey);await page.locator('dialog button[type=submit]').click();await page.locator('#cfg-form').waitFor();
    await page.locator('[data-action=delete]').click();await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving && !configurationState.dirty);
    assert.equal((await request('https://127.0.0.1:'+proxyPort+'/echo',{headers:{'x-route-key':'route-secret'}})).status,200);
    await page.locator('#cfg-remote summary').click();await page.locator('#cfg-allow-remote').uncheck();await page.locator('#cfg-clear-secret').check();await page.locator('#cfg-save').click();await page.waitForFunction(()=>!configurationState.saving && !configurationState.dirty);
    assert.equal((await withKey('/admin/access')).status,401);
    const localSession=await request(apiBase()+'/admin/session',{method:'POST',headers:{Origin:apiBase()}});assert.equal(localSession.status,200);
    const localToken=JSON.parse(localSession.body).token;
    const finalView=JSON.parse((await request(apiBase()+'/admin/access',{headers:{Authorization:'Bearer '+localToken}})).body);
    finalView.settings.config['remote-management']['disable-control-panel']=true;
    assert.equal((await request(apiBase()+'/admin/access',{method:'PUT',headers:{Authorization:'Bearer '+localToken,'Content-Type':'application/json'},body:JSON.stringify({revision:finalView.settings.revision,config:finalView.settings.config})})).status,200);
    assert.equal((await request(apiBase()+'/')).status,404);assert.equal((await request(apiBase()+'/assets/configuration.js')).status,404);
    assert.equal((await request(apiBase()+'/healthz')).status,200);assert.equal((await request(apiBase()+'/admin/access',{headers:{Authorization:'Bearer '+localToken}})).status,200);
    assert.deepEqual(errors,[]);
    console.log('PASS: configuration CRUD, concurrent editing, protected management, key rotation, client auth and routing, restart persistence, real HTTPS, directory switching, panel disable, responsive UI and keyboard/language state.');
  } catch(error) {
    if(page) { const message=await page.locator('#cfg-error').textContent().catch(()=>''); if(message) console.error(message); if(artifact) await page.screenshot({path:path.join(artifact,'configuration-failure.png'),fullPage:true}).catch(()=>{}); }
    throw error;
  } finally { if(browser) await browser.close();if(gateway&&gateway.exitCode===null){const exit=once(gateway,'exit');gateway.kill('SIGTERM');await exit;}await new Promise(r=>upstream.close(r));await fs.rm(temporary,{recursive:true,force:true}); }
})().catch(error=>{console.error(error.stack);process.exitCode=1;});
