// SPDX-License-Identifier: MIT
import {createRequire} from 'node:module';
import {randomBytes} from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
const {chromium}=createRequire(import.meta.url)('playwright');
const [handoff,stop,output]=process.argv.slice(2);
if(!handoff||!stop||!output) throw new Error('uso: HANDOFF STOP PASTA_NOVA');
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
for(let n=0;n<100&&!fs.existsSync(handoff);n++) await delay(200);
const fixture=JSON.parse(fs.readFileSync(handoff,'utf8'));
assert.match(fixture.token_file,/^\/var\/tmp\/xmqr-admin-a24-browser-[^/]+\/access\.token$/);
const tokenPath=process.platform==='win32'?`\\\\wsl.localhost\\Ubuntu-26.04${fixture.token_file.replaceAll('/','\\')}`:fixture.token_file;
const token=fs.readFileSync(tokenPath,'utf8').trim();
const password=randomBytes(24).toString('hex');
fs.mkdirSync(output,{recursive:true});
const rows=[],errors=[];
let browser;
try {
  browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{})});
  const context=await browser.newContext({viewport:{width:1440,height:1050},reducedMotion:'reduce'});
  const page=await context.newPage();
  page.on('pageerror',()=>errors.push('excecao JS'));
  page.on('dialog',async dialog=>{errors.push('HTML remoto executado');await dialog.dismiss();});
  await page.goto(fixture.url);await page.waitForLoadState('networkidle');
  await page.locator('#setup-form').waitFor({state:'visible'});
  await page.screenshot({path:path.join(output,'setup.png'),fullPage:true});
  await page.locator('#setup-token').fill(token);
  await page.locator('#setup-username').fill('adminqa');
  await page.locator('#setup-display-name').fill('Administrador QA');
  await page.locator('#setup-password').fill(password);
  await page.locator('#setup-password-confirm').fill(password);
  await page.locator('#setup-submit').click();
  await page.locator('#workspace').waitFor({state:'visible'});
  assert.equal(await page.locator('#setup-password').inputValue(),'');
  assert.equal(await page.locator('#setup-token').inputValue(),'');
  rows.push('bootstrap unico e entrada do administrador sem expor senha/codigo');
  const cookie=(await context.cookies()).find(item=>item.name==='xmqr_session');
  assert(cookie?.httpOnly&&cookie.sameSite==='Strict');
  await page.getByRole('button',{name:'Usuários e acesso',exact:true}).click();
  for(const [username,role,name] of [['viewerqa','viewer','Consulta <img src=x onerror=alert(1)>'],['operatorqa','operator','Operador QA']]) {
    await page.locator('#new-username').fill(username);
    await page.locator('#new-display-name').fill(name);
    await page.locator('#new-role').selectOption(role);
    await page.locator('#new-password').fill(password);
    await page.locator('#new-password-confirm').fill(password);
    await page.locator('#create-user').click();
    await page.waitForFunction(name=>document.querySelector('#users-list').textContent.includes(name),username);
  }
  assert.equal(await page.locator('#users-list img').count(),0);
  rows.push('administrador cria contas com papeis e nomes renderizados sem HTML');
  await page.evaluate(()=>window.scrollTo(0,0));
  await page.screenshot({path:path.join(output,'users-desktop.png'),fullPage:true});
  async function logout() {
    await page.locator('#logout').click();
    await page.locator('#user-login-form').waitFor({state:'visible'});
  }
  async function login(username) {
    await page.locator('#login-username').fill(username);
    await page.locator('#login-password').fill(password);
    await page.locator('#user-login-submit').click();
    await page.locator('#workspace').waitFor({state:'visible'});
  }
  await logout();
  await page.locator('#login-username').fill('viewerqa');
  await page.locator('#login-password').fill(randomBytes(24).toString('hex'));
  await page.locator('#user-login-submit').click();
  await page.locator('#login-message.error').waitFor();
  rows.push('senha incorreta recusada com erro generico');
  await login('viewerqa');
  assert.equal(await page.locator('.nav[data-view="console"]').isVisible(),false);
  assert.equal(await page.locator('.nav[data-view="users"]').isVisible(),false);
  await page.reload();await page.waitForLoadState('networkidle');
  await page.locator('#workspace').waitFor({state:'visible'});
  assert.equal(await page.locator('.nav[data-view="console"]').isVisible(),false);
  rows.push('consulta restrita e sessao restaurada apos refresh');
  await logout();await login('operatorqa');
  assert.equal(await page.locator('.nav[data-view="console"]').isVisible(),true);
  assert.equal(await page.locator('.nav[data-view="users"]').isVisible(),false);
  rows.push('operador acessa console sem gestao de usuarios');
  await logout();await login('adminqa');
  await page.getByRole('button',{name:'Usuários e acesso',exact:true}).click();
  await page.getByRole('button',{name:'Desativar usuário viewerqa',exact:true}).click();
  await page.getByRole('button',{name:'Ativar usuário viewerqa',exact:true}).waitFor();
  await page.setViewportSize({width:390,height:844});
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
  await page.screenshot({path:path.join(output,'users-mobile.png'),fullPage:true});
  rows.push('desativacao de conta e layout mobile sem overflow');
  await logout();
  assert.equal((await context.cookies()).some(item=>item.name==='xmqr_session'),false);
  await Promise.all([
    page.waitForResponse(response=>new URL(response.url()).pathname==='/api/auth/session'&&response.status()===401),
    page.reload(),
  ]);
  await page.locator('#user-login-form').waitFor({state:'visible'});
  assert.equal(await page.locator('#workspace').isVisible(),false);
  assert.deepEqual(await page.evaluate(()=>[localStorage.length,sessionStorage.length]),[0,0]);
  assert(!page.url().includes('?'));
  assert.deepEqual(errors,[]);
  rows.push('logout revoga sessao e nenhum segredo persiste em storage ou URL');
  fs.writeFileSync(path.join(output,'browser.json'),JSON.stringify({status:'PASS',steps:rows,backend:'servidor A24 e contas reais descartaveis; nenhum dado do operador',browser:'Chromium/Playwright'},null,2));
  console.log('A24_BROWSER_PASS',rows.length);
} catch(error) {
  fs.writeFileSync(path.join(output,'browser.json'),JSON.stringify({status:'FAIL',steps:rows,error:String(error)},null,2));throw error;
} finally {if(browser) await browser.close();fs.writeFileSync(stop,'stop');}
