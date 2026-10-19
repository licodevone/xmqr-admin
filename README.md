# XMQR Admin

## Fonte A24 — autenticação por usuários

Com `XMQR_ADMIN_WEB_USERS_FILE` apontando para arquivo absoluto em diretório
privado 0700, o painel usa contas locais e sessões com cookie HttpOnly e CSRF.
No primeiro acesso, o token local autoriza criar um administrador com senha
escolhida no navegador. Depois, o bootstrap é fechado e o Bearer antigo não
contorna o login. Senhas ficam somente como hash Argon2id com salt aleatório.

Observador consulta métricas; operador também usa o console compartilhado;
administrador gerencia contas. Desativar uma conta revoga suas sessões, e o
último administrador ativo é protegido. Contas web são independentes das
credenciais MQTT. [Configuração e limites](docs/web-auth.md).
O modo token anterior continua quando a variável não é configurada.
A24 ainda é fonte em validação de CI; não foi publicada nova release/pacote.

## Fonte A23 — painel e console MQTT local

O painel agora tem visão geral, console MQTT, operações administrativas e
configuração/acesso. O console pode assinar um filtro continuamente, mostrar
até 200 mensagens recentes e publicar texto até 4096 bytes com QoS 0/1/2 e
retained. Conexão só aparece ativa após SUBACK; publicação só aparece concluída
após sucesso do client, distinguindo envio QoS0 de confirmação MQTT QoS1/2.
Não há confirmação de processamento do dispositivo.

O backend permanece no Ubuntu/WSL e o navegador Windows abre a porta 9080.
9090/ready continua sendo a saúde do broker e 8883 seu listener TLS. O console
usa o `mqtt-client` instalado por perfis privados preparados pelo operador,
sem enviar credenciais ao browser ou abrir listeners novos no broker.
Sem configurar os perfis, o console mostra que está desabilitado.
[Configuração e uso](docs/mqtt-board.md).

A consulta administrativa de métricas continua somente leitura. O envio de
mensagens MQTT é uma operação explícita do console; CRUD de usuários/ACL,
backups e canal administrativo continuam no CLI. Esta mudança de fonte está
em validação, ainda sem nova versão/tag/release ou substituição do pacote 0.11.0.

## Release experimental 0.11.0 — core v16

Compatibilidade A21 com o core público broker0.43.0 fixado em
`ba6ef50204dec2c6385b272e595260b9def1e794`: backup/verify/restore offline
preserva watermark, intervalo original de expiração e Will pendente do
documento16. Migração de estado15 real não inventa intervalo original.
[Aceite A21](prompts/registros/A21-core-v16-expiry-compatibility.md) e
[nota de distribuição](docs/releases/0.11.0.md).

[Prerelease0.11.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.11.0)
publicada com .deb/tarUbuntu26amd64, Docker numerado e14anexos verificados.
Gates e digest no [registro A22](prompts/registros/A22-release-0.11.0.md).
Downgrade de pacote não converte estado16: rollback executável do broker
exige backup15 anterior à migração. Painel segue somente leitura/loopback,
sem serviços automáticos e sem fechar a fase6 integrada por esta versão.

A release experimental 0.10.0 fixa o core do broker P164 para o estado
MQTT v15 e preserva um Will pendente com Message Expiry em backup offline.
O broker 0.29.0 é o par; o broker 0.28.0 rejeita v15. A versão 0.9.0 do
admin copia backup bruto v15, mas não é o par homologado. Para rollback do
broker, preserve backup privado v14 anterior. O painel segue somente
leitura. [Release 0.10.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.10.0) e [aceite A20](prompts/registros/A20-release-0.10.0.md).

