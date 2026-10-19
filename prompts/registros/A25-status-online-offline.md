# A25 — indicadores dos três softwares

2026-10-19. Prompt080fe87 antes da funcionalidade, revisõesv1.1–1.3 antes
das respectivas correções/ativação. Base52fa750/A24 com CI39292851127 PASS.
Pedido do usuário: ON-LINE/OFF-LINE no painel web.

Implementado agregado autenticado systems em /api/status, preservando todos
os campos anteriores. Broker confirma reachable&&ready; client significa
somente assinatura compartilhada após SUBACK; admin confirma resposta válida
ao navegador. Viewer recebe apenas booleanos/reasons e agregados numéricos.
Não lê perfis/disco, não copia buffer de mensagens e não renova lease.

Interface inicia aguardando, limpa ao sair/trocar conta e invalida leitura
em falha/timeout/JSON ou métricas inválidos e rede offline. Um pedido MQTT
lento não adia a invalidação de status. Resposta anterior a evento de rede
não restaura estado antigo. Falha isolada do console não sobrescreve status
válido. Dependentes OFF-LINE incluem texto explícito sem confirmação, sem
afirmar que o processo broker morreu. Texto+cor e layout desktop/mobile.

## Validação local

Ubuntu26.04/WSL, ELF final release SHA256
2f9e677a3b20fc61c3c7f16c5c8dd890c3bf3f181d3274f844138b313fb2b404.
46 testes Rust PASS (40unitários,3bundle,3CLI), fmt, Clippyall-targets-Dwarnings,
buildsdebug/release e inventário PASS. Contratos Python33 PASS (11distribuição,
7notices,12canal,3export). Nenhuma dependência ou formato persistido alterado.

- [Integração debug](../../docs/evidence/a25/integration-debug.json):9/9PASS.
- [Integração release](../../docs/evidence/a25/integration-release.json):9/9PASS.
  AdminHTTP real, monitorHTTP próprio, CLI determinístico; autenticação,
  viewer sem payload, monitor pronto/não pronto/inválido/timeout, SUBACK,
  disconnect, saída inesperada e espera real do lease60s sob polling de viewer.
- [Regressão HTTP A24](../../docs/evidence/a25/auth-regression.json):18/18PASS.
- [Regressão HTTP A23](../../docs/evidence/a25/console-regression.json):13/13PASS.
- [Browser A25](../../docs/evidence/a25/browser.json):16/16PASS Chromium1248,
  incluindo métricas como string/readiness2, timeoutAbortController, rede
  offline real, resposta tardia e máximo1consulta de status em paralelo.
- [Browser autenticação](../../docs/evidence/a25/auth-browser.json):7/7PASS.
- [Browser console legado](../../docs/evidence/a25/console-browser.json):9/9PASS.

Capturas desktop/mobile em dist/a25-browser-1 e regressões em dist/a25 foram
inspecionadas. Recursos próprios encerrados, sem senhas/tokens de operador.
As falhas de rede/browser são injetadas e identificadas; o CLI determinístico
não equivale a MQTT externo. A integração mTLS de A23 permanece separada.

Revisões preventivas: v1.1 eliminou atraso do allSettled para falha de status;
v1.2 validou contrato numérico e texto de leitura inválida; v1.3 documentou
o reset feito pelo usuário. Nenhum cenário executado desta fatia falhou.

## Prévia e limites

Após a limpeza solicitada/executada pelo usuário, token/users/perfis estavam
ausentes. Encerrado exclusivamente o antigo PID17460, validando argv/bind;
prévia atualizada com ELF final na9080, sem iniciar broker ou reinstalar pacotes.
Novo token privado gerado pelo CLI existente para bootstrap; não escolhido
usuário/senha web, nem restaurados certificados/senhas/perfis de backup.
HTTP200 e HTML com os três indicadores confirmados no Windows. Auth/config:
modeusers/setup_requiredtrue. Console desabilitado até preparar novos perfis.
Processo supervisionado pela sessão17569; nenhum segredo impresso/publicado.

Esta fatia não conclui fase8/bridgeC148 nem distribui versão nova do admin.
Commit/push autorizados; CI do commit exato pendente de conclusão.
