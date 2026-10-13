# Trilha administrativa por prompts

- [x] ~~A01: CLI independente e compatibilidade de usuários/backups.~~ [Evidência](../prompts/registros/A01-cli.md).
- [x] ~~A02: painel local autenticado de leitura e evidências browser/API.~~ [Evidência](../prompts/registros/A02-painel.md).
- [x] ~~A03: publicação inicial aceita, tag/release/.deb/Docker.~~ [Evidência](../prompts/registros/A03-release-0.1.0.md).
- [ ] Próximo: validar/editor de bundle com grupos/papéis/ACL e schema real.
- [ ] Canal administrativo: decisãoA04 e implementaçãoA09 no plano do broker,
  autenticação/RBAC/filas/idempotência/negativos e operações pelo ator.
- [ ] Sessões/retained/clientes: leitura paginada e operações explicitamente
  autorizadas, sem abrirWAL fora do dono.
- [ ] Painel de escrita: integração com canal aceito, confirmações/auditoria,
  acesso por função e rotação de credenciais.
- [ ] Distribuição estendida/serviçoWindows/arm64/acessoHTTPS remoto.

Marcar somente após gates e registro real. A implementação do painel de leitura
foi solicitada explicitamente antes do canal de escrita; não aprova esse canal,
não encerra a fase5 nem suspende B07/B08/C07 do broker/client. MIT preservada.
