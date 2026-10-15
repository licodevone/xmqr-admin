# A11 — release experimental 0.4.0

- Prompt: [A11 rev1](../admin/A11-release-fase5.md). Fonte imutável: commit `7e3304629af5901828125a3d1ac2b32f37e7c4e6`, tag anotada `v0.4.0`, [prerelease](https://github.com/licodevone/xmqr-admin/releases/tag/v0.4.0).
- Gates Ubuntu26/WSL: 21 testes unitários/6 integrações, fmt, Clippy `-D warnings`, build release, inventário MIT, 9 testes de distribuição, 3 de export e `CHANNEL_CLI_PASS` cruzado com broker0.15.0. [CI](https://github.com/licodevone/xmqr-admin/actions/runs/38783877530) PASS.
- Distribuição: tar e `.deb` Ubuntu26amd64, imagem `lepinheiro100/xmqr-admin-ui:0.4.0`; dez anexos (pacotes, export Docker, manifestos e SHA256) tiveram nome/tamanho/digest conferidos pela API GitHub. `.deb` baixado novamente confirmou SHA256.
- Digest remoto Docker Hub `sha256:1d1299ee3409fe216c42dc5104e5e7e0cbb29b8f5da4bf233fd8eec712f6f159`, executado por digest com `xmqr-admin 0.4.0`. Sem mover `latest`, tags antigas ou alterar core Git pinado. Painel web permanece somente leitura; Linux/WSL local é o escopo aceito.
