# A02 — execução e aceite do painel local de leitura

Revisão pré-tagA02v1.4: negativos token fortalecidos com pai700 e positivo
prévio; antes, duas fixtures eram recusadas no pai755 de tempfile. Runtime
intocado, mas novo commit/artefatos/gates substituirão candidato440e0d1 para
primeira tag0.1.0. Os resultados abaixo são checkpoints anteriores; o novo
aceite deve ser registrado após a repetição, sem alegar negativos específicos
de FIFO/symlink por recusa apenas do pai. Nenhuma tag já publicada foi movida.

Aceitev1.4 realmente reexecutado no rootWSL antes do commit: fmt21RustClippy
releaseinventárioPASS. Positivo privado prévio passou e negativos específicos
de FIFO/symlink/permissões/bytes passaram com pai700. BinárioreleaseSHA256
b6edea14c7214ce4574f08d954a8a2e4833995f0f5b6bd054d133f412d0f473e,
idêntico ao testado no browser/API e reproduçãoTCP independente; somente
testes mudaram, não o runtime. Clone/pacotes/imagem do novo commit continuam
obrigatóriosA03 antes da primeira tag.

Data: 2026-10-13. Prompt de entrada:
[A02 v1.3](../admin/A02-painel-local.md): ensaios iniciais sobre v1.1,
correção de admissão TCP/token v1.2 e ajuste de fixture v1.3, todos com prompt
antes das respectivas alterações. Esta é a repetição final dos gates.
Versão candidata: xmqr-admin 0.1.0; broker independente de teste: 0.11.5.

## Recorte entregue

Servidor Axum e assets próprios locais, sem CDN, painel PT-BR e sidebar
Visão geral / Operações administrativas / Sobre. Os agregados vêm do monitor
loopback; o backend não abre WAL ou executa CLI por shell para atender HTTP.
Operações administrativas no painel são orientações para comandos, não
botões de alteração ao vivo. Token aleatório privado distinto de senha MQTT.

## Gates e negativos reais

Suite Rust recompilada após A02 v1.3: 21 PASS (18 unitários + 3 integração),
inclusive configuração restrita a IP loopback/porta não zero, Host/Origin
exatos, token privado/symlinks,
token inválido, duplicatas Host/Auth, body/chunked, concorrência e interpretação
de métricas. Os novos negativos cobriram admissão TCP/saturação/liberação e
deadline absoluto mesmo com bytes de headers lentos, além de pai público/FIFO
do token sem bloqueio. Clippy/fmt/release/inventário também PASS.

Integração por HTTP com processos release próprios: 18 cenários PASS no
conjunto A01/A02. Para o painel, foram recusados bearer ausente/incorreto,
Host diferente, Origin diferente/null, POST, query e corpo GET. Shell
público apresentou headers de segurança. MQTT independente alterou contagens
reais; três consultas não mudaram documentos duráveis. Broker parado resultou
em indisponibilidade, não métricas zero inventadas. Monitores simulados
exclusivamente para negativos cobriram readiness 503, métricas inválidas,
resposta maior que 64 KiB, redirect e timeout.

## Browser e revisão visual

Python Playwright estava ausente nos runtimes conferidos. Conforme A02 v1.1,
usado Playwright JavaScript/Chromium já instalado, sem instalação global,
com o helper `with_server.py` da skill webapp-testing após consultar `--help`.
O helper gerenciou a fixture WSL; browser isolado headless, sem perfil do
usuário. Nenhum gate Python Playwright foi declarado PASS.

O browser final repetido contra release atualizado passou oito verificações:

1. Shell sem dados privados antes de login.
2. Token incorreto recusado, com orientação.
3. Token correto mostra agregados do broker real de teste e limpa input.
4. Sidebar e atualização funcionam.
5. Sem local/session storage, token/query ou overflow horizontal desktop.
6. Mobile 390 px sem overflow horizontal.
7. Sair/recarregar limpa acesso e indicadores.
8. Sem exceções não tratadas ou erros inesperados no console.

Imagens desktop 1440 × 1000 e mobile 390 px foram abertas e inspecionadas:
texto, navegação, indicadores, instruções e rodapé legíveis no recorte,
sem cortes/overlap horizontais. Isso não é certificação de acessibilidade
ou ensaio de todos os browsers e viewports.

Saídas finais ignoradas por Git em `dist/qa-final/`; SHA256 no checkpoint:

| Arquivo | SHA256 |
| --- | --- |
| browser.json | 9623a3d665c8b7056ee1e62a2756b47cff6580ee1d9f02552513bde21d0b752a |
| desktop.png | 6d6cb1804652af128f67427a8c9ea0f5b103a93acbf682c235e97079e73a495c |
| mobile.png | 17756e5e5ca7baf2a7fc7ce79d7a1b638e1d30ac0cf1dfbec25abd736a1e3d98 |
| integration.json | d48dd7dcc1797428119f73d2f0902a1399379df7d4359792f37337e44e4ff41c |

## Encerramento e limites

Portas 19890/19891/19892 estavam livres em Windows e WSL antes do ensaio.
Handoff/STOP próprios em `../.artifacts/admin-qa-final-20261013/`;
`handoff.closed.json` confirmou `cleaned: true`. O browser encerrou e o
helper confirmou parada do servidor. Nenhuma dessas portas permaneceu
escutando em Windows/WSL e o arquivo de token próprio deixou de existir.
Token temporário em memória, nunca impresso ou versionado; fixture removeu
estado/token Linux próprios.

As telas representam uma conexão/sessão/mensagem retida de teste, não dados
do operador ou evidência ESP32. Revisão paralela intermediária reproduziu
64 conexões com headers HTTP incompletos abertas por 35 s, fora da admissão
do middleware.
Antes do aceite, A02 v1.2 acrescentou limite/lifetime TCP e abertura
endurecida do token. O primeiro gate após isso passou os negativos listener
e token, mas quatro fixtures web falharam por pai não privado. A02 v1.3
precedeu o ajuste da fixture para 0700, sem relaxar o runtime. Repetição final
de fmt, 21 testes Rust, Clippy, release, inventário, 18 integração e 8 browser
PASS, sem nova edição de código pelo agente QA.

Reprodução TCP independente posterior pelo agente de segurança, com o mesmo
binário release SHA256
`b6edea14c7214ce4574f08d954a8a2e4833995f0f5b6bd054d133f412d0f473e`:
64 headers incompletos resultaram em 32 conexões admitidas/32 fechadas;
após 11 s não havia conexão aberta; novo HTTP retornou 200. SIGTERM com
saturação encerrou em 5.049 s e fechou todas as conexões. A fixture foi limpa.
O lifetime absoluto 10 s é política do GET local, incluindo keepalive;
não comprova streaming ou desempenho remoto.
Os quatro cenários foram conferidos no relatório sanitizado
[HTTP/admissão](../../docs/evidence/http-admission.json). Ancestrais dos
diretórios precisam ser confiáveis; não é sandbox contra o próprio usuário
ou root.

A02: PASS no escopo local de leitura e limites efetivamente testados.
Sem ACL/grupos/papéis/sessões ao vivo, HTTPS remoto, produção ou homologação
Windows nativa. Fases 3/5 continuam independentes; A03/distribuição exige
aceites próprios. Não houve publicação comprovada por este registro.
