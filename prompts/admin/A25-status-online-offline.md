# A25 v1.0 — disponibilidade dos três softwares no painel

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

Checklist inicialmente pendente:
- [x] Prompt preparado antes do código.
- [ ] Agregado de disponibilidade sem efeitos.
- [ ] Indicadores e invalidação no navegador.
- [ ] Testes, documentação e evidências.
- [ ] Commit/push e CI do commit exato.
