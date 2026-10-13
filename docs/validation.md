# Validação do primeiro XMQR Admin independente

Checkpoint final A01/A02 em 2026-10-13, candidato `xmqr-admin 0.1.0`, após
correção de admissão TCP/token e ajuste de fixture A02 v1.3. Estes resultados
validam A01/A02 no ambiente descrito; não comprovam uma publicação A03,
homologação Windows nativa ou conformidade MQTT/HTTP completa.

## Ambiente e origem

- Ubuntu-26.04 em WSL, x86_64; Rust/Cargo 1.99.0, Python 3.14.4.
- Core `mqtt-broker 0.11.5` por dependência Git imutável
  `3606ede844210997fa733408855099535145ac48`; extrações em `ORIGIN.json`.
- Broker independente de teste: `../xmqr/target/release/mqtt-broker`, cuja
  saída `--version` confirmou 0.11.5. O admin não iniciou serviços reais.
- Cargo metadata: 181 pacotes resolvidos, incluindo o projeto raiz;
  inventário de 180 dependências declaradas, inclusive desenvolvimento e
  outras plataformas. Isso não é um inventário exato do binário nem uma
  auditoria jurídica das licenças.
- Browser: Chromium headless/Playwright JavaScript já disponíveis no bundle
  Windows, conectado a processos Linux de teste pelo encaminhamento localhost.
  Python Playwright estava indisponível; não foi instalado nem declarado PASS.

## Gates executados

| Gate | Resultado deste checkpoint |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo test --locked` | PASS: 18 unitários + 3 integração Rust = 21 |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| `cargo build --locked --release` | PASS |
| `python3 scripts/license_inventory.py --check` | PASS |
| `scripts/integration.py` com admin/broker release reais | PASS: 18 cenários |
| `scripts/browser_qa.mjs` com servidor gerenciado | PASS: 8 verificações |
| Inspeção visual desktop 1440 × 1000 e mobile 390 px | PASS no recorte testado |
| Encerramento da fixture browser | `handoff.closed.json`: `cleaned: true` |
| Reprodução TCP real após correção | PASS: 64 incompletos, 32 admitidos/32 fechados; após 11 s, zero abertos; reconexão HTTP 200 |
| SIGTERM sob saturação TCP | PASS: 5.049 s, todas as conexões fechadas |

Revisão paralela identificou conexões com headers HTTP incompletos mantidas
além do limite do middleware. Após prompt, o listener ganhou teto de 32
conexões e lifetime absoluto de 10 s, independente de headers, keepalive ou
atividade do peer. Os dois novos testes TCP reais e a reprodução independente
passaram; os gates Rust, API e browser também foram repetidos. O teto de
16 requests no middleware continua complementar, não substitui admissão TCP.

O binário release efetivamente usado na campanha final e na reprodução TCP
teve SHA256 `b6edea14c7214ce4574f08d954a8a2e4833995f0f5b6bd054d133f412d0f473e`.
Este é um checksum de evidência local, não uma confirmação de asset publicado.
Os quatro cenários da revisão TCP independente estão no relatório sanitizado
[HTTP/admissão](evidence/http-admission.json), lido e conferido contra o mesmo
checksum. O modelo pressupõe ancestrais de diretório confiáveis; não protege
contra o próprio usuário ou root.

Os dois registros detalhados são [A01](../prompts/registros/A01-cli.md) e
[A02](../prompts/registros/A02-painel.md). Não contar os 18 cenários Python
como testes Rust, nem somar subprocessos como cenários adicionais.

## Falhas encontradas e retomada

O ensaio inicial de senha via PTY registrou apenas diagnóstico sanitizado
`exit: 0, prompts: 2, echoed: true`. A automação enviava a entrada entre o
flush do prompt e a desativação de ECHO pelo leitor de senha. Conforme A01
v1.3, o harness passou a aguardar o estado `termios.ECHO` desativado, com
deadline. A exigência de senha ausente da saída foi preservada. A integração
ampliada passou; não houve correção do CLI para mascarar a falha.

Depois da mudança de diretório de rapidez para acmebr, a primeira repetição
dos gates passou os 15 unitários mas falhou os três testes CLI com ENOENT.
O teste compilado ainda continha o caminho absoluto antigo em
`CARGO_BIN_EXE_xmqr-admin`. O binário no novo diretório existia. Confirmados
`readlink -f target` e Cargo `target_directory` dentro de `xmqr-admin/target`,
`cargo clean -p xmqr-admin` removeu somente artefatos regeneráveis desse
pacote (1199 arquivos, 481.6 MiB). Nenhuma fonte, fixture privada ou asserção
foi alterada. A reconstrução e todos os gates subsequentes do snapshot A02
v1.1 passaram.

Ao endurecer a leitura do token em A02 v1.2, quatro testes web passaram a
recusar o pai da fixture. Os dois negativos listener e três token passaram,
mas a suite parou (14/18 unitários PASS). Conforme prompt A02 v1.3, a fixture
`test_state` foi ajustada para diretório privado 0700 antes da leitura. A
validação do runtime não foi relaxada. A repetição final passou 21 testes
Rust, fmt, Clippy, build release locked, inventário, 18 cenários de integração
e 8 de browser; não foi reaproveitado o PASS do snapshot anterior.

## Reprodução

Executar no diretório do repositório, em Linux/WSL:

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
python3 scripts/license_inventory.py --check
python3 scripts/integration.py \
  --admin target/release/xmqr-admin \
  --broker ../xmqr/target/release/mqtt-broker \
  --report dist/qa-final/integration.json
```

Para browser Windows/WSL, executar primeiro o `--help` do helper
`webapp-testing/scripts/with_server.py`. O ensaio usou o helper para iniciar
`scripts/integration.py --browser-fixture` em WSL, com portas livres
19890/19891/19892 e handoff/STOP novos em
`../.artifacts/admin-qa-final-20261013/`. Em seguida executou o Node
disponível com `scripts/browser_qa.mjs HANDOFF STOP dist/qa-final`, definindo
`NODE_PATH` somente nesse processo para os módulos do bundle.
O helper e o browser encerraram seus processos. Não reutilizar um STOP
existente: preferir diretório novo e portas comprovadamente livres.

O token temporário foi lido da fixture própria em memória pelo browser;
nunca passou por argumento, URL, relatório ou arquivo versionado. O handoff
continha somente o caminho temporário e endereços. Não usar credenciais reais
do operador nesses ensaios.

## Evidências locais e limites

Saídas finais regeneráveis e ignoradas por Git: `dist/qa-final/integration.json`,
`browser.json`, `desktop.png`, `mobile.png`. A primeira campanha retomada
está em `dist/qa-resumed/` e `dist/integration-resumed.json`; não substitui a
campanha final. O cleanup do browser está no
diretório de artefatos acima; o token e o estado Linux temporários foram
removidos pela fixture, não publicados.

O browser viu contagens reais do broker de teste: uma conexão, uma sessão e
uma mensagem retida; filas/erros zero. Esses valores não representam o broker
ou ESP32 do operador. Estado indisponível é diferenciado de zero.

O painel é local e somente leitura de agregados. Não testa alteração ao vivo
de ACL/grupos/papéis/sessões, administração remota, TLS do painel, firmware,
acesso de produção ou exposição em LAN. O CRUD CLI continua no modelo legado
de laboratório; backup/restore continua offline e sujeito a lock. Fases 3/5
do plano do broker não são encerradas por esta validação.

Pacotes, Docker, clone exato, tag, release e confirmação remota de checksums
pertencem a A03 e exigem seus próprios resultados. Não foram inferidos a
partir destes gates.
