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
- Publicação: pendente neste checkpoint; registrar commit/tag, .deb/tar,
  Docker, hashes, assets e confirmação remota após o ciclo de distribuição.
