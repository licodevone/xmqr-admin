"use strict";
let accessToken = "";
let authMode = "loading";
let setupRequired = false;
let sessionInfo = null;
let authPending = false;
let usersPending = false;
let usersMutationPending = false;
let generation = 0;
let pending = false;
let connectionPending = false;
let publishPending = false;
let leaving = false;
let pollTimer = null;
let paused = false;
let localMessages = [];
const seenMessages = new Set();
let mqttState = {enabled:false,status:"disabled",label:""};
const byId = id => document.getElementById(id);
const titles = {overview:"Visão geral",console:"Console MQTT",operations:"Operações administrativas",settings:"Configuração e acesso",users:"Usuários e acesso"};
const roleNames={viewer:"Observador",operator:"Operador",admin:"Administrador"};
const hasSession=()=>authMode==="users"?sessionInfo!==null:authMode==="token"&&accessToken!=="";
const canMqtt=()=>hasSession()&&(authMode==="token"||["operator","admin"].includes(sessionInfo.role));
const isAdmin=()=>authMode==="users"&&sessionInfo?.role==="admin";
const allowedView=view=>view==="overview"||(hasSession()&&(view==="users"?isAdmin():canMqtt()));
const stateNames = {disabled:"Não configurado",disconnected:"Desconectado",connecting:"Conectando",connected:"Assinatura ativa",error:"Falha na conexão"};
const setFeedback = (id,text,tone="") => {const node=byId(id);node.textContent=text;node.className="feedback"+(tone?" "+tone:"");};
const message = (text,error=false) => {byId("login-message").textContent=text;byId("login-message").classList.toggle("error",error);};

function showView(view) {
  if (!Object.hasOwn(titles,view)||!allowedView(view)) return;
  for (const section of document.querySelectorAll(".view")) section.hidden=section.id!==view;
  for (const button of document.querySelectorAll(".nav[data-view]")) {
    const active=button.dataset.view===view;
    button.classList.toggle("active",active);
    if(active) button.setAttribute("aria-current","page");else button.removeAttribute("aria-current");
  }
  byId("page-title").textContent=titles[view];
  if(view==="users") void loadUsers();
}

function updateControls() {
  const enabled=canMqtt() && mqttState.enabled===true && !leaving;
  const active=["connecting","connected"].includes(mqttState.status);
  byId("connect").disabled=!enabled||active||connectionPending;
  byId("disconnect").disabled=!enabled||(!active&&mqttState.status!=="error")||connectionPending;
  byId("subscribe-topic").disabled=!enabled||active||connectionPending;
  byId("subscribe-qos").disabled=!enabled||active||connectionPending;
  byId("publish").disabled=!enabled||publishPending;
  byId("publish").textContent=publishPending?"Publicando…":"Publicar mensagem";
}

function authForms() {
  byId("login-form").hidden=authMode!=="token";
  byId("user-login-form").hidden=authMode!=="users"||setupRequired;
  byId("setup-form").hidden=authMode!=="users"||!setupRequired;
  byId("login-description").textContent=authMode==="token"?"Consulte a operação e teste mensagens em um único lugar. Desbloqueie o painel com seu token local.":authMode==="users"?(setupRequired?"Defina o administrador que poderá autorizar outras pessoas nesta estação.":"Entre com sua conta para acessar as funções autorizadas para você."):"Verificando o modo de acesso desta estação.";
}

function updatePermissions() {
  for(const button of document.querySelectorAll(".nav[data-view]")) button.hidden=!allowedView(button.dataset.view);
  byId("console-next-action").hidden=!canMqtt();
  byId("identity").hidden=!hasSession();
  byId("identity-name").textContent=authMode==="users"&&sessionInfo?sessionInfo.display_name:"Acesso por token";
  byId("identity-role").textContent=authMode==="users"&&sessionInfo?roleNames[sessionInfo.role]:"Token local";
  const active=document.querySelector(".nav.active");
  if(active&&!allowedView(active.dataset.view)) showView("overview");
  updateControls();
}

