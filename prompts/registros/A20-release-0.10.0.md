# A20 — marco admin independente 0.10.0 publicado

O [prompt A20](../admin/20-release-0.10.0-core-v15.md) antecedeu a atualização de versão (`a4a6f2c`). Fonte exata `35109ff926d8aeca940c2de074890c6b08d3b84b`: 21 testes Rust, 3 de bundle, 3 de CLI, fmt, 11 contratos de distribuição e inventário de licenças locais PASS; [CI PASS](https://github.com/licodevone/xmqr-admin/actions/runs/39020427073). Integração A19 retornou `A19_V15_PASS`, incluindo o admin e broker antigos. O ciclo descartável Ubuntu 26.04 revisão 1→2→1→remoção preservou fixture privada e o binário legado.

Pacote `xmqr-admin_0.10.0-1_amd64.deb`: SHA-256 `1d5d50adbac415a84aead1bcb8c0d4630f587970c4d77cbf930dbd4a714e9f7c`. A tag anotada `v0.10.0` aponta à fonte exata; a [prerelease](https://github.com/licodevone/xmqr-admin/releases/tag/v0.10.0) tem 10 anexos com tamanho e SHA-256 remotos iguais aos locais (`ADMIN_REMOTE_HASH_PASS 10`). Imagem numerada `lepinheiro100/xmqr-admin-ui:0.10.0` executada pelo digest remoto `sha256:c340764c2deb97b6b82a423b49ce4a44ede70b151ba65da3c35a1e227be6843d`, com versão conferida.

O core fixa o broker 0.29.0/estado v15. O ciclo de pacote não reverte v15 a v14: rollback do broker para 0.28.0 requer backup privado v14 anterior. WSL do operador e Docker `latest` permaneceram intactos. Painel de escrita e fases globais continuam abertos.
