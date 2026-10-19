"use strict";
let accessToken = "";
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
const titles = {overview:"Visão geral",console:"Console MQTT",operations:"Operações administrativas",settings:"Configuração e acesso"};
const stateNames = {disabled:"Não configurado",disconnected:"Desconectado",connecting:"Conectando",connected:"Assinatura ativa",error:"Falha na conexão"};
const setFeedback = (id,text,tone="") => {const node=byId(id);node.textContent=text;node.className="feedback"+(tone?" "+tone:"");};
const message = (text,error=false) => {byId("login-message").textContent=text;byId("login-message").classList.toggle("error",error);};

function showView(view) {
  if (!Object.hasOwn(titles,view)) return;
  for (const section of document.querySelectorAll(".view")) section.hidden=section.id!==view;
  for (const button of document.querySelectorAll(".nav[data-view]")) {
    const active=button.dataset.view===view;
    button.classList.toggle("active",active);
    if(active) button.setAttribute("aria-current","page");else button.removeAttribute("aria-current");
  }
  byId("page-title").textContent=titles[view];
}

function updateControls() {
  const enabled=!!accessToken && mqttState.enabled===true && !leaving;
  const active=["connecting","connected"].includes(mqttState.status);
  byId("connect").disabled=!enabled||active||connectionPending;
  byId("disconnect").disabled=!enabled||(!active&&mqttState.status!=="error")||connectionPending;
  byId("subscribe-topic").disabled=!enabled||active||connectionPending;
  byId("subscribe-qos").disabled=!enabled||active||connectionPending;
  byId("publish").disabled=!enabled||publishPending;
  byId("publish").textContent=publishPending?"Publicando…":"Publicar mensagem";
}

function lock() {
  generation++;
  accessToken="";
  clearTimeout(pollTimer);
  byId("token").value="";
  byId("workspace").hidden=true;
  byId("login-section").hidden=false;
  byId("publish-payload").value="";
  byId("publish-topic").value="";
  byId("subscribe-topic").value="";
  byId("message-search").value="";
  for(const item of document.querySelectorAll("[data-metric]")) item.textContent="—";
  byId("commit-total").textContent="Commits duráveis: —";
  byId("sample-time").textContent="Nenhuma leitura realizada";
  for(const id of ["received-total","published-total","dropped-total"]) byId(id).textContent=({"received-total":"Recebidas: —","published-total":"Publicadas: —","dropped-total":"Descartadas do buffer: —"})[id];
  localMessages=[];
  seenMessages.clear();
  paused=false;
  byId("pause").textContent="Pausar exibição";
  byId("pause").setAttribute("aria-pressed","false");
  byId("pause-note").hidden=true;
  mqttState={enabled:false,status:"disabled",label:""};
  renderMessages();
  updateControls();
  showView("overview");
  byId("token").focus();
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
  const token=accessToken;
  if(!token||leaving&&!options.allowLeaving) throw new Error("no_session");
  const controller=new AbortController();
  const timer=setTimeout(()=>controller.abort(),options.timeout||8000);
  try {
    const response=await fetch(path,{method:options.body?"POST":"GET",headers:{Authorization:"Bearer "+token,...(options.body?{"Content-Type":"application/json"}:{})},body:options.body?JSON.stringify(options.body):undefined,cache:"no-store",signal:controller.signal});
    if(accessToken!==token) throw new Error("stale_session");
    if(response.status===401||response.status===403) {lock();message("Token recusado ou origem não autorizada. Confira o arquivo privado.",true);throw new Error("unauthorized");}
    if(!response.ok) {const error=new Error("request_failed");error.status=response.status;throw error;}
    if(response.status===204) return null;
    const text=await response.text();
    if(accessToken!==token) throw new Error("stale_session");
    return text?JSON.parse(text):null;
  } finally {clearTimeout(timer);}
}

function schedulePoll() {
  clearTimeout(pollTimer);
  if(accessToken&&!leaving) pollTimer=setTimeout(refresh,3000);
}

async function refresh() {
  if(pending||!accessToken||leaving) return false;
  pending=true;byId("refresh").disabled=true;
  const epoch=generation;
  try {
    const results=await Promise.allSettled([api("/api/status",{timeout:5000}),api("/api/mqtt",{timeout:5000})]);
    if(epoch!==generation||!accessToken||leaving) return false;
    const status=results[0],mqtt=results[1];
    if(status.status==="fulfilled"&&status.value) {
      renderStatus(status.value);byId("login-section").hidden=true;byId("workspace").hidden=false;message("O token fica apenas na memória desta página.");
    } else if(byId("workspace").hidden) {message("Não foi possível consultar o painel local. Confira se ele está em execução.",true);return false;}
    else renderStatus({reachable:false,ready:false,metrics:{}});
    if(mqtt.status==="fulfilled"&&mqtt.value) renderMqtt(mqtt.value);
    else {
      byId("connection-state").textContent="Consulta indisponível";byId("connection-state").dataset.state="error";
      setFeedback("console-message","Não foi possível consultar o console. Verifique o servidor; a interface não repete publicações.","error");
    }
    return true;
  } finally {pending=false;byId("refresh").disabled=false;schedulePoll();}
}