function lock() {
  generation++;accessToken="";sessionInfo=null;
  clearTimeout(pollTimer);
  for(const id of ["token","login-password","setup-token","setup-password","setup-password-confirm","new-password","new-password-confirm"]) byId(id).value="";
  byId("workspace").hidden=true;byId("login-section").hidden=false;
  for(const id of ["publish-payload","publish-topic","subscribe-topic","message-search"]) byId(id).value="";
  byId("users-list").replaceChildren();
  byId("create-user-form").reset();
  setFeedback("create-user-message","A senha não será exibida nem recuperada depois.");
  setFeedback("publish-message","A publicação usa o perfil autorizado no servidor.");
  for(const item of document.querySelectorAll("[data-metric]")) item.textContent="—";
  byId("commit-total").textContent="Commits duráveis: —";byId("sample-time").textContent="Nenhuma leitura realizada";
  for(const [id,label] of [["received-total","Recebidas"],["published-total","Publicadas"],["dropped-total","Descartadas do buffer"]]) byId(id).textContent=label+": —";
  localMessages=[];seenMessages.clear();paused=false;
  byId("pause").textContent="Pausar exibição";byId("pause").setAttribute("aria-pressed","false");byId("pause-note").hidden=true;
  mqttState={enabled:false,status:"disabled",label:""};renderMessages();authForms();updatePermissions();showView("overview");
  const focus=authMode==="token"?"token":setupRequired?"setup-token":"login-username";
  if(authMode!=="loading") byId(focus).focus();
}

function renderStatus(data) {
  const available=data.reachable===true;
  const ready=available&&data.ready===true;
  const metrics=data.metrics&&typeof data.metrics==="object"?data.metrics:{};
  byId("availability").dataset.state=ready?"ready":available?"unready":"offline";
  byId("overview-title").textContent=ready?"Broker pronto para operar":available?"Monitor ativo, broker não está pronto":"Monitor do broker indisponível";
  byId("broker-description").textContent=ready?"O ator e o armazenamento responderam à verificação.":available?"Verifique a instância do broker. O painel não reinicia serviços automaticamente.":"Sem leitura válida. Confira o monitor opcional e seu endereço local.";
  byId("monitor-help").hidden=available;
  for(const item of document.querySelectorAll("[data-metric]")) item.textContent=available&&Object.hasOwn(metrics,item.dataset.metric)?String(metrics[item.dataset.metric]):"—";
  byId("commit-total").textContent="Commits duráveis: "+(available&&Object.hasOwn(metrics,"xmqr_persistence_commits_total")?String(metrics.xmqr_persistence_commits_total):"—");
  byId("sample-time").textContent="Última consulta: "+new Date().toLocaleTimeString("pt-BR");
  if(typeof data.admin_version==="string") {byId("version-sidebar").textContent="Versão "+data.admin_version;byId("version-footer").textContent=data.admin_version;}
}

function emptyMessages(searching) {
  const empty=document.createElement("div");empty.className="empty-state";
  const icon=document.createElement("span");icon.setAttribute("aria-hidden","true");icon.textContent="⇄";
  const title=document.createElement("h4");title.textContent=searching?"Nenhuma mensagem encontrada":"Nenhuma mensagem por aqui";
  const description=document.createElement("p");description.textContent=searching?"Ajuste a busca para encontrar outro tópico ou conteúdo.":"Conecte uma assinatura e publique no mesmo tópico a partir do ESP32 ou de outro client.";
  empty.append(icon,title,description);return empty;
}

