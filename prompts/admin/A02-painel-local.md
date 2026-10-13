# A02 v1.3 — painel local autenticado, somente leitura

EntradaA00/A01. Objetivo: interface funcional própria para observar um broker
XMQR e consultar como executar operaçõesCLI; não é API de mutação ao vivo.
Preparado antes de src/web.rs, config/token/monitor e assetsHTML/CSS/JS.
BackendRust/Axum0.8; HTTPclientReqwest0.12 sem TLS (somente monitorloopbackHTTP).
Fontes oficiaisAxum/Reqwest consultadas antes de uso; lock fixa versões reais.

Configuração: XMQR_ADMIN_BIND default127.0.0.1:9080 e
XMQR_ADMIN_MONITOR_BIND default127.0.0.1:9090, ambosSocketAddrloopback/porta1..65535;
XMQR_ADMIN_TOKEN_FILE obrigatório, regular privado0600Unix, leitura até128bytes,
token aleatório32bytes emhex64. token-init cria somente arquivo novo em pai
privado, sem imprimir token. Estado/configuração imutáveis por processo.

GET públicos:HTML e assets locais, sem CDN, scripts inline ou dados privados.
GET/api/status:Bearer constante-tempo, Host/Origin loopback e porta do servidor
validados, CORS ausente, cache no-store, CSP/frame-ancestorsnone/nosniff/referrer.
Não guardar token em URL/storage/logs; frontend mantém somente memória, permite
sair e limpar. Erros fixos sem dados da origem. Só leitura, métodos de escrita
recusados. Não expor em LAN nem usar token como identidade MQTT.

Monitor:URLs construídas só com endereço fixado; sem proxy/redirect; deadlines
de conexão/total, resposta até64KiB, concorrência limitada16 e um refreshUI
em voo; sem poll automático agressivo. Consultar/ready e /metrics do monitor
opcional existente; aceitar apenas agregados numéricos de nomes fixos. Ausência,
503, resposta inválida ou timeout não são zero/sucesso inventado. A UI não
mostra ClientIDs, usernames, tópicos ou payloads. Não abreWAL nem chamaCLI via shell.

Design: azul aço#18364A, branco#F6F8FB, tinta#183047, azul#087CAA,
âmbar#8E5B13 e erro#B2383F. FonteSegoeUI/TrebuchetMS local, Consolas apenas
valores. Sidebar236px, cabeçalho de estação, linha de disponibilidade e lista
de indicadores com separadores; sem cartões decorativos/sombras ou dados fictícios.
TabsVisão/Operações/Sobre funcionais; linguagemPT-BR, teclado/focus/aria,
responsive360px e reducedmotion. Revisão do brief: painel de operação, não landing
page; reduzir seis cards idênticos a uma grade de indicadores de leitura.

Gates:config/token/headers/auth/CSRF/SSRF/bodylimit/timeout/invalidmetrics negativos;
processo real com broker temporário, MQTT cliente independente para alterar
agregados, sem alteração de estado pelo painel; baseline saúde e broker parado.
BrowserPlaywright com servidor gerenciado, screenshot desktop/mobile, login ruim,
login bom, atualização real, sair, tabs, ausência de storage/leaks e console limpo.
Ferramenta indisponível = BLOCKED explícito, não PASS. Fase3/5 não concluídas.

Revisão antes de correções: primeiro compile/test12PASS; Clippy recusou
format_collect apenas no gerador de amostra. Usar writeln sobreString, sem
relaxar lint. PythonPlaywright ausente nos dois runtimes conferidos, mas
PlaywrightJavaScript/browser já existem no bundle: adaptar QA paraNode/MJS,
sem instalação global. Registrar fallback e gatebrowser real, não alegar
PythonPlaywrightPASS. Configuração envnãoUTF8 deve falhar, não usar default.

Revisão de segurança paralela encontrou P2 real antes do primeiro commit:
64 sockets HTTP com headers incompletos ficaram abertos por35s, fora do
limite16 adquirido pelo middleware. Antes da publicação, acrescentar
src/listener.rs: até32 conexões aceitas simultâneas, permit vinculado à vida
do stream e limite absoluto10s por conexão, incluindo headers incompletos,
keepalive e peer não leitor. Saturação fecha o novo socket, sem tarefa extra;
não alterar bind/autorização/limites do monitor. HTTP GET local não é streaming;
navegador reabre conexão nas consultas seguintes. Sem unsafe. Negativos TCP
reais para saturação/liberação, headers lentos e reconexão; repetir gates/API/
browser e reprodução64sockets. Documentar política, não alegar TLS/remoto.
Token: endurecer leitura contra troca do caminho: validar pai privado e abrir
com O_NOFOLLOW/O_NONBLOCK, validar tipo/mode do mesmo descritor; dependência
libc apenas constantesUnix, sem chamadaunsafe. Testes FIFO/symlink/permissões.
Não emitir detalhes de vulnerabilidade em release antes da correção aceita.

Gate pós-correção: quatro testes web recusaram pai de token não privado.
A fixture test_state criava TempDir sem chmod700; ajustar somente a fixture
antes de token::read, preservando exigência de pai privado no runtime.
Repetir suíte completa e negativos, sem relaxar validação para passar testes.
