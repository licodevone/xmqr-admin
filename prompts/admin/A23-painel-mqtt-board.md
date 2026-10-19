# A23 v1.2 — painel local XMQR e console MQTT

2026-10-19. Entrada admin0.11.0/main0796d3b, Git limpo. Usuário solicita
interface semelhante a HiveMQ Web Client/MQTT Board e autoriza agentes paralelos.
Fase8/clientC148 permanece em desenvolvimento separado; não declarada concluída.

Criar interface própria em português no admin existente, servida em9080, com
visão real do monitor9090 e console MQTT para conectar, assinar um filtro,
receber mensagens continuamente, publicar tópico/payload/QoS/retained e desconectar.
`/ready`9090 permanece endpoint de saúde. Não copiar identidade visual/código
dos produtos de referência, nem inventar métricas/mensagens. Sem CDN/telemetria.

Arquitetura: browser fala apenas com API local autenticada do admin; servidor
usa o executável mqtt-client instalado como processo filho com perfis privados
preparados pelo operador. Isso permite TLS8883 existente sem alterar listener,
firewall ou firmware e sem implementar outro codec MQTT. Execução direta sem
shell, sem senha/token em argv, sem endpoints/arquivos arbitrários no HTTP.
Novos campos de configuração via ambiente: XMQR_ADMIN_MQTT_CLIENT_BIN (absoluto,
default/usr/bin/mqtt-client), XMQR_ADMIN_MQTT_SUB_PROFILE e opcional
XMQR_ADMIN_MQTT_PUB_PROFILE (default igual ao sub), XMQR_ADMIN_MQTT_LABEL.
Sem perfis, console desabilitado com instrução clara. Perfis0600 no Linux,
arquivos regulares não symlink. Configuração e senhas nunca retornam ao browser.

Contrato HTTP acordado antes da execução dos agentes:
- GET /api/mqtt: snapshot {enabled,label,status,error,topic,qos,messages,
  received_total,published_total,dropped_total}. Status disabled/disconnected/
  connecting/connected/error. error é código genérico, sem stderr remoto.
  messages até200: {id,received_at,topic,payload,qos,retain,direction};
  received_at Unix milissegundos, direction received/published. Payload textual
  até4096 bytes; binário deve ser indicado como tal, sem HTML executável.
- POST /api/mqtt/connect {topic,qos}: uma assinatura contínua por painel local,
  sem --count; ID exclusivo e estável no processo. Recusar segunda conexão
  enquanto ativa, filtro inválido ou qos fora0..2. Confirmação de conexão só
  após SUBACK real. Sem reconexão infinita.
- POST /api/mqtt/disconnect {}: encerra/recolhe filho, idempotente.
- POST /api/mqtt/publish {topic,payload,qos,retain}: publicação independente
  com prazo limitado, sucesso somente se CLI terminar0; QoS0 indica envio,
  QoS1/2 confirmação MQTT sem alegar processamento do assinante.

Preservar autenticação Bearer, Host/Origin exatos, loopback, CSP self/no-store,
sem CORS. GET continua sem body/query. POST apenas nas três rotas novas, JSON
limitado8192 bytes, Origin local obrigatório e sem transferência chunked;
demais métodos/rotas recusados. Limitar processos/concorrência, buffers/linhas,
tempo de publicação e lease da assinatura (expira sem consultas do browser).
Fechar/recolher filhos ao desconectar, sair ou encerrar servidor; não afetar
processos/serviços do operador. Credenciais permanecem em perfis/arquivos
privados, e payload remoto usa textContent, nunca innerHTML.

UI: sidebar com os mesmos títulos das seções, visão geral, console MQTT e
operação/configuração. Estado vazio honesto, conexão/falha visíveis, filtro,
lista de mensagens com busca/pausa visual/limpeza local e editor de publicação.
Responsividade, teclado, contraste e dados verdadeiros. Não exibir features
administrativas ainda exclusivas do CLI como botões funcionais.

Execução paralela por arquivos: agente interface apenas assets; agente backend
apenas src/mqtt_console.rs (e novos testes internos), propõe mudanças de Cargo
ao integrador; agente validação apenas scripts/verify_mqtt_board.py e
docs/mqtt-board.md. Integrador controla web.rs/main.rs/Cargo/README/CI/prompt/
registro e revisa segurança, contratos e gates. Não permitir commits dos agentes.

Aceite: testes backend de limites/negação/limpeza, fakeCLI determinístico e
integração com broker/client reais isolados TLS; browser real desktop/mobile,
login, recebimento/publicação e XSS, falha/indisponibilidade e logout. Gates
fmt/test/Clippy/buildrelease/inventário, CIexata e evidências. Commit/push após
aceite; versão/tag/release em marco próprio. Não substituir pacote instalado
ou iniciar instância real sem autorização; usuário autoriza levantar serviço
web de teste local para visualizar o resultado, preservando broker ativo.

Primeira validação local:35testesRustPASS. Clippy recusou snapshot async sem
await, dois let _ em retorno unit e assert de vetor vazio. Preservar logs
/var/tmp/a23-tests-1.log e /var/tmp/a23-clippy-1.log; corrigir sem enfraquecer
o gate. snapshot pode retornar std::future::Ready para manter contrato await.
Revisão também exigiu fechar stdin antes de aguardar EOF do publisher e
revalidar estado closed após lock, evitando filhos criados durante shutdown.

Primeira fixture HTTP preservada em dist/a23-integration-1.json:3PASS, depois
FAIL no código de schema desconhecido (Axum422, fixture400; sem processo
iniciado). Padronizar rejeições JSON como400/invalid_request para não expor
detalhes do parser; não aceitar campo extra nem flexibilizar teste. ClientIDs
com sufixos devem ter até23bytes para compatibilidade com MQTT3.1.1.

Browser1 foi impedido antes de abrir página: Playwright pede Chromium1234,
mas o ambiente tem1243/1248. RelatórioFAIL preservado. Usar caminho explícito
do Chromium já instalado, sem instalar ou declarar QA visual antecipadamente.

Fixture2 preservada:8PASS, depois FAIL porque o listener tem lifetime10s e
o publisher tinha prazo12s. Manter a proteção do listener; reduzir prazo do
publisher para6s (até3s adicionais de recolhimento), para retornar erroHTTP
antes do fechamento. Não flexibilizar teste nem ampliar conexões do servidor.
Browser2:9cenáriosPASS emChromium1248, desktop/mobile e conteúdo semXSS.
