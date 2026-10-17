# A18 — compatibilidade core v14 e marco 0.9.0 candidato

- [Prompt A18](../admin/A18-core-v14-release-0.9.md) commitado antes da implementação (`bc09391`). Core Git pinado no commit exato broker `53d86a7e296c158d9247bc5011dec1da49c39b29`; Cargo.lock e inventário de licenças atualizados.
- Ensaio `A18_V14_PASS`: duas sessões compartilhadas, QoS1 sem PUBACK redistribuído ao membro persistente, snapshot v14, backup/verify/restore offline e entrega retomada com identificador próprio; lock ativo, snapshot adulterado e perfil ausente recusados. Suíte local 21 testes unitários, 3 de bundle e 3 de CLI PASS; fmt, Clippy, inventário de licenças e `ADMIN_INTEGRATION_PASS 18` com broker 0.27.0. Gates do commit 0.9.0 exato, pacote, rollback, imagem, tag e prerelease ainda pendentes.
