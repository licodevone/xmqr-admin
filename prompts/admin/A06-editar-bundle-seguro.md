# A06 rev1 — edição offline do bundle seguro

2026-10-15. Entrada: admin 0.2.0 aceita validação somente leitura; broker
0.14.0 usa bundle TOML v1 privado e reload SIGHUP. A matriz global A02/A03
continua aberta. Core Git permanece pinado; não abrir WAL nem sinalizar PID.

Implementar CLI para listar, criar, atualizar e excluir papéis/grupos e suas
ACLs, além de habilitar/desabilitar identidades. Escrita exclusivamente offline
sob lock privado, preservando arquivo íntegro por temp 0600, sync e rename
atômico. Validar candidato com `DynamicPolicy` do core antes do rename. Recusar
symlinks, permissões/owner incorretos, referências quebradas, duplicatas e
quotas. Saída de listagem mostra apenas nomes/estado e ACL, jamais hashes.

Compatibilidade: bundle v1 existente, campos desconhecidos rejeitados;
nenhuma mudança no broker ou painel. Sem reload automático: operação online
depende de A09. Negativos incluem candidato inválido sem alterar bytes,
exclusão referenciada, arquivo público, symlink e dois escritores. Gates:
fmt, testes, Clippy, release build, licenças e integração CLI em Linux/WSL.

## rev2 — concessões e vínculos de identidade

Para A02 ser útil, o editor precisa também atribuir papéis/grupos a usuários
existentes e substituir concessões diretas de publicação/assinatura. Adicionar
`user set-membership` e `user update-acl` com listas repetíveis. Cada comando
substitui a lista inteira, incluindo a possibilidade de limpar usando lista
vazia; validação do core recusa referências/tópicos inválidos. Manter hashes
e vínculos de certificado intactos.
