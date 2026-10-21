# A26 - serviço Windows experimental do painel

- Prompt: [A26 v1.3](../admin/A26-windows-service.md), preparado antes do código;
  isolamento de dependência revisado após bloqueio real offline.
- Pedido: controlar admin instalado pela bandeja; paralelização autorizada.
- Data: 2026-10-20, Windows nativo, Rust/Cargo 1.99.1, PowerShell.
- Entrada: 2b6f245c30efb6d57af85b70d56e645d7a3b9b71 / 0.11.0, árvore limpa.

## Implementação

Adaptador SCM opt-in, parada encaminhada ao encerramento HTTP/console existente,
Running somente após bind e inicialização. TOML versionado de endereços/token
por caminho absoluto, limite 64 KiB, campos desconhecidos recusados e loopback
preservado. `serve --config` permite usar a configuração explícita em console.
Logs de ciclo de vida têm mensagens fixas e categorias de erro, sem segredos.

O token privado continua exclusivo de Unix: execução nativa falha fechado antes
do Running. Nenhuma flexibilização de ACL, core Git ou formato persistido.
Fontes main.rs/config.rs/web.rs/windows_service.rs, docs, prompt e manifest
experimental foram alterados/adicionados. Manifest principal preserva todas
as dependências e Cargo.lock; somente adiciona check-cfg da feature experimental.
packaging/windows-service/Cargo.toml habilita o adaptador usando a mesma fonte.
Seu lock não foi fabricado; build nativo de serviço permanece bloqueado.

## Validação real

| Comando/checagem | Resultado |
| --- | --- |
| cargo fmt --all -- --check | PASS, saída 0 |
| cargo fmt --manifest-path packaging/windows-service/Cargo.toml -- --check | PASS, saída 0 |
| git diff --check | PASS, saída 0 |
| cargo check --offline --all-targets (antes do isolamento) | BLOCKED: windows-service ausente do cache |
| cargo test --locked --offline | 13 PASS / 5 FAIL; teste novo config estrito/limitado/loopback PASS |
| cargo clippy --locked --offline --all-targets -- -D warnings | FAIL: 10 avisos Windows preexistentes em bundle_edit/token/web_auth |
| cargo build --locked --offline --release | PASS, saída 0; avisos Windows preservados |
| cargo check --manifest-path packaging/windows-service/Cargo.toml --offline | BLOCKED: windows-service ausente no cache |
| Smoke executável release --version | PASS, saída 0, versão 0.11.0 |
| Smoke executável normal --windows-service-capabilities | PASS: recusado, não é falsamente anunciado como serviço |
| Smoke serve --config usando fixture em target/a26-smoke | PASS: erro Unsupported de token Unix antes de escutar |
| Alinhamento dos manifests por tomllib | PASS: versão/dependências/core pin idênticos |
| CARGO_NET_OFFLINE=true python scripts/license_inventory.py --check | BLOCKED: r-efi 5.3.0 ausente no cache metadata |
| wsl.exe --list --quiet | BLOCKED: Wsl/E_ACCESSDENIED |

Falhas de teste: dois binds de socket negados (10013) e três operações de
usuários que exigem Linux/WSL. Não alterar esses módulos nem suprimir a guarda
para obter resultado artificial. Testes/Clippy falhos não são aceite Windows.
Sem instalação de serviço, VM, permissões do host ou release.

O primeiro smoke com TemporaryDirectory padrão foi BLOCKED por PermissionDenied
no temp redirecionado do sandbox. Conforme v1.3, a fixture foi escrita dentro
de target/a26-smoke, sem segredo e sem modificar permissões; o smoke de recusa
foi então aprovado. --version e recusa do marker já haviam passado.

O build experimental declara capabilities JSON schema=1, component=admin,
service_entrypoint=true; ainda não executado por falta da dependência. O marker
não comprova suporte ao token privado Windows. STOP precede o lock de status;
callback não faz IO e erros são categorizados em mensagens fixas sem conteúdo.

## Comprovação P239 — 2026-10-21 (v1.4/v1.5)

Revisões preparadas antes da execução; HEAD 2b6f245 e alterações anteriores
preservados. Cargo gerou packaging/windows-service/Cargo.lock de verdade,
com windows-service 0.8.1; o core Git continua em ba6ef502, sem alterar seu pin.
Corrigidos collapsible_if no callback STOP e needless_return no bloco Windows
de web::shutdown, sem mudança de comportamento. Repetição offline Windows
x64 MSVC/Rust 1.99.1: fmt, build --locked, marker real, --version, 1 teste de
status e 3 testes de configuração PASS. Clippy --all-targets -D warnings ainda
FAIL por 10 avisos Windows preexistentes em bundle_edit/token/web_auth; os dois
lints novos desapareceram, sem supressões.

Config relativa recusada (saída 1); dispatcher fora do SCM recusado com erro
1063 (saída 1); serve --config em fixture própria confirmou Unsupported de
token privado Unix (saída 1). Esse negativo preserva a proteção, não comprova
painel nativo operacional. Nenhum serviço instalado/iniciado.

Runner e relatório agregado no broker:
[P239](../../../xmqr/prompts/registros/P239-windows-services-tray.md) e
[comandos/hashes](../../../xmqr/docs/results/p239-windows-service-proof-20261021.json).
Executável comprovado: ../xmqr/target/p239-service-proof/x86_64-pc-windows-msvc/debug/xmqr-admin.exe.
Os bloqueios de dependência/build acima são históricos. Token/ACL nativos,
controle SCM e operação do painel como serviço continuam sem aceite.
