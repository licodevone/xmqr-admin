# Versionamento

Admin independente: v0.1.0 inicial, binárioxmqr-admin. Não corresponde à versão
do mqtt-admin legado dentro do broker. Minor acrescenta fatia; patch corrige
defeito. Documentação isolada não muda a versão. Tags identificam código aceito
e não são movidas para incorporar registros de publicação posteriores.

Pedido do mantenedor autoriza commit/push e tags após alterações de código
validadas; prereleases e distribuição quando o marco estiver pronto. Sequência
em prompts/admin/A03-distribuicao.md. Semforce, overwrite oulatest. PacoteDebian
usa revisão separada0.1.0-1; imagens da interface usam xmqr-admin-ui, preservando
as imagens do utilitário legado xmqr-admin:0.11.5.
