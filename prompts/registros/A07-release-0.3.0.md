# A07 — publicação admin 0.3.0

- Prompt: [A07](../admin/A07-release-0.3.0.md).
- Fonte: editor offline A06, sem canal A09 ou escrita online.
- Gates após versão 0.3.0 no Ubuntu26/WSL: `cargo fmt --check`, 21 testes
  unitários + 6 integrações Rust, Clippy `-D warnings`, build release,
  inventário MIT/dependências e nove contratos de distribuição PASS.
- Pacotes 0.3.0 revisão 1 e 2 gerados em Ubuntu26. O ciclo Debian em
  container descartável passou instalação, upgrade de revisão, rollback,
  remoção e preservação de token/legado. O gate histórico com versão 0.1.0
  fixa foi parametrizado antes desta repetição.
- Publicação: commit `4cf82b19f89989c3db273ee84a403847d3232b9c`, tag anotada
  `v0.3.0` e [prerelease GitHub](https://github.com/licodevone/xmqr-admin/releases/tag/v0.3.0).
  Dez assets (.deb/tar Ubuntu26, imagem Docker exportada, manifestos e hashes)
  conferidos por SHA-256 entre cópia local e GitHub, sem diferenças.
  `lepinheiro100/xmqr-admin-ui:0.3.0` publicado com digest remoto
  `sha256:b71edd42f220beb13edd080548103336a4883ec830ec82aa973c8c765ce75ec1`.
  `latest` não mudou. Rollback: versão 0.2.0; edição offline nova deixa de
  estar disponível, e o bundle privado existente continua sob o operador.
