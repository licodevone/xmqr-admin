# A16 — compatibilidade core v13 e distribuição 0.8.0

- [Prompt A16](../admin/A16-core-v13-release-0.8.md) commitado antes da implementação. O core Git foi fixado no commit exato `0254a55659b2bdcb02d0abae6057d457e6a49bcf` da tag broker `v0.26.0`; Cargo.lock e inventário de licenças atualizados. Sem dependência de caminho ou ramo móvel.
- Compatibilidade local: `A16_V13_PASS` com associação compartilhada durável, identificador de assinatura e mensagem QoS2 offline preservados por backup/verify/restore; recusa de lock ativo, snapshot adulterado e perfil ausente. 21 testes unitários, 3 de bundle, 3 de CLI, fmt, Clippy, inventário de licenças, 10 contratos de pacote e 3 contratos de export passaram no Ubuntu 26.04. Build release, CI e distribuição ainda pendentes neste ponto.
- [ ] Distribuição própria 0.8.0: versão, pacote Ubuntu 26.04 amd64, imagem Docker numerada, tag/prerelease e hashes/digest remotos. O painel de escrita e demais itens do roadmap continuam abertos.