A árvore após 0.9.0 fixa o broker 0.29.0 no commit `9215d4c1e6fb59f3ab73d684ecfbaf9ed38e72b2`
para operar com o documento MQTT v15 e preservar, em backup/verify/restore
offline, um Will pendente com intervalo de validade. O admin 0.9.0 publicado
também consegue copiar um backup bruto v15 sem interpretar o documento, mas
o broker 0.28.0 rejeita v15 (`Malformed` no ensaio). Para rollback executável,
guarde um backup privado v14 anterior à primeira gravação v15. A atualização
do admin foi publicada no marco 0.10.0. [Aceite A19](prompts/registros/A19-core-v15-compatibility.md).

A versão experimental publicada 0.9.0 fixa o core Git do broker 0.27.0 e lê
estado v14, inclusive origem durável e redistribuição de cópia QoS1
compartilhada. [A18](prompts/registros/A18-core-v14-release-0.9.md) registra
CI, ensaio offline, pacote, imagem e anexos remotos. Preserve backup privado
v13 antes da primeira gravação pelo broker 0.27.0 para poder voltar a 0.8.0.
[Release v0.9.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.9.0).

A versão experimental publicada 0.8.0 fixa o core Git da tag broker 0.26.0
para ler estado v13, incluindo associação compartilhada durável e entrega QoS2
escolhida. [A16](prompts/registros/A16-core-v13-release-0.8.md) registra os gates.
O backup segue offline e não contém `.mqtt-broker-profile`; para rollback do
estado v13, preserve antes da migração um backup privado v12. A distribuição
0.8.0 passou nos gates e na conferência dos artefatos remotos.

O marco experimental publicado 0.4.0 reúne editor offline do bundle seguro, CLI do canal
administrativo local, auditoria e diagnóstico de recuperação. O painel web
permanece somente leitura.

O marco experimental 0.5.0 fixa o core Git do broker publicado 0.23.0
para validar bundle e backup/restore de estado v10. O backup durável não inclui
`.mqtt-broker-profile`: ao restaurar, escolha e confira o perfil do listener
separadamente, com o broker parado. O broker recusa iniciar `open-lab` sobre
estado restaurado sem marcador; o admin não cria nem copia esse marcador.
O painel permanece somente leitura. A distribuição 0.5.0 passou nos gates [A13](prompts/registros/A13-release-core-0.23.md).

O [marco experimental 0.3.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.3.0)
acrescenta o editor offline do bundle. Ele oferece
`bundle role|group list/create/update/delete`, `bundle user list/enable/disable`,
`bundle user update-acl` e `bundle user set-membership`. Use sempre `--file
ARQUIVO` logo após a ação, `--name` para papel/grupo ou `--username` para
identidade; repita `--publish`, `--subscribe`, `--role` e `--group` conforme a
ação. `update-acl` e `set-membership` substituem integralmente suas listas;
sem opções, limpam as concessões/vínculos correspondentes. A edição exige
bundle privado existente em filesystem Linux, valida o candidato com o core e
faz rename atômico; não sinaliza nem altera um broker em execução. O painel
continua somente leitura. A aplicação online é feita pelo CLI e canal local.

## Aplicar bundle pelo canal local