function renderMessages() {
  if(paused) return;
  const list=byId("messages");
  const search=byId("message-search").value.toLocaleLowerCase("pt-BR");
  const visible=localMessages.filter(item=>(String(item.topic)+" "+String(item.payload)).toLocaleLowerCase("pt-BR").includes(search));
  const fragment=document.createDocumentFragment();
  for(const item of [...visible].reverse()) {
    const row=document.createElement("article");row.className="message-item"+(item.direction==="published"?" published":"");
    const meta=document.createElement("div");meta.className="message-meta";
    const direction=document.createElement("span");direction.className="message-direction";direction.textContent=item.direction==="published"?"Publicada":"Recebida";
    const time=document.createElement("time");const date=new Date(Number(item.received_at));
    time.textContent=Number.isNaN(date.getTime())?"Horário indisponível":date.toLocaleTimeString("pt-BR");
    if(!Number.isNaN(date.getTime())) {time.dateTime=date.toISOString();time.title=date.toLocaleString("pt-BR");}
    const qos=document.createElement("span");qos.textContent="QoS "+String(item.qos);
    meta.append(direction,time,qos);
    if(item.retain===true) {const retain=document.createElement("span");retain.className="retained-label";retain.textContent="Retained";meta.append(retain);}
    const topic=document.createElement("p");topic.className="message-topic";topic.textContent=String(item.topic);
    const payload=document.createElement("pre");payload.className="message-payload";payload.textContent=typeof item.payload==="string"?(item.payload===""?"(conteúdo vazio)":item.payload):"[Conteúdo binário]";
    row.append(meta,topic,payload);fragment.append(row);
  }
  if(!visible.length) fragment.append(emptyMessages(!!search));
  list.replaceChildren(fragment);
  byId("message-count").textContent=search?visible.length+" de "+localMessages.length+" mensagens":localMessages.length+" mensagens nesta página";
}

function renderMqtt(data) {
  mqttState={...data,enabled:data.enabled===true};
  const status=Object.hasOwn(stateNames,data.status)?data.status:"error";
  mqttState.status=status;
  byId("connection-state").textContent=stateNames[status];
  byId("connection-state").dataset.state=status;
  byId("console-disabled").hidden=mqttState.enabled;
  byId("console-tools").hidden=!mqttState.enabled;
  const label=typeof data.label==="string"&&data.label?data.label:"Broker configurado no servidor";
  byId("mqtt-label").textContent=mqttState.enabled?label:"Console não configurado";
  byId("connection-detail").textContent=label+(data.topic?" · Filtro: "+String(data.topic):" · Uma assinatura contínua por painel local.");
  if(status==="connected") setFeedback("console-message","Assinatura confirmada pelo broker. Aguardando e recebendo mensagens.","success");
  else if(status==="connecting") setFeedback("console-message","Aguardando a confirmação da assinatura pelo broker.");
  else if(status==="error") setFeedback("console-message","A conexão MQTT falhou. Confira o broker, as permissões e o perfil privado no servidor.","error");
  else if(status==="disabled") setFeedback("console-message","Prepare os perfis privados no servidor para habilitar o console.");
  else setFeedback("console-message","Desconectado. Informe um filtro e conecte para receber mensagens.");
  if(["connected","connecting"].includes(status)&&typeof data.topic==="string") {byId("subscribe-topic").value=data.topic;if([0,1,2].includes(data.qos)) byId("subscribe-qos").value=String(data.qos);}
  for(const [key,labelText] of [["received_total","Recebidas"],["published_total","Publicadas"],["dropped_total","Descartadas do buffer"]]) {
    const id={received_total:"received-total",published_total:"published-total",dropped_total:"dropped-total"}[key];
    byId(id).textContent=labelText+": "+(Number.isSafeInteger(data[key])&&data[key]>=0?data[key]:"—");
  }
  let changed=false;
  for(const item of Array.isArray(data.messages)?data.messages.slice(-200):[]) {
    if(!item||typeof item!=="object"||item.id===undefined) continue;
    const key=String(item.id);
    if(seenMessages.has(key)) continue;
    seenMessages.add(key);localMessages.push(item);changed=true;
  }
  if(localMessages.length>200) localMessages=localMessages.slice(-200);
  while(seenMessages.size>800) seenMessages.delete(seenMessages.values().next().value);
  if(changed) renderMessages();
  updateControls();
}

