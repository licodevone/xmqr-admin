# Painel Windows nativo - A27

O manifest de servico `packaging/windows-service/Cargo.toml` habilita o adaptador
SCM sobre a mesma fonte da CLI. O manifest principal continua sem entrypoint SCM.
Ambos usam verificacoes nativas de arquivos privados.

```toml
version = 1
bind = "127.0.0.1:9080"
monitor = "127.0.0.1:9090"
token_file = "state/token"
users_file = "state/users.json"
```

`serve --config CAMINHO_ABSOLUTO` e o servico aceitam essa configuracao.
Os caminhos relativos sao resolvidos pela pasta do TOML, nunca pelo diretorio
de trabalho do SCM. `users_file` e opcional: sem ele, a variavel existente
`XMQR_ADMIN_WEB_USERS_FILE` continua disponivel; sem ambos, o painel usa bearer.
O TOML guarda apenas caminhos e enderecos. Binds remotos/porta zero, campos
desconhecidos e documentos maiores que 64 KiB continuam recusados.

Provisione o diretorio `state` com DACL protegida: somente a conta operadora,
SYSTEM, Administrators e LocalService podem receber grants. Para SCM,
LocalService precisa Modify no estado e logs, e Read nos arquivos de configuracao.
Para arquivos usados pelo servico, owner deve ser SYSTEM, Administrators ou
LocalService e as concessoes devem ficar somente nesses tres SIDs. `token-init`
executado pelo operador inclui seu SID para uso em console: a provisao elevada
do servico deve remover esse grant do token, ajustar o owner privilegiado e
normalizar a DACL protegida antes de iniciar o SCM. Nao basta adicionar
LocalService mantendo o grant ou owner de outra conta operadora.
O programa recusa permissoes concedidas a Everyone, Users ou Authenticated Users,
mesmo que outro deny as mascare. DACL nula/ausente/vazia e tipos de ACE nao
suportados tambem sao recusados. Arquivos e ancestrais nao podem ser junctions,
symlinks ou outros reparse points.

```powershell
xmqr-admin.exe token-init --file C:\ProgramData\XMQR\admin\state\token
xmqr-admin.exe serve --config C:\ProgramData\XMQR\admin\admin.toml
```

`token-init` nunca sobrescreve um arquivo existente e nao mostra o segredo.
Aplique a privacidade ao diretorio antes de executar: o programa cria o arquivo
vazio, aplica sua DACL privada e a verifica antes de gravar o token.
Com usuarios web configurados, o primeiro acesso faz bootstrap usando esse token
e cria a conta administrativa. Senhas sao persistidas como hashes Argon2id;
os contratos de sessao, CSRF e roles permanecem os mesmos.

Os comandos de usuarios MQTT estaticos tambem usam arquivos privados nativos.
Eles continuam distintos das contas web. O bundle dinamico do core pinado ainda
exige Linux/WSL; comandos de socket Unix e backup/restore nao foram homologados
como recursos nativos por esta entrega.

Stop/Shutdown durante inicializacao seguem para o drain normal. HTTP tem prazo
de encerramento e o console MQTT e encerrado junto. Em console, Ctrl+C e
Ctrl+Break iniciam a parada cooperativa. Testes de HTTP em console nao substituem
o aceite SCM em VM elevada. Veja [registro A27](../prompts/registros/A27-windows-private-runtime.md).