Use o broker 0.15.0, em Linux/WSL, com bundle
dinâmico e as três variáveis `MQTT_ADMIN_*` descritas na
[documentação do canal](https://github.com/licodevone/xmqr/blob/main/docs/admin-channel.md). Broker e admin precisam
rodar sob o mesmo UID. O token de operador fica em arquivo privado, nunca em
argumento. O token do painel web é separado.

```sh
xmqr-admin channel status --socket /caminho/privado/admin.sock --token-file /caminho/privado/view.token
xmqr-admin channel reload --socket /caminho/privado/admin.sock --token-file /caminho/privado/operator.token --file /caminho/privado/bundle.toml --expected-generation 1
xmqr-admin channel receipt --socket /caminho/privado/admin.sock --token-file /caminho/privado/view.token --id ID_EXIBIDO_PELO_RELOAD
xmqr-admin channel sessions --socket /caminho/privado/admin.sock --token-file /caminho/privado/operator.token --limit 16
xmqr-admin channel retained --socket /caminho/privado/admin.sock --token-file /caminho/privado/operator.token --limit 8
xmqr-admin channel retained-delete --socket /caminho/privado/admin.sock --token-file /caminho/privado/operator.token --topic topico/concreto
xmqr-admin channel session-disconnect --socket /caminho/privado/admin.sock --token-file /caminho/privado/operator.token --client-id ID_EXATO
xmqr-admin channel audit --socket /caminho/privado/admin.sock --token-file /caminho/privado/operator.token --limit 16
xmqr-admin channel audit-rotate --socket /caminho/privado/admin.sock --token-file /caminho/privado/operator.token --archive /caminho/privado/audit-epoch0.json
xmqr-admin audit-verify --file /caminho/privado/audit-epoch0.json
xmqr-admin diagnose --state-dir /caminho/privado/estado --bundle /caminho/privado/bundle.toml --backup /caminho/privado/backup-verificado --socket /caminho/privado/admin.sock --token-file /caminho/privado/view.token
```

`status` informa a geração atual. Edite/valide o bundle antes do `reload`;
o comando valida novamente, calcula SHA256, gera ID aleatório e **exibe o ID
antes de enviar**. Salve esse ID: em timeout, desconexão ou resultado incerto,
consulte `receipt` antes de repetir. Para uma repetição consciente, passe
`--request-id ID` no final do comando `reload` original, conservando bundle e
geração originais. O broker recusa mudanças de conteúdo, geração obsoleta ou
ID conflitante. O recibo persiste no WAL; a política ativa é reaplicada do
bundle no restart. A exclusão de retained também gera ID/recibo durável.
O broker 0.14.0 não lê o novo documento MQTT v5; faça
backup offline antes de atualizar se precisar de rollback.
As listagens retornam apenas metadados, sem payload; `next_cursor` permite
pedir a página seguinte com `--cursor VALOR`. Páginas não formam um snapshot
imutável quando clientes MQTT publicam ou se conectam ao mesmo tempo.
`session-disconnect` encerra uma conexão ativa como desligamento não gracioso:
Will autorizado é publicado, sessão persistente e inbound QoS2 aceito ficam
duráveis. Não apaga a sessão persistente. Ambas as ações exibem ID antes do
envio e aceitam `--request-id ID` para repetição consciente após `receipt`.
`audit` pagina recibos do ator por índice numérico e mostra somente metadados;
`next_cursor` serve como `--cursor` seguinte. Viewer não pode listar a
auditoria. A tabela tem limite de 128. `audit-rotate` exporta todos os recibos
para arquivo novo privado 0600, sincroniza arquivo/diretório e só então pede
ao broker para validar hash/contagem e avançar a época no WAL. Se outra
mutação ocorrer entre exportação e commit, nada é rotacionado; o arquivo
exportado fica para inspeção. `audit-verify` confere o SHA256 interno;
compare-o ao recibo de rotação. IDs antigos não executam após a mudança de
época; o admin gera novos IDs com prefixo da época lida de `status`.
`diagnose` não muda o broker nem abre o WAL ativo: valida bundle e backup
existentes, consulta prontidão/versão, mede espaço livre (piso de 16 MiB) e
retorna JSON sem caminhos ou segredos. Crie o backup offline com broker parado.

Administração independente do ecossistema XMQR, em Rust, com CLI e painel web
local. Versão publicada **0.8.0 experimental**. Licença própria MIT.

[Release v0.1.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.1.0)
publicada com `.deb`/tar Ubuntu26amd64, exportDocker, hashes e evidências.
[Release v0.4.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.4.0)
publicada com CLI da fase 5, tar/`.deb`, export Docker e hashes conferidos;
veja [A11](prompts/registros/A11-release-fase5.md).
[Release v0.5.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.5.0)
distribui o core broker 0.23.0 com estado v10; veja [A13](prompts/registros/A13-release-core-0.23.md).
[Release v0.6.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.6.0)
distribui o core broker 0.24.0 com estado v11; veja [A14](prompts/registros/A14-core-v11-release-0.6.md).
[Release v0.7.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.7.0)
distribui o core broker 0.25.0 com estado v12; veja [A15](prompts/registros/A15-core-v12-release-0.7.md).
[Release v0.8.0](https://github.com/licodevone/xmqr-admin/releases/tag/v0.8.0)
distribui o core broker 0.26.0 com estado v13; veja [A16](prompts/registros/A16-core-v13-release-0.8.md).
[Imagem Docker Hub](https://hub.docker.com/r/lepinheiro100/xmqr-admin-ui):
`lepinheiro100/xmqr-admin-ui:0.8.0` é a versão publicada. Veja [publicação verificada](prompts/registros/A16-core-v13-release-0.8.md)
e [distribuição](docs/distribution.md). Nenhuma tag antiga/latest foi alterada.

## O que existe

- CLI: criar/listar usuários, alterar senha, excluir usuário; hash Argon2id;
  fingerprint de certificado; backup/verify/restore do estado MQTT.
- Painel autenticado: disponibilidade do broker e oito agregados reais do
  monitor HTTP opcional; atualização manual e acesso pelo navegador.
- Proteções locais: token aleatório em arquivo0600, comparação constante-tempo,
  validação de Host/Origin, sem CORS, sem token em URL/storage, no-store/CSP.

CLI CRUD gerencia arquivos legados dos modos password-lab/acl-lab, não o bundle
dinâmico do modo seguro. Senhas são pedidas no terminal, não por argumento.
Mudanças em arquivos legados exigem reiniciar o broker. Backup exige broker
parado, verificação usa a cópia e restore cria um diretório novo.

Há inventário e ações locais de sessão/retained pelo CLI. A fonte A23 acrescenta
o console de publicação/assinatura, executando o client por perfis privados.
O painel não abre WAL, modifica configuração do broker ou reinicia serviços.
Não há emissão de certificados nem controle de firmware do ESP32 pela interface.

## Três projetos, sem conflito

O [planejamento integrado dos três sistemas](docs/planejamento-tres-sistemas.md)
define a evolução administrativa inspirada em Dynamic Security/mosquitto_ctrl:
bundle seguro, grupos/papéis/ACL, canal autenticado, revogação, auditoria e
gestão de sessões/retained. A fase 5 foi aceita no recorte local Linux/WSL
conforme o [aceite](https://github.com/licodevone/xmqr/blob/main/docs/phase5-acceptance.md).
O painel administrativo observa agregados; alterações administrativas usam o
CLI. O console MQTT A23 publica e assina mensagens separadamente dessas ações.

`xmqr` contém o broker publicado 0.43.0 e seu utilitário `mqtt-admin` legado.
O admin publicado é **`xmqr-admin`** 0.11.0 independente, lado a lado.
`xmqr-client` publica `mqtt-client` 0.42.1. A23 ainda é fonte em validação.
Não é Cargo workspace. Reutilizamos a biblioteca do core por dependência Git
fixada no commit ba6ef50204dec2c6385b272e595260b9def1e794, sem dependência local.
[Origem dos arquivos extraídos](ORIGIN.json); [fluxo por prompts](prompts/admin/A00-contexto.md).

## Compilar no Ubuntu/WSL

Rust2024/MSRV1.88 declarado; aceite inicial emRust1.99/Ubuntu26.04amd64.
Não exige uma cópia local do broker para compilar; Cargo baixa o core pinado.

```sh
git clone https://github.com/licodevone/xmqr-admin.git
cd xmqr-admin
cargo build --locked --release
./target/release/xmqr-admin --version
./target/release/xmqr-admin --help
```

O backend e operações de arquivos privados são destinados aLinux/WSL. Windows
nativo ainda não foi homologado; usar o navegador no Windows não é executar
o backend nativamente. Não houve mudança de firewall ou serviço instalado.

## Abrir o painel local

Crie um diretório privado no filesystemLinux, fora do repositório:

```sh
install -d -m 700 "$HOME/.config/xmqr-admin"
./target/release/xmqr-admin token-init --file "$HOME/.config/xmqr-admin/access.token"
export XMQR_ADMIN_TOKEN_FILE="$HOME/.config/xmqr-admin/access.token"
export XMQR_ADMIN_BIND=127.0.0.1:9080
export XMQR_ADMIN_MONITOR_BIND=127.0.0.1:9090
./target/release/xmqr-admin serve
```

Abra `http://127.0.0.1:9080`. Consulte **localmente** o arquivoaccess.token com
seu editor e cole o valor no campo de acesso. Não envie esse valor ao GitHub,
chat, captura de tela ou argumentos de processo. Não há token/senha padrão.
O token fica em memória da página e é limpo ao sair; não é uma identidadeMQTT.
Para invalidar um token, pare o admin, prepare um arquivo novo e reinicie com
ele. Tokens em arquivo existentes não são substituídos por token-init.

No broker, o monitor precisa estar explicitamente habilitado **antes de iniciar**:

```sh
export MQTT_MONITOR_ENABLED=true
export MQTT_MONITOR_BIND=127.0.0.1:9090
```

Não reinicie sua instância automaticamente apenas para seguir este exemplo:
escolha uma janela de manutenção e siga a configuração já utilizada. Monitor
desabilitado/parado aparece como indisponível, não como zero nem sucesso.
Monitor não tem autenticação própria; esta interface não justifica expô-lo.
Somente IP literal loopback, porta não zero e HTTPlocal. Sem proxy/redirect.
O backend limita32 conexões simultâneas,16 requisições e10s de vida de cada
conexão (incluindo headers incompletos/keepalive). Consultas são curtas;
o navegador reabre a conexão quando necessário. Não é interface streaming.
Não exponha painel/monitor na LAN/internet nem configure0.0.0.0.

## Comandos administrativos

```sh
xmqr-admin user create --users-file ARQUIVO --username USUARIO
xmqr-admin user list --users-file ARQUIVO
xmqr-admin user update-password --users-file ARQUIVO --username USUARIO
xmqr-admin user delete --users-file ARQUIVO --username USUARIO
xmqr-admin hash-password
xmqr-admin fingerprint CERTIFICADO.crt
xmqr-admin state backup --state-dir DIR --destination NOVO_DIR
xmqr-admin state verify --backup DIR_BACKUP
xmqr-admin state restore --backup DIR_BACKUP --state-dir NOVO_DIR
```

Leia [segurança e limites](SECURITY.md) antes de operar. Backup contém payloads
MQTT e deve ficar privado. Ele não inclui users/ACL, certificados ou chaves;
esses arquivos exigem cópias privadas separadas. Nunca abrir estado por um
segundo escritor nem restaurar por cima do diretório ativo.

## Desenvolvimento e próximos marcos

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 scripts/license_inventory.py --check
```

Sempre prompt -> implementação -> negativos/gates -> registro -> commit/push ->
tag do código aceito -> release quando pertinente. [Planejamento](docs/roadmap.md).
O painel não encerra fases3/5 do broker. Não cria novas funcionalidades por um
prompt futuro existir. As versões e tags antigas são imutáveis.

Referências técnicas: [Axum](https://docs.rs/axum/latest/axum/),
[Reqwest](https://docs.rs/reqwest/latest/reqwest/),
[XMQR](https://github.com/licodevone/xmqr),
[XMQR Client](https://github.com/licodevone/xmqr-client),
[Mosquitto Dynamic Security](https://mosquitto.org/documentation/dynamic-security/).
