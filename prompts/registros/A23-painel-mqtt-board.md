# A23 — painel e console MQTT local

2026-10-19. Entrada0796d3b/admin0.11.0, prompt553139c antes do código;
revisões1.1/1.2 após falhas preservadas. Três agentes trabalharam em assets,
backend e validação/guia; integrador controlou rotas/segurança/Cargo/CI.

PASS local Ubuntu26.04/WSL:36testesRust, fmt, Clippyall-targets-Dwarnings,
builddebug/release, inventário e33contratosPython(distribuição11/notices7/
lifecycle12/export3). CacheCARGO_TARGET_DIR=/var/tmp/xmqr-admin-a22-native.

Fixture17/17PASS: authHostOriginBearer, limites/métodos/JSON/schema, buffer200,
SUBACK antes deconnected, stdin/argv semshell/ambienteMQTTlimpo, prazo6s,
concorrência/recolhimento/lease60s, broker/client próprios reais mTLS,
QoS0/1/2 eACLnegativa. ELFdebugSHA256
d5e1679fc65d361b7c9ab0d7a85ebb912fe745c03a5126c28f8d354f9feb2f42.
[Evidência](../../docs/evidence/a23/integration.json).

Browser9cenáriosPASS emChromium1248/PlaywrightWindows:login válido/inválido,
assinatura/publicação, conteúdo semXSS, busca/pausa/limpeza, mobile semoverflow,
logout limpando dados/assinatura e ausência deexceptions. Servidor e fakeCLI
próprios; MQTTreal é gate separado acima. Screenshots inspecionadas em
dist/a23-browser-2/{desktop,mobile}.png.
[Relatório](../../docs/evidence/a23/browser.json).

FAILs preservados:fixture1 recebeu422 para unknownfield; handlers passaram a
400/invalid_request sem detalhes do parser. Fixture2 detectou prazo12s maior
que lifetimeHTTP10s; publisher agora6s+reap<=3s, mantendo proteções do listener.
Browser1 não abriu por Chromium1234 ausente; usado1248 já instalado.
Clippy inicial corrigiu async/unit/assert, sem enfraquecer gates.

Preview próprio iniciado em127.0.0.1:9080, com perfis privados e token no home
Linux, executáveldebug sem substituir pacotes instalados. Broker original
mantido; seu monitor9090 continuaHTTP200. Preflight da assinatura real DHT22
confirmouSUBACK e descartou stdout sem publicar conteúdo nem alterar ACL.
Nenhuma credencial entrou no repositório ou nos relatórios.

Limites:token único local nesta fatia, uma assinatura compartilhada por painel,
texto4096bytes/200recentes; operações administrativas continuam noCLI. Perfis
não são editáveis peloHTTP; semlistenerWSSnovo nem acesso remoto ao painel.
Autenticação por usuários é pedido posterior A24, não implementada porA23.
Client/broker e fase8 permanecem separados, sem aceite global antecipado.

Commit/push autorizado; CI da fonte exata será vinculada após execução.
Sem bump/tag/release nesta fatia de fonte: pacote publicado continua0.11.0.

CI posteriormente confirmada: [39291303909](https://github.com/licodevone/xmqr-admin/actions/runs/39291303909),
fonte5f98f4a7a3b2a9e12b4cc1ccdb07920edead5098, quatro jobsPASS:validate,
core-v16,ubuntu-package,mqtt-board. Inclui novo gate real MQTTmTLS e pacotes
descartáveis. Nenhuma tag/release nova foi criada.