async function api(path,options={}) {
  const epoch=generation,token=accessToken,csrf=sessionInfo?.csrf_token;
  if((!options.public&&!hasSession())||(leaving&&!options.allowLeaving)) throw new Error("no_session");
  const controller=new AbortController();const timer=setTimeout(()=>controller.abort(),options.timeout||8000);
  const headers={};
  if(options.body) headers["Content-Type"]="application/json";
  if(!options.public) {
    if(authMode==="token") headers.Authorization="Bearer "+token;
    else if(options.body) headers["X-XMQR-CSRF"]=csrf;
  }
  try {
    const response=await fetch(path,{method:options.body?"POST":"GET",headers,body:options.body?JSON.stringify(options.body):undefined,credentials:"same-origin",cache:"no-store",signal:controller.signal});
    if(epoch!==generation) throw new Error("stale_session");
    if((response.status===401||response.status===403)&&!options.public) {
      lock();message(authMode==="users"?"Sua sessão expirou ou o acesso foi revogado. Entre novamente.":"Token recusado ou origem não autorizada. Confira o arquivo privado.",true);
      const error=new Error("unauthorized");error.status=response.status;throw error;
    }
    if(!response.ok) {const error=new Error("request_failed");error.status=response.status;throw error;}
    if(response.status===204) return null;
    const text=await response.text();if(epoch!==generation) throw new Error("stale_session");
    return text?JSON.parse(text):null;
  } finally {clearTimeout(timer);}
}

function schedulePoll() {clearTimeout(pollTimer);if(hasSession()&&!leaving) pollTimer=setTimeout(refresh,3000);}

async function refresh() {
  if(pending||!hasSession()||leaving) return false;
  pending=true;byId("refresh").disabled=true;const epoch=generation;
  try {
    const tasks=[api("/api/status",{timeout:5000})];if(canMqtt()) tasks.push(api("/api/mqtt",{timeout:5000}));
    const results=await Promise.allSettled(tasks);
    if(epoch!==generation||!hasSession()||leaving) return false;
    const status=results[0],mqtt=results[1];
    if(status.status==="fulfilled"&&status.value) {
      renderStatus(status.value);byId("login-section").hidden=true;byId("workspace").hidden=false;
      message(authMode==="users"?"Sessão autenticada nesta estação.":"O token fica apenas na memória desta página.");
    } else if(byId("workspace").hidden) {message("Não foi possível consultar o painel local. Confira se ele está em execução.",true);return false;}
    else renderStatus({reachable:false,ready:false,metrics:{}});
    if(canMqtt()) {
      if(mqtt?.status==="fulfilled"&&mqtt.value) renderMqtt(mqtt.value);
      else {byId("connection-state").textContent="Consulta indisponível";byId("connection-state").dataset.state="error";setFeedback("console-message","Não foi possível consultar o console. Verifique o servidor; a interface não repete publicações.","error");}
    }
    updatePermissions();return true;
  } finally {pending=false;byId("refresh").disabled=false;schedulePoll();}
}

function acceptSession(data) {
  if(!data||typeof data.username!=="string"||typeof data.display_name!=="string"||!Object.hasOwn(roleNames,data.role)||typeof data.csrf_token!=="string"||!data.csrf_token) throw new Error("invalid_session");
  generation++;accessToken="";sessionInfo={username:data.username,display_name:data.display_name,role:data.role,csrf_token:data.csrf_token};setupRequired=false;
  authForms();updatePermissions();showView("overview");
}

function setAuthBusy(busy) {
  authPending=busy;for(const id of ["user-login-submit","setup-submit","retry-auth"]) byId(id).disabled=busy;
}