function validTopic(topic,filter) {
  if(!topic||new TextEncoder().encode(topic).length>512||/[\u0000-\u001f\u007f]/.test(topic)) return false;
  if(!filter) return !/[+#]/.test(topic);
  const levels=topic.split("/");
  return levels.every((level,index)=>(!level.includes("+")||level==="+")&&(!level.includes("#")||(level==="#"&&index===levels.length-1)));
}

byId("login-form").addEventListener("submit",async event=>{
  event.preventDefault();if(pending||leaving) return;
  const value=byId("token").value.trim();
  if(!/^[a-fA-F0-9]{64}$/.test(value)) {message("Informe os 64 caracteres do token gerado em arquivo privado.",true);return;}
  generation++;accessToken=value;byId("token").value="";await refresh();
});
byId("refresh").addEventListener("click",refresh);
byId("connect-form").addEventListener("submit",async event=>{
  event.preventDefault();if(connectionPending||!accessToken||!mqttState.enabled||["connected","connecting"].includes(mqttState.status)) return;
  const topic=byId("subscribe-topic").value;
  if(!validTopic(topic,true)) {setFeedback("console-message","Informe um filtro válido, com + isolado e # apenas no último nível (até 512 bytes).","error");return;}
  const epoch=generation;connectionPending=true;updateControls();setFeedback("console-message","Solicitando conexão e assinatura…");
  try {await api("/api/mqtt/connect",{body:{topic,qos:Number(byId("subscribe-qos").value)},timeout:15000});if(epoch===generation&&accessToken) {setFeedback("console-message","Pedido enviado. Aguardando a confirmação da assinatura pelo broker.");await refresh();}}
  catch(error) {if(epoch===generation&&accessToken) setFeedback("console-message",error.status===409?"Já existe uma conexão ativa. Atualize a leitura para consultá-la.":"Não foi possível confirmar a conexão. Confira o perfil, o broker e as permissões; atualize a leitura antes de tentar novamente.","error");}
  finally {connectionPending=false;updateControls();}
});
byId("disconnect").addEventListener("click",async()=>{
  if(connectionPending||!accessToken) return;
  const epoch=generation;connectionPending=true;updateControls();
  try {await api("/api/mqtt/disconnect",{body:{}});if(epoch===generation&&accessToken) {mqttState.status="disconnected";setFeedback("console-message","Assinatura desconectada.");await refresh();}}
  catch(_) {if(epoch===generation&&accessToken) setFeedback("console-message","Não foi possível confirmar o encerramento. Atualize a leitura para conferir a conexão.","error");}
  finally {connectionPending=false;updateControls();}
});
byId("publish-form").addEventListener("submit",async event=>{
  event.preventDefault();if(publishPending||!accessToken||!mqttState.enabled) return;
  const topic=byId("publish-topic").value,payload=byId("publish-payload").value,qos=Number(byId("publish-qos").value),retain=byId("publish-retain").checked;
  if(!validTopic(topic,false)) {setFeedback("publish-message","Informe um tópico de até 512 bytes, sem + ou #.","error");return;}
  if(new TextEncoder().encode(payload).length>4096) {setFeedback("publish-message","O conteúdo ultrapassa 4.096 bytes. Caracteres acentuados podem ocupar mais de um byte.","error");return;}
  const epoch=generation;publishPending=true;updateControls();setFeedback("publish-message","Enviando publicação…");
  try {await api("/api/mqtt/publish",{body:{topic,payload,qos,retain},timeout:20000});if(epoch===generation&&accessToken) {setFeedback("publish-message",qos===0?"Mensagem enviada com QoS 0. Não há confirmação MQTT de recebimento.":"Publicação confirmada pelo broker com QoS "+qos+". O processamento pelo assinante não é confirmado.","success");await refresh();}}
  catch(_) {if(epoch===generation&&accessToken) setFeedback("publish-message","A publicação não foi confirmada. Ela não será repetida automaticamente; confira o assinante antes de reenviar.","error");}
  finally {publishPending=false;updateControls();}
});
byId("pause").addEventListener("click",()=>{paused=!paused;byId("pause").textContent=paused?"Retomar exibição":"Pausar exibição";byId("pause").setAttribute("aria-pressed",String(paused));byId("pause-note").hidden=!paused;if(!paused) renderMessages();});
byId("clear").addEventListener("click",()=>{localMessages=[];const wasPaused=paused;paused=false;renderMessages();paused=wasPaused;});
byId("message-search").addEventListener("input",renderMessages);
byId("logout").addEventListener("click",async()=>{
  if(leaving) return;leaving=true;clearTimeout(pollTimer);updateControls();byId("logout").disabled=true;
  try {if(accessToken) await api("/api/mqtt/disconnect",{body:{},allowLeaving:true,timeout:8000});}
  catch(_) { /* A lease do servidor limita a assinatura se o encerramento não chegar. */ }
  finally {lock();leaving=false;byId("logout").disabled=false;message("Acesso encerrado nesta página.");}
});
window.addEventListener("pagehide",()=>{
  const token=accessToken;leaving=true;clearTimeout(pollTimer);
  if(token) fetch("/api/mqtt/disconnect",{method:"POST",headers:{Authorization:"Bearer "+token,"Content-Type":"application/json"},body:"{}",cache:"no-store",keepalive:true}).catch(()=>{});
  generation++;accessToken="";byId("token").value="";
});
window.addEventListener("pageshow",event=>{if(event.persisted){leaving=false;lock();message("A página foi reaberta. Informe seu token para continuar.");}});
for(const button of document.querySelectorAll("[data-view]")) button.addEventListener("click",()=>showView(button.dataset.view));
byId("panel-address").textContent=window.location.origin;
updateControls();
