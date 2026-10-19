# A25 v1.3 — disponibilidade dos três softwares no painel

2026-10-19. Entrada: main 52fa750, A24 aceito, CI39292851127 PASS.
Pedido: indicar ON-LINE/OFF-LINE para broker, client e admin no painel web.

## Contrato antes da implementação

- Acrescentar três indicadores na Visão geral, preservando visual, login,
  papéis, segurança HTTP e compatibilidade dos campos atuais de /api/status.
- GET autenticado /api/status acrescenta systems.admin, systems.broker e
  systems.client, cada um com online booleano e reason enum estável.
  admin online=true/available significa resposta válida do servidor local.
  broker online=reachable && ready: ready, not_ready ou monitor_unavailable.
  client significa exclusivamente a assinatura compartilhada do console,
  online apenas após SUBACK, não um inventário de processos/ESP32 externos.
  reason: connected, connecting, disconnected, disabled ou error.
- O agregado client é permitido ao viewer, sem tópico, payload, credenciais,
  perfis ou caminhos. Sua leitura não renova lease nem clona buffer de mensagens.
- Browser atualiza a cada ciclo existente (3s, sem consultas sobrepostas),
  inicia em Aguardando e nunca presume ON-LINE pela instalação/HTML carregado.
  Resposta inválida, timeout, rede offline ou falha de /api/status invalida os
  três estados: admin OFF-LINE e dependentes sem confirmação, texto explícito.
  Não afirmar que o broker morreu apenas porque o admin ficou inacessível.
  Processar a conclusão de /api/status assim que ocorrer: uma consulta MQTT
  lenta em paralelo não pode adiar a invalidação de uma falha de status já
  conhecida. Manter a barreira das consultas para impedir sobreposição.
  Validar também métricas: monitor acessível exige os oito agregados numéricos
  inteiros não negativos, xmqr_ready=0/1 e ready=true exige xmqr_ready=1.
  Monitor inacessível exige métricas vazias. Nunca exibir string arbitrária
  no lugar de contador nem manter ON-LINE com resposta estruturalmente inválida.
  Limpar estados no logout e ao trocar conta. Falha apenas em /api/mqtt não
  deve substituir o agregado autenticado válido de /api/status.
- Explicar nos indicadores os critérios e limites; cor + texto, acessibilidade,
  desktop/mobile sem overflow e sem dependências novas. Texto remoto via textContent.

## Arquivos e migração

src/web.rs e src/mqtt_console.rs (agregado sem efeitos); assets/index.html,
app.css, app.js; scripts de evidência/browser; docs e registro A25.
Nenhuma alteração de formato persistido ou dependência. Não apagar configurações,
não reiniciar serviços do broker nem alterar instalações/perfis reais.
Preview admin só poderá receber atualização preservando seu ambiente e estado.
Exceção documentada para reset explicitamente pedido nesta conversa: após o
usuário remover as configurações, não restaurar senhas/certificados/perfis de
backup. Se token local também foi removido, gerar novo token privado pelo CLI
existente para o bootstrap de contas novas. Retirar variáveis de perfis que
não existem e iniciar console desabilitado. Não reinstalar/subir broker.
Validar PID/argv/bind da prévia exata antes de encerrá-la; nenhum outro processo.

## Aceite e validação

- Unitários proporcionais: agregado sem mensagens/lease, estados e pronta resposta.
- Integração própria: autenticação exigida, viewer sem payload, monitor pronto,
  indisponível/não pronto; client desconectado, conectando, SUBACK e desconexão.
- Browser: estados iniciais, ON-LINE/OFF-LINE, falha/timeout e recuperação,
  rede offline, logout, desktop/mobile e regressões login A24/console A23.
- Gates: fmt, testes Rust, clippy, build release, inventário; testes próprios
  sem dados do operador. Documentar limites/fracassos e revisar prompt antes
  de correções originadas em falha. Commit/push conforme combinado; não criar
  tag/release de distribuição sem pacote/imagem e respectivos gates.

Revisão v1.1 antes da correção: revisão da interface identificou que o
Promise.allSettled herdado poderia aguardar /api/mqtt por até5s mesmo após
/api/status falhar. Antecipar o tratamento de status na própria promessa e
testar falha de status com MQTT lento; sem enfraquecer prazos/autenticação.

Revisão v1.2 antes da correção: revisão independente encontrou validação de
metrics apenas como objeto. Acrescentar ensaio com contador string e valor
de prontidão inválido; validar contrato numérico do monitor no navegador.

Revisão operacional v1.3: após a limpeza executada pelo usuário, leitura de
existência confirmou token/users/perfis ausentes, mas o processo antigo ainda
respondia em memória. A nova prévia deve refletir o reset sem reaproveitar
segredos antigos ou anunciar console/broker ativos sem evidência.

Checklist inicialmente pendente:
- [x] Prompt preparado antes do código.
- [x] Agregado de disponibilidade sem efeitos.
- [x] Indicadores e invalidação no navegador.
- [x] Testes, documentação e evidências.
- [ ] Commit/push e CI do commit exato.
