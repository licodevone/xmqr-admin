# A13 — distribuição experimental 0.5.0 do admin independente

- [Prompt A13](../admin/A13-release-core-0.23.md) registrado antes da mudança da versão. Core Git exato do broker 0.23.0 (`b884a919f44a3a14c53b973b8a3f3e0aabc663e3`); A12 já aceito com [CI PASS](https://github.com/licodevone/xmqr-admin/actions/runs/39004091694).
- Candidato 0.5.0: versão de Cargo/lock/Docker sincronizada, painel somente leitura e perfil de listener ainda escolhido separadamente do backup. O probe Docker passa a ler a versão da fonte em vez de exigir 0.1.0.
- Gates locais do candidato: 21 testes unitários, 3 de bundle e 3 de CLI PASS; fmt, Clippy `-D warnings`, build release, inventário de licenças, 9 contratos de pacote e 3 de export PASS. Binário release responde `xmqr-admin 0.5.0`; integração v10 offline e `CHANNEL_CLI_PASS` com broker 0.23.0 passaram. Probe Docker compilado, ainda sem ensaio de imagem.
- Commit exato, CI, ciclo de pacote, imagem, hashes, tag e prerelease: **pendentes**. Não declarar 0.5.0 publicada até completar e conferir a distribuição.
