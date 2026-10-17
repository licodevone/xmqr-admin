# Trilha administrativa por prompts

- [x] ~~A01: CLI independente e compatibilidade de usuários/backups.~~ [Evidência](../prompts/registros/A01-cli.md).
- [x] ~~A02: painel local autenticado de leitura e evidências browser/API.~~ [Evidência](../prompts/registros/A02-painel.md).
- [x] ~~A03: publicação inicial aceita, tag/release/.deb/Docker.~~ [Evidência](../prompts/registros/A03-release-0.1.0.md).
- [x] ~~A06 parcial: editor offline de bundle v1, papéis/grupos/ACL, vínculos e
  habilitação/revogação de identidades, com validação do core antes do rename.~~
- [x] ~~A05/U02 parcial: validar bundle seguro offline, privado e sem escrita via CLI.~~
- [x] ~~Canal administrativo A09 local: autenticação/RBAC/filas/idempotência,
  negativos, recibos e operações pelo ator.~~ [Aceite](https://github.com/licodevone/xmqr/blob/main/docs/phase5-acceptance.md).
- [x] ~~Sessões/retained/clientes: leitura paginada e operações explicitamente
  autorizadas, sem abrir WAL fora do dono.~~
- [x] ~~A05/A08: auditoria exportável/rotacionável e diagnóstico redigido de
  recuperação, backup, espaço e versões.~~
- [x] ~~A12: core pinado no broker 0.23.0, backup/restore offline do estado v10 e
  perfil de listener explícito; CI PASS.~~ [Evidência](../prompts/registros/A12-core-broker-0.23-compat.md).
- [x] ~~A13: distribuir admin independente 0.5.0 com core 0.23.0, pacotes e imagem numerados, tag e prerelease verificadas.~~ [Evidência](../prompts/registros/A13-release-core-0.23.md).
- [ ] Painel de escrita: integração com canal aceito, confirmações/auditoria,
  acesso por função e rotação de credenciais.
- [ ] Distribuição estendida/serviçoWindows/arm64/acessoHTTPS remoto.

O aceite da fase 5 não inclui painel de escrita, acesso remoto, Windows nativo,
arm64 ou produção. Essas entregas permanecem futuras. MIT preservada.
