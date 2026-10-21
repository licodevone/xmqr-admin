# A27 - execucao privada Windows nativa

Data: 2026-10-21. Base: arvore A26 existente, sem commit/push/instalacao de servico.
Prompt: [A27](../admin/A27-windows-private-runtime.md).

## Implementacao

`windows-permissions = 0.2.4` fornece descritores/ACLs seguros por handle.
`winsafe = 0.0.29`, feature advapi, fornece SID real do token do processo.
GetSecurityInfo recebe SE_FILE_OBJECT explicitamente; o trait generico da
dependencia usa UNKNOWN_OBJECT e falhou com erro 87 nos testes reais.
Ambos sao pinados exatamente e usados nos dois manifests; unsafe proprio segue
proibido. Owner e cada ACE allow devem ser usuario atual, SYSTEM, Administrators
ou LocalService. DACL ausente/nula/vazia, owner desconhecido, grants amplos,
ACE callback/object e reparse points sao recusados. Nao se deduz ACL por readonly.
Ancestrais sao abertos de cima para baixo sem compartilhamento de delete e
permanecem vivos durante aberturas e substituicoes atomicas.

Token-init usa create_new e aplica DACL protegida no arquivo vazio antes do token.
Usuarios web/estaticos usam locks e escrita por candidato privado sincronizado.
No Windows, o candidato e sincronizado antes da troca no mesmo diretorio; nao
se promete semantica de directory-fsync Unix. Arquivo TOML aceita users_file
opcional, relativo ao TOML; ausencia preserva comportamento de ambiente/token.

SCM reaproveita o modulo atualizado do broker: Stop retido durante inicializacao,
status inicial serializado com handler e Ctrl+Break em console. Shutdown HTTP
continua limitado e encerra o console MQTT.

## Validacao

Windows x64 MSVC, Cargo/Rust 1.99.1, 2026-10-21. Gates usam CARGO_INCREMENTAL=0,
CARGO_BUILD_JOBS=2, CARGO_HOME temporario privado xa27 e TMP/TEMP sob target.
O primeiro cache compartilhado perdeu manifests de dependencias durante a sessao;
foi restaurado de arquivos .crate com checksums verificados. Novo target de Clippy
contornou erro de ambiente os1224 em arquivos de fingerprint mapeados.

| Comando/evidencia | Resultado |
| --- | --- |
| cargo fmt --all -- --check | PASS |
| cargo fmt --manifest-path packaging/windows-service/Cargo.toml -- --check | PASS |
| git diff --check | PASS |
| cargo test --manifest-path packaging/windows-service/Cargo.toml --locked --offline | 31 PASS, zero ignorados |
| cargo test --locked --offline --all-targets | 29 unitarios + 2 CLI PASS |
| cargo clippy --manifest-path packaging/windows-service/Cargo.toml --locked --offline --all-targets -- -D warnings | PASS |
| cargo clippy --locked --offline --all-targets -- -D warnings | PASS |
| cargo build --manifest-path packaging/windows-service/Cargo.toml --locked --offline | PASS |
| cargo build --locked --offline | PASS; CLI normal --version 0.11.0, marker SCM recusado com exit 2 |
| CARGO_NET_OFFLINE=true python -X utf8 scripts/license_inventory.py --check | PASS |
| --windows-service-capabilities no binario de servico | schema 1, component admin, service_entrypoint true |

Os testes incluem token privado/nao sobrescrita, owner/DACL nula/vazia/ampla,
arquivo e parent publicos, aliases ADS/dispositivos DOS, junction real verificada
e recusada pelo erro especifico do guard, HTTP Host/Origin/bearer/body/method/slots,
usuarios MQTT estaticos concorrentes e bootstrap/login/reabertura de usuarios web.
Criar junction por cmd/PowerShell foi negado pelo ambiente; o teste usa o junction
real de compatibilidade Windows Documents and Settings, com fixture alternativa
configuravel por XMQR_TEST_REPARSE_DIRECTORY, sem ignorar o teste.

Integracao executada pelo agente principal: nove cenarios PASS com broker/cliente
reais, incluindo GET /api/status sem token 401, bearer 200 com broker ready,
users_file explicito com bootstrap/cookie/status 200, senha incorreta 401 e banco
persistido apos restart. Prova em xmqr/target/p242-runtime-debug-proof.
Binario testado: target/a27-windows/debug/xmqr-admin.exe, SHA256
4df87bd49dad04bc16bed9dff7db329e04d546a63296f6512ab6677744d16ab0.
O rebuild final apos ajuste equivalente do cfg de directory-sync tem SHA256
fe0e17461b323b42d7d7ee38df232c8951b9693b649f818775b87800cde37243.
CLI normal em target/a27-cli/debug/xmqr-admin.exe, SHA256
34d62abfb2cce45bd64ed25b24541f916a7b358b740e2d33e3f104f7ae6077ad.

Provisao para SCM deve normalizar owner para SYSTEM/Administrators/LocalService
e DACL protegida somente com esses SIDs. Token-init em console inclui o SID do
operador atual; esse grant e owner de operador nao sao aceitos por um processo
LocalService diferente. O teste de descritor verifica essa transicao; nao se
declara execucao SCM sob LocalService nesta maquina.

## Limites

O core Git pinado ainda recusa bundle dinamico privado fora de Linux/WSL;
nenhuma guarda foi removida. Canal por socket Unix e backup/restore mantem seus
contratos anteriores. Execucao console HTTP e testes de arquivos privados nao
equivalem a homologacao de install/start/stop SCM, que requer VM elevada.

Nenhum segredo entra em fonte, argumentos, logs ou este registro.
