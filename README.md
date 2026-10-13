# XMQR Admin

Administração independente do ecossistema XMQR, em Rust, com CLI e painel web
local. Primeiro marco experimental **0.1.0**. Licença própria MIT.

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

Não há gestão de sessões/grupos/ACL ao vivo, emissão de certificados, controle
remoto de ESP32, painel de publicação MQTT ou declaração de produção. O painel
é somente leitura e não abre WAL, não modifica configuração e não reinicia
serviços. Client continua sendo quem publica/recebe mensagens MQTT.

## Três projetos, sem conflito

`xmqr` contém o broker e o utilitário legado `mqtt-admin`0.11.5.
Este projeto instala **`xmqr-admin`**, com versão0.1.0 independente, lado a lado.
`xmqr-client` contém o cliente `mqtt-client`0.8.1. Nenhum deles é substituído.
Não é Cargo workspace. Reutilizamos a biblioteca do core por dependênciaGit
fixada no commit3606ede844210997fa733408855099535145ac48, sem pathdependency.
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
