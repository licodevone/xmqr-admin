// SPDX-License-Identifier: MIT
// Real local admin/monitor/CLI fixture, plus explicitly identified browser faults.
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
const {chromium}=createRequire(import.meta.url)('playwright');
const [handoff,stop,output,distro='Ubuntu-26.04']=process.argv.slice(2);
if(!handoff||!stop||!output) throw new Error('uso: HANDOFF STOP PASTA_NOVA [WSL_DISTRO]');
if(fs.existsSync(output)) throw new Error('a pasta de evidencias deve ser nova');
fs.mkdirSync(output,{recursive:true});
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const rows=[],errors=[];
let browser,current='inicializacao',fixture;
try {
  for(let attempt=0;attempt<100&&!fs.existsSync(handoff);attempt++) await delay(100);
  fixture=JSON.parse(fs.readFileSync(handoff,'utf8'));
  const owned=/^\/var\/tmp\/xmqr-admin-a25-browser-[^/]+\/(access\.token|monitor\.control|client\.control)$/;
  for(const field of ['token_file','monitor_control','client_control']) assert.match(fixture[field],owned);
  const localPath=file=>process.platform==='win32'?`\\\\wsl.localhost\\${distro}${file.replaceAll('/','\\')}`:file;
  const token=fs.readFileSync(localPath(fixture.token_file),'utf8').trim();
  assert.match(token,/^[0-9a-f]{64}$/);
  const control=(name,value)=>fs.writeFileSync(localPath(fixture[name]),value);
  browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{})});
  const context=await browser.newContext({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
  const page=await context.newPage();
  page.on('pageerror',()=>errors.push('excecao JS'));
  page.on('dialog',async dialog=>{errors.push('HTML remoto executado');await dialog.dismiss();});
  let activeStatus=0,maxActiveStatus=0;
  page.on('request',request=>{if(new URL(request.url()).pathname==='/api/status') {activeStatus++;maxActiveStatus=Math.max(maxActiveStatus,activeStatus);}});
  const finished=request=>{if(new URL(request.url()).pathname==='/api/status') activeStatus--;};
  page.on('requestfinished',finished);page.on('requestfailed',finished);
  const state=async(name,value,timeout=10000)=>page.waitForFunction(({name,value})=>document.getElementById('system-'+name)?.dataset.state===value,{name,value},{timeout});
  const allOnline=async()=>{for(const name of ['admin','broker','client']) await state(name,'online');};
  const unconfirmed=async(timeout=10000)=>{
    await state('admin','offline',timeout);await state('broker','offline',timeout);await state('client','offline',timeout);
    assert.match(await page.locator('#system-broker-detail').textContent(),/Sem confirmação/i);
    assert.match(await page.locator('#system-client-detail').textContent(),/Sem confirmação/i);
    assert.match(await page.locator('#system-broker-detail').textContent(),/não confirma.*parado/i);
  };
  const idle=async()=>page.waitForFunction(()=>!document.getElementById('refresh').disabled,undefined,{timeout:12000});
  const refresh=async()=>{await idle();await page.locator('#refresh').click();};
  current='estados iniciais sem inferir disponibilidade pelo HTML';
  await page.goto(fixture.url);await page.waitForLoadState('networkidle');
  await page.locator('#login-form').waitFor({state:'visible'});
  for(const name of ['admin','broker','client']) {
    await state(name,'unknown');
    assert.match(await page.locator(`#system-${name}-status`).textContent(),/Aguardando/i);
  }
  rows.push(current);
  current='login real e agregado HTTP do admin com client ainda desconectado';
  await page.locator('#token').fill(token);
  await page.getByRole('button',{name:'Desbloquear painel',exact:true}).click();
  await page.locator('#workspace').waitFor({state:'visible'});
  await state('admin','online');await state('broker','online');await state('client','offline');
  assert.match(await page.locator('#system-admin-status').textContent(),/ON.LINE/i);
  assert.match(await page.locator('#system-client-status').textContent(),/OFF.LINE/i);
  rows.push(current);
  current='conexao permanece OFF-LINE ate o marcador SUBACK do processo real filho';
  await page.locator('.nav[data-view="console"]').click();
  await page.locator('#subscribe-topic').fill('a25/#');
  await page.locator('#connect').click();
  await page.waitForFunction(()=>document.getElementById('connection-state').dataset.state==='connecting');
  await state('client','offline');
  assert.match(await page.locator('#system-client-detail').textContent(),/confirma|conectando/i);
  control('client_control','connected');
  await state('client','online');
  await page.locator('.nav[data-view="overview"]').click();
  await allOnline();await page.screenshot({path:path.join(output,'systems-desktop.png'),fullPage:true});
  rows.push(current);
  current='HTTP real do monitor distingue nao pronto e indisponivel sem derrubar admin ou assinatura';
  control('monitor_control','not_ready');await refresh();await state('broker','offline');
  await state('admin','online');await state('client','online');
  assert.match(await page.locator('#system-broker-detail').textContent(),/pront|pronto/i);
  control('monitor_control','drop');await refresh();await state('broker','offline');
  await page.waitForFunction(()=>/monitor/i.test(document.getElementById('system-broker-detail').textContent));
  control('monitor_control','ready');await refresh();await allOnline();
  rows.push(current);

  const sampleResponse=await context.request.get(fixture.url+'/api/status',{headers:{Authorization:'Bearer '+token}});
  assert.equal(sampleResponse.status(),200);
  const valid=await sampleResponse.json();
  let mode='real',mqttMode='real',releaseStatus,releaseMqtt,heldStatus=false,heldMqtt=false;
  await page.route('**/api/status',async route=>{
    const selected=mode;
    if(selected==='real') return route.continue();
    if(selected==='hold') {heldStatus=true;await new Promise(resolve=>{releaseStatus=resolve;});}
    if(selected==='slow') await delay(6500);
    try {
      if(selected==='fail') return await route.fulfill({status:500,contentType:'application/json',body:'{"error":"fixture"}'});
      if(selected==='invalid') return await route.fulfill({json:{...valid,systems:{...valid.systems,admin:{online:true,reason:'<img src=x onerror=alert(1)>'}}}});
      if(selected==='metrics-string') return await route.fulfill({json:{...valid,metrics:{...valid.metrics,xmqr_connections_active:'1'}}});
      if(selected==='readiness-two') return await route.fulfill({json:{...valid,metrics:{...valid.metrics,xmqr_ready:2}}});
      if(selected==='malformed') return await route.fulfill({contentType:'application/json',body:'{'});
      return await route.fulfill({json:valid});
    } catch { /* The intended client AbortController may already have cancelled. */ }
  });
  await page.route('**/api/mqtt',async route=>{
    const selected=mqttMode;
    if(selected==='hold') {heldMqtt=true;await new Promise(resolve=>{releaseMqtt=resolve;});}
    if(selected==='fail') return route.fulfill({status:500,contentType:'application/json',body:'{"error":"fixture"}'});
    return route.continue();
  });
  current='falha de status invalida imediatamente mesmo com consulta MQTT pendente (interceptacao)';
  mode='fail';mqttMode='hold';await refresh();
  await page.waitForFunction(()=>document.getElementById('system-admin').dataset.state==='offline',undefined,{timeout:1500});
  for(let n=0;n<100&&!heldMqtt;n++) await delay(10);
  assert(heldMqtt);await unconfirmed(1500);
  assert.equal(await page.locator('#refresh').isDisabled(),true);
  releaseMqtt();await idle();mode='real';mqttMode='real';await refresh();await allOnline();
  rows.push(current);
  current='falha isolada na API MQTT nao substitui agregado de status valido (interceptacao)';
  mqttMode='fail';await refresh();await idle();await allOnline();mqttMode='real';
  rows.push(current);
  for(const [fault,label] of [['invalid','estrutura invalida e texto remoto sem HTML'],['metrics-string','metrica numerica recebida como string'],['readiness-two','readiness fora de 0 ou 1'],['malformed','JSON invalido'],['slow','timeout real do AbortController']]) {
    current=label+' invalidam indicadores e permitem recuperacao (interceptacao)';
    mode=fault;await refresh();await unconfirmed();
    assert.equal(await page.locator('#system-admin img,#system-client img,#system-broker img').count(),0);
    assert.equal(await page.locator('[data-metric="xmqr_connections_active"]').textContent(),'—');
    await idle();mode='real';await refresh();await allOnline();rows.push(current);
  }
  current='evento offline invalida imediatamente e resposta anterior nao restaura estado antigo (evento controlado)';
  mode='hold';heldStatus=false;await refresh();
  for(let n=0;n<100&&!heldStatus;n++) await delay(10);
  assert(heldStatus);
  await page.evaluate(()=>window.dispatchEvent(new Event('offline')));await unconfirmed(1500);
  releaseStatus();await idle();await unconfirmed();
  mode='real';await page.evaluate(()=>window.dispatchEvent(new Event('online')));await allOnline();
  rows.push(current);
  current='rede offline real do contexto Chromium e recuperacao';
  await context.setOffline(true);await unconfirmed(1500);
  await context.setOffline(false);await allOnline();rows.push(current);
  current='falha real do processo filho reflete client OFF-LINE sem invalidar admin e broker';
  control('client_control','exit');await refresh();await state('client','offline');
  await page.waitForFunction(()=>/falh|erro/i.test(document.getElementById('system-client-detail').textContent));
  await state('admin','online');await state('broker','online');rows.push(current);
  current='desktop e mobile sem overflow e indicadores com texto alem de cor';
  await page.setViewportSize({width:390,height:844});
  for(const name of ['admin','broker','client']) {
    const text=await page.locator(`#system-${name}-status`).textContent();
    assert.match(text,/ON.LINE|OFF.LINE/i);
  }
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
  await page.screenshot({path:path.join(output,'systems-mobile.png'),fullPage:true});rows.push(current);
  current='logout limpa estados e segredos sem consultas sobrepostas';
  await page.locator('#logout').click();await page.locator('#login-form').waitFor({state:'visible'});
  for(const name of ['admin','broker','client']) await state(name,'unknown');
  assert.deepEqual(await page.evaluate(()=>[localStorage.length,sessionStorage.length]),[0,0]);
  assert.equal(await page.locator('#token').inputValue(),'');
  assert(!page.url().includes('?'));
  assert(maxActiveStatus<=1,'consultas de status sobrepostas');
  assert.deepEqual(errors,[]);rows.push(current);
  fs.writeFileSync(path.join(output,'browser.json'),JSON.stringify({status:'PASS',steps:rows,
    hashes:{...fixture.hashes,browser_fixture_sha256:createHash('sha256').update(fs.readFileSync(new URL(import.meta.url))).digest('hex')},
    scope:'Admin e HTTP de monitor reais, CLI deterministica privada; falhas de resposta e eventos explicitamente identificados. Sem broker MQTT real neste ensaio.',
    browser:'Chromium/Playwright',max_parallel_status: maxActiveStatus},null,2),{flag:'wx'});
  console.log('A25_BROWSER_PASS',rows.length);
} catch(error) {
  fs.writeFileSync(path.join(output,'browser.json'),JSON.stringify({status:'FAIL',steps:rows,failed_step:current,error_type:error.name,hashes:fixture?.hashes},null,2),{flag:'wx'});
  console.error('A25_BROWSER_FAIL',current,error.name);process.exitCode=1;
} finally {
  if(browser) await browser.close();
  if(!fs.existsSync(stop)) fs.writeFileSync(stop,'stop',{flag:'wx'});
}