async function initializeAuth() {
  if(authPending||leaving) return;
  setAuthBusy(true);byId("retry-auth").hidden=true;message("Consultando a configuração de acesso…");
  try {
    const config=await api("/api/auth/config",{public:true,timeout:5000});
    if(!config||!["token","users"].includes(config.mode)||typeof config.setup_required!=="boolean") throw new Error("invalid_config");
    authMode=config.mode;setupRequired=config.mode==="users"&&config.setup_required;authForms();updatePermissions();
    if(authMode==="users"&&!setupRequired) {
      try {const info=await api("/api/auth/session",{public:true,timeout:5000});acceptSession(info);await refresh();return;}
      catch(error) {if(error.status!==401) {message("Não foi possível restaurar a sessão. Entre novamente ou tente atualizar a página.",true);return;}}
    }
    message(authMode==="token"?"O token fica apenas na memória desta página.":setupRequired?"Use seu código local para autorizar a criação do primeiro administrador.":"Informe seu usuário e senha do painel.");
  } catch(_) {message("Não foi possível consultar a configuração de acesso. Verifique o servidor e tente novamente.",true);byId("retry-auth").hidden=false;}
  finally {setAuthBusy(false);}
}

const byteLength=value=>new TextEncoder().encode(value).length;
function validAccount(username,displayName,password,confirmation) {
  if(!/^[a-zA-Z0-9_-]{3,32}$/.test(username)) return "Use um usuário de 3 a 32 letras, números, _ ou -.";
  if(byteLength(displayName)<1||byteLength(displayName)>80||/[\u0000-\u001f\u007f]/.test(displayName)) return "Informe um nome de exibição de 1 a 80 bytes, sem caracteres de controle.";
  if(byteLength(password)<8||byteLength(password)>128||/[\u0000-\u001f\u007f]/.test(password)) return "Use uma senha de 8 a 128 bytes, sem caracteres de controle.";
  if(password!==confirmation) return "As senhas não coincidem. Digite novamente.";
  return "";
}

function renderUsers(users) {
  if(!Array.isArray(users)||users.length>64) throw new Error("invalid_users");
  const fragment=document.createDocumentFragment();
  for(const user of users) {
    if(!user||typeof user.username!=="string"||typeof user.display_name!=="string"||!Object.hasOwn(roleNames,user.role)||typeof user.enabled!=="boolean") throw new Error("invalid_user");
    const row=document.createElement("tr"),person=document.createElement("td"),name=document.createElement("strong"),username=document.createElement("small");
    name.textContent=user.display_name;username.textContent=user.username+(user.username===sessionInfo?.username?" (você)":"");person.append(name,username);
    const role=document.createElement("td");role.textContent=roleNames[user.role];
    const state=document.createElement("td");const badge=document.createElement("span");badge.className="account-state"+(user.enabled?" active":"");badge.textContent=user.enabled?"Ativa":"Desativada";state.append(badge);
    const action=document.createElement("td"),button=document.createElement("button");button.type="button";button.textContent=user.enabled?"Desativar":"Ativar";button.setAttribute("aria-label",button.textContent+" usuário "+user.username);button.disabled=usersMutationPending||leaving;
    button.addEventListener("click",()=>void changeUserState(user.username,!user.enabled));action.append(button);row.append(person,role,state,action);fragment.append(row);
  }
  byId("users-list").replaceChildren(fragment);
}

async function loadUsers() {
  if(!isAdmin()||usersPending||usersMutationPending||leaving) return;
  const epoch=generation;usersPending=true;setUsersBusy(usersMutationPending);
  try {const users=await api("/api/auth/users");if(epoch!==generation||!isAdmin()||leaving) return;renderUsers(users);setFeedback("users-message",users.length+" conta(s) cadastrada(s). Somente administradores gerenciam o acesso.");}
  catch(_) {if(epoch===generation&&isAdmin()) setFeedback("users-message","Não foi possível consultar os usuários. Atualize a lista para tentar novamente.","error");}
  finally {usersPending=false;setUsersBusy(usersMutationPending); }
}

