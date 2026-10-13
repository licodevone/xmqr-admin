"use strict";
let accessToken = "";
let pending = false;
const byId = id => document.getElementById(id);
const message = (text, error=false) => { byId("login-message").textContent=text; byId("login-message").classList.toggle("error", error); };

function lock() {
  accessToken = "";
  byId("token").value = "";
  byId("workspace").hidden = true;
  byId("login-section").hidden = false;
  for (const item of document.querySelectorAll("[data-metric]")) item.textContent = "—";
  byId("commit-total").textContent="Commits duráveis: —";
  byId("sample-time").textContent="Nenhuma leitura realizada";
  byId("token").focus();
}

function render(data) {
  const available = data.reachable === true;
  const ready = available && data.ready === true;
  byId("availability").dataset.state = ready ? "ready" : available ? "unready" : "offline";
  byId("overview-title").textContent = ready ? "Broker pronto para operar" : available ? "Monitor ativo, broker não está pronto" : "Monitor do broker indisponível";
  byId("broker-description").textContent = ready ? "O ator e o armazenamento responderam à verificação." : available ? "Verifique a instância do broker. O painel não reinicia serviços automaticamente." : "Sem leitura válida. Confira o monitor opcional e seu endereço local.";
  byId("monitor-help").hidden = available;
  for (const item of document.querySelectorAll("[data-metric]")) item.textContent = available && Object.hasOwn(data.metrics,item.dataset.metric) ? String(data.metrics[item.dataset.metric]) : "—";
  byId("commit-total").textContent = "Commits duráveis: " + (available ? String(data.metrics.xmqr_persistence_commits_total) : "—");
  byId("sample-time").textContent = "Última consulta: " + new Date().toLocaleTimeString("pt-BR");
  byId("version-sidebar").textContent="Versão " + data.admin_version;
  byId("version-footer").textContent=data.admin_version;
}

async function refresh() {
  if (pending || !accessToken) return false;
  pending = true;
  byId("refresh").disabled = true;
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 5000);
  const requestToken = accessToken;
  try {
    const response = await fetch("/api/status",{headers:{Authorization:"Bearer " + requestToken},cache:"no-store",signal:controller.signal});
    if (accessToken !== requestToken) return false;
    if (response.status === 401 || response.status === 403) { lock(); message("Token recusado ou origem não autorizada. Confira o arquivo privado.",true); return false; }
    if (!response.ok) throw new Error("local_request_failed");
    const data = await response.json();
    if (accessToken !== requestToken) return false;
    render(data);
    byId("login-section").hidden = true;
    byId("workspace").hidden = false;
    message("O token fica apenas na memória desta página.");
    return true;
  } catch (_) {
    if (accessToken === requestToken) {
      if (byId("workspace").hidden) message("Não foi possível consultar o painel local. Confira se ele está em execução.",true);
      else render({reachable:false,ready:false,metrics:{},admin_version:byId("version-footer").textContent});
    }
    return false;
  } finally { clearTimeout(timer); pending=false; byId("refresh").disabled=false; }
}

byId("login-form").addEventListener("submit", async event => {
  event.preventDefault();
  if (pending) return;
  const value=byId("token").value.trim();
  if (!/^[a-fA-F0-9]{64}$/.test(value)) { message("Informe os 64 caracteres do token gerado em arquivo privado.",true); return; }
  accessToken=value;
  byId("token").value="";
  await refresh();
});
byId("refresh").addEventListener("click",refresh);
byId("logout").addEventListener("click",() => { lock(); message("Acesso encerrado nesta página."); });
window.addEventListener("pagehide",() => { accessToken=""; byId("token").value=""; });
for (const button of document.querySelectorAll("[data-view]")) {
  button.addEventListener("click",() => {
    for (const section of document.querySelectorAll(".view")) section.hidden=section.id !== button.dataset.view;
    for (const item of document.querySelectorAll("[data-view]")) { item.classList.toggle("active",item===button); if(item===button) item.setAttribute("aria-current","page"); else item.removeAttribute("aria-current"); }
    byId("page-title").textContent=button.textContent;
  });
}
