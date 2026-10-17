# A14 — core v11 e distribuição 0.6.0

- [Prompt A14](../admin/A14-core-v11-release-0.6.md) criado antes do pin e da versão. Broker `v0.24.0` publicado, tag anotada no commit `8dde5fda09935fe9d72b3524fe898da39f6bb6f1`.
- Etapa de compatibilidade: core Git exato atualizado para 0.24.0, sem dependência de caminho. Integração offline descartável v11 com opções No Local/RAP persistidas: backup bloqueado durante broker ativo, verify/restore, perfil externo explícito, retomada QoS0/1/2 e snapshot adulterado recusado (`A14_V11_PASS`). Admin ainda em 0.5.0 nesta etapa; distribuição 0.6.0 pendente de gates próprios.
- Etapa de compatibilidade aceita na fonte `b9443e2d97c9cd3df52fb261a9ab0fb5d5c57d25`, [CI PASS](https://github.com/licodevone/xmqr-admin/actions/runs/39006513294): 21 testes unitários, 3 de bundle, 3 de CLI, fmt, Clippy, build release, inventário de licenças, 10 contratos de pacote e 3 de export PASS. `CHANNEL_CLI_PASS` com broker 0.24.0; sem mudança no painel.
- Candidato 0.6.0: versão Cargo/lock/Docker, changelog, README e [nota de release](../../docs/releases/0.6.0.md). Gates e distribuição da versão ainda pendentes.
