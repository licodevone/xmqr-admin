# Versionamento

Admin independente: v0.1.0 inicial, binárioxmqr-admin. Não corresponde à versão
do mqtt-admin legado dentro do broker. Minor acrescenta fatia; patch corrige
defeito. Documentação isolada não muda a versão. Tags identificam código aceito
e não são movidas para incorporar registros de publicação posteriores.

A12 em desenvolvimento após 0.4.0 usa o core Git imutável do broker 0.23.0
(`b884a919f44a3a14c53b973b8a3f3e0aabc663e3`); a versão própria do admin
permanece 0.4.0 até um marco de distribuição testado.

Pedido do mantenedor autoriza commit/push e tags após alterações de código
validadas; prereleases e distribuição quando o marco estiver pronto. Sequência
em prompts/admin/A03-distribuicao.md. Semforce, overwrite oulatest. PacoteDebian
usa revisão separada0.1.0-1; imagens da interface usam xmqr-admin-ui, preservando
as imagens do utilitário legado xmqr-admin:0.11.5.

Primeira publicação concluída2026-10-13: v0.1.0 no commit
7085d7b1996cb54cb70e588ca73b104eb948a324, prerelease22anexos e Hubnumérico.
Commit440e0d1 é checkpoint histórico sem tag: negativos de token foram
fortalecidos por7085 antes da primeira tag, sem mudar binário. Registros
documentais posteriores em main não movem a tag nem mudam a versão.
