# A24 — autenticação por usuários e autorização web

2026-10-19. Entrada5f98f4a, promptd0c3e27 antes do código. Agentes paralelos
implementaram backend, interface e fixture/guia; integrador controloumiddleware,
revisão, persistência, testes reais de navegador e ativação do preview.

PASS local Ubuntu26.04/WSL:44testesRust (incluindo8novos deAuth), fmt,
Clippyall-targets-Dwarnings, buildsdebug/release e inventário. Tempfile passou
de dependência de teste a runtime, com a mesma versão/lock/inventário.
Não houve mudança do core pinado, usuáriosMQTT, ACL ou firmware.

[FixtureHTTPdebug](../../docs/evidence/a24/integration.json):18/18PASS,
ELFSHA2565e112841c76642a9e21576716c8829d2334f64d481839a17a1ff1f181ec90bd0.
[FixtureELFrelease](../../docs/evidence/a24/release-elf.json):18/18PASS.
Inclui bootstrapúnico, Bearerantigosembypass, cookie/CSRF, cada papel, recusa
de dados inválidos, Argon2id/salts, últimaadmin, revogação/logout/restart,
locks/corrupção/symlink/modos/64contas, tentativasconcorrenteslimitadas.

[Browser](../../docs/evidence/a24/browser.json):7cenáriosPASS Chromium1248,
com servidor e contas reais descartáveis, desktop/mobile, setup/login,
gestão de contas, nomes semXSS, observador semconsole, operador semusers,
cookie revogado e session401 apóslogout/reload, semsegredos emstorage/URL.
[Regressão tokenA23](../../docs/evidence/a24/legacy-token-browser.json):9PASS,
incluindo publicação/assinatura com fakeCLIpróprio (MQTTmTLSreal aceito emA23).
Screenshotsinspecionadas emdist/a24-browser-2; nenhum estado do operador usado.

FAILpreservados: primeirocheck tentou extensãoIdentity semClone; removida
extensão inutilizada, autorização mantida. Clippy recusou quatro subtrações
deInstant emtestes, trocadas porchecked_sub. Browser1 teve6passos completos
e timeout de networkidleglobal apóslogout; [relatório](../../docs/evidence/a24/browser-first-fail.json).
Browser2 usa prova específica mais forte: cookie ausente e session401 no
reload; demais asserções preservadas. Mutex de estado não cobre fsync: disco
é serializado emworkersspawn_blocking com fila/concorrência limitada.

Modo opt-inporXMQR_ADMIN_WEB_USERS_FILE. Senha pessoal escolhida pelo usuário,
sem default, apenasArgon2idv19 m19456/t2/p1 salt16B/output32B persistidos.
Sessões256bits emcookieHttpOnly/Strict, sódigest no servidor, CSRF próprio,
64sessões/8h/30mininatividade. Observador/operador/admin. Consolecompartilhado
entreoperadoresautorizados; cada contawebnão cria identidadeMQTT nova.
Nãoháreset/troca de senha/edição de papel nesta fatia; criação e ativação/
desativação estão na interface. Armazenamento e transportedo painel são locais.

PreviewreleaseA24ativadoem127.0.0.1:9080 com storeprivadonovo nohomeLinux.
GETauth/config confirmoumodeusers/setup_requiredtrue; usuário cria sua conta
diretamente no navegador. Modelonão escolheu nem recebeu senha do operador.
Broker8883 e monitor9090 foram preservados; readyHTTP200apósativação.
Pacotesinstalados0.33/0.28/0.10intactos; preview é fonteadmin0.11.0 emvalidação.

Commit/pushautorizados; CIexata será vinculada após conclusão, sem PASS
antecipado. Não criada nova tag/release nesta fatia de fonte. Fase8/C148 do
client permanece em desenvolvimento, sem aceite global ou release antecipada.

Verificação posterior em 2026-10-19: commit52fa7506aecb8c5c1318863692fca9d333d6154f
com [CI39292851127](https://github.com/licodevone/xmqr-admin/actions/runs/39292851127)
concluída com sucesso nos quatro jobs. Fonte A24 aceita; distribuição permanece
no marco anterior, sem nova versão/tag/release.
