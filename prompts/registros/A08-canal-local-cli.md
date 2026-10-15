# A08 — CLI do canal local

- Prompt: [A08 rev1–2](../admin/A08-canal-local-cli.md).
- Entrada: admin 0.3.0 `80b90ea`, broker P107 `7c25e3f`; Linux/WSL Ubuntu 26.04.
- Entrega: `channel status|receipt|reload`, token lido de arquivo privado, socket Unix privado, request/response JSON v1 limitados, deadline, correlação do ID e ID de 128 bits exibido antes do envio. Reload valida bundle, calcula SHA256 e exige geração; timeout/resultado incerto orienta consulta de recibo, sem repetição automática. Pin Git do core inalterado.
- Gates: `cargo fmt --all -- --check`, `cargo test --locked -q` (21 unitários, 6 integrações), `cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked --release`, `python3 scripts/license_inventory.py --check` PASS.
- Integração: `python3 scripts/verify_channel_cli.py --broker ../xmqr/target/debug/mqtt-broker --admin target/release/xmqr-admin` PASS (`CHANNEL_CLI_PASS`) com processos/estado temporários. Cobriu status, reload, recibo, viewer proibido, token público/symlink, ID inválido, conflito de bundle e deduplicação após restart.
- Rev2: `channel sessions|retained` consulta P108 com limites 16/8, cursor e metadados sem payload; integração com broker isolado e viewer negado PASS após novos gates.
- Limites: painel permanece somente leitura; sessões/retained, retenção da auditoria, falhas de storage/concorrência e aceite A09/global fase 5 ainda abertos. Nenhum serviço real modificado.