function setUsersBusy(busy) {
  usersMutationPending=busy;byId("create-user").disabled=busy||usersPending||leaving;
  byId("users-refresh").disabled=busy||usersPending||leaving;
  for(const button of byId("users-list").querySelectorAll("button")) button.disabled=busy||usersPending||leaving;
}

async function changeUserState(username,enabled) {
  if(!isAdmin()||usersMutationPending||usersPending||leaving) return;
  const epoch=generation;setUsersBusy(true);
  try {
    const users=await api("/api/auth/users/state",{body:{username,enabled}});
    if(epoch!==generation||!isAdmin()||leaving) return;
    if(!enabled&&username===sessionInfo.username) {lock();message("Sua conta foi desativada e a sessão foi encerrada.",true);return;}
    renderUsers(users);setFeedback("users-message",enabled?"Conta ativada. O usuário já pode entrar no painel.":"Conta desativada. As sessões desse usuário foram revogadas.","success");
  } catch(error) {if(epoch===generation&&isAdmin()) setFeedback("users-message",error.status===409?"Não é possível desativar o último administrador ativo.":"Não foi possível alterar o acesso. Atualize a lista antes de tentar novamente.","error");}
  finally {setUsersBusy(false);}
}

function validTopic(topic,filter) {
  if(!topic||new TextEncoder().encode(topic).length>512||/[\u0000-\u001f\u007f]/.test(topic)) return false;
  if(!filter) return !/[+#]/.test(topic);
  const levels=topic.split("/");
  return levels.every((level,index)=>(!level.includes("+")||level==="+")&&(!level.includes("#")||(level==="#"&&index===levels.length-1)));
}

byId("login-form").addEventListener("submit",async event=>{
  event.preventDefault();if(authMode!=="token"||pending||leaving) return;
  const value=byId("token").value.trim();
  if(!/^[a-fA-F0-9]{64}$/.test(value)) {message("Informe os 64 caracteres do token gerado em arquivo privado.",true);return;}
  generation++;accessToken=value;byId("token").value="";updatePermissions();await refresh();
});
byId("user-login-form").addEventListener("submit",async event=>{
  event.preventDefault();if(authMode!=="users"||setupRequired||authPending||leaving) return;
  const username=byId("login-username").value,password=byId("login-password").value;
  const epoch=generation;setAuthBusy(true);byId("login-password").value="";message("Verificando suas credenciais…");
  try {const info=await api("/api/auth/login",{public:true,body:{username,password},timeout:15000});if(epoch!==generation||leaving) return;acceptSession(info);await refresh();}
  catch(error) {if(epoch===generation) message(error.status===401?"Usuário ou senha inválidos.":error.status===429?"Muitas tentativas. Aguarde um pouco antes de entrar novamente.":"Não foi possível entrar. Confira o servidor e tente novamente.",true);}
  finally {setAuthBusy(false);}
});
byId("setup-form").addEventListener("submit",async event=>{
  event.preventDefault();if(authMode!=="users"||!setupRequired||authPending||leaving) return;
  const username=byId("setup-username").value,display_name=byId("setup-display-name").value,password=byId("setup-password").value,bootstrap_token=byId("setup-token").value.trim();
  const invalid=validAccount(username,display_name,password,byId("setup-password-confirm").value);
  if(invalid) {message(invalid,true);return;}
  if(!/^[a-fA-F0-9]{64}$/.test(bootstrap_token)) {message("Informe os 64 caracteres do código local de configuração.",true);return;}
  const epoch=generation;setAuthBusy(true);for(const id of ["setup-token","setup-password","setup-password-confirm"]) byId(id).value="";message("Criando o administrador…");
  try {const info=await api("/api/auth/setup",{public:true,body:{bootstrap_token,username,display_name,password},timeout:15000});if(epoch!==generation||leaving) return;acceptSession(info);await refresh();}
  catch(error) {if(epoch===generation) {if(error.status===409) {setupRequired=false;authForms();message("A configuração inicial já foi concluída. Entre com sua conta.",true);}else message(error.status===401||error.status===403?"Código de configuração inválido ou acesso recusado.":error.status===429?"Muitas tentativas. Aguarde antes de tentar novamente.":"Não foi possível concluir a configuração. Confira o servidor e tente novamente.",true);}}
  finally {setAuthBusy(false);}
});
byId("retry-auth").addEventListener("click",initializeAuth);
byId("users-refresh").addEventListener("click",loadUsers);
byId("create-user-form").addEventListener("submit",async event=>{
  event.preventDefault();if(!isAdmin()||usersMutationPending||usersPending||leaving) return;
  const username=byId("new-username").value,display_name=byId("new-display-name").value,password=byId("new-password").value,role=byId("new-role").value;
  const invalid=validAccount(username,display_name,password,byId("new-password-confirm").value);
  if(invalid) {setFeedback("create-user-message",invalid,"error");return;}
  const epoch=generation;setUsersBusy(true);byId("new-password").value="";byId("new-password-confirm").value="";setFeedback("create-user-message","Criando o usuário…");
  try {const users=await api("/api/auth/users",{body:{username,display_name,password,role},timeout:15000});if(epoch!==generation||!isAdmin()||leaving) return;renderUsers(users);byId("create-user-form").reset();setFeedback("create-user-message","Usuário criado. A conta já pode entrar com o papel selecionado.","success");}
  catch(error) {if(epoch===generation&&isAdmin()) setFeedback("create-user-message",error.status===409?"Esse usuário já existe ou o limite de contas foi atingido.":"Não foi possível criar o usuário. Confira os campos e atualize a lista antes de tentar novamente.","error");}
  finally {setUsersBusy(false);}
});
byId("refresh").addEventListener("click",refresh);
byId("connect-form").addEventListener("submit",async event=>{
  event.preventDefault();if(connectionPending||!canMqtt()||!mqttState.enabled||["connected","connecting"].includes(mqttState.status)) return;
  const topic=byId("subscribe-topic").value;
  if(!validTopic(topic,true)) {setFeedback("console-message","Informe um filtro válido, com + isolado e # apenas no último nível (até 512 bytes).","error");return;}
  const epoch=generation;connectionPending=true;updateControls();setFeedback("console-message","Solicitando conexão e assinatura…");
  try {await api("/api/mqtt/connect",{body:{topic,qos:Number(byId("subscribe-qos").value)},timeout:15000});if(epoch===generation&&hasSession()) {setFeedback("console-message","Pedido enviado. Aguardando a confirmação da assinatura pelo broker.");await refresh();}}
  catch(error) {if(epoch===generation&&hasSession()) setFeedback("console-message",error.status===409?"Já existe uma conexão ativa. Atualize a leitura para consultá-la.":"Não foi possível confirmar a conexão. Confira o perfil, o broker e as permissões; atualize a leitura antes de tentar novamente.","error");}
  finally {connectionPending=false;updateControls();}
});
byId("disconnect").addEventListener("click",async()=>{
  if(connectionPending||!canMqtt()) return;
  const epoch=generation;connectionPending=true;updateControls();
  try {await api("/api/mqtt/disconnect",{body:{}});if(epoch===generation&&hasSession()) {mqttState.status="disconnected";setFeedback("console-message","Assinatura desconectada.");await refresh();}}
  catch(_) {if(epoch===generation&&hasSession()) setFeedback("console-message","Não foi possível confirmar o encerramento. Atualize a leitura para conferir a conexão.","error");}
  finally {connectionPending=false;updateControls();}
});
byId("publish-form").addEventListener("submit",async event=>{
  event.preventDefault();if(publishPending||!canMqtt()||!mqttState.enabled) return;
  const topic=byId("publish-topic").value,payload=byId("publish-payload").value,qos=Number(byId("publish-qos").value),retain=byId("publish-retain").checked;
  if(!validTopic(topic,false)) {setFeedback("publish-message","Informe um tópico de até 512 bytes, sem + ou #.","error");return;}
  if(new TextEncoder().encode(payload).length>4096) {setFeedback("publish-message","O conteúdo ultrapassa 4.096 bytes. Caracteres acentuados podem ocupar mais de um byte.","error");return;}
  const epoch=generation;publishPending=true;updateControls();setFeedback("publish-message","Enviando publicação…");
  try {await api("/api/mqtt/publish",{body:{topic,payload,qos,retain},timeout:20000});if(epoch===generation&&hasSession()) {setFeedback("publish-message",qos===0?"Mensagem enviada com QoS 0. Não há confirmação MQTT de recebimento.":"Publicação confirmada pelo broker com QoS "+qos+". O processamento pelo assinante não é confirmado.","success");await refresh();}}
  catch(_) {if(epoch===generation&&hasSession()) setFeedback("publish-message","A publicação não foi confirmada. Ela não será repetida automaticamente; confira o assinante antes de reenviar.","error");}
  finally {publishPending=false;updateControls();}
});
byId("pause").addEventListener("click",()=>{paused=!paused;byId("pause").textContent=paused?"Retomar exibição":"Pausar exibição";byId("pause").setAttribute("aria-pressed",String(paused));byId("pause-note").hidden=!paused;if(!paused) renderMessages();});
byId("clear").addEventListener("click",()=>{localMessages=[];const wasPaused=paused;paused=false;renderMessages();paused=wasPaused;});
byId("message-search").addEventListener("input",renderMessages);
byId("logout").addEventListener("click",async()=>{
  if(leaving) return;leaving=true;clearTimeout(pollTimer);updateControls();byId("logout").disabled=true;
  let confirmed=authMode!=="users";
  try {
    if(canMqtt()) {try {await api("/api/mqtt/disconnect",{body:{},allowLeaving:true,timeout:8000});}catch(_) { /* A lease limita a assinatura se o encerramento não chegar. */ }}
    if(authMode==="users"&&hasSession()) {try {await api("/api/auth/logout",{body:{},allowLeaving:true});confirmed=true;}catch(error) {confirmed=error.status===401;}}
  } finally {
    lock();leaving=false;byId("logout").disabled=false;
    message(confirmed?"Acesso encerrado nesta página.":"A página foi bloqueada, mas a saída no servidor não foi confirmada. Uma sessão ainda válida pode ser restaurada ao reabrir.",!confirmed);
  }
});
window.addEventListener("pagehide",()=>{
  const headers={"Content-Type":"application/json"};const disconnect=canMqtt();
  if(authMode==="token") headers.Authorization="Bearer "+accessToken;else if(sessionInfo) headers["X-XMQR-CSRF"]=sessionInfo.csrf_token;
  leaving=true;clearTimeout(pollTimer);
  if(disconnect) fetch("/api/mqtt/disconnect",{method:"POST",headers,body:"{}",credentials:"same-origin",cache:"no-store",keepalive:true}).catch(()=>{});
  generation++;accessToken="";sessionInfo=null;
  for(const id of ["token","login-password","setup-token","setup-password","setup-password-confirm","new-password","new-password-confirm"]) byId(id).value="";
});
window.addEventListener("pageshow",event=>{if(event.persisted){leaving=false;lock();void initializeAuth();}});
for(const button of document.querySelectorAll("[data-view]")) button.addEventListener("click",()=>showView(button.dataset.view));
byId("panel-address").textContent=window.location.origin;
authForms();updatePermissions();void initializeAuth();
