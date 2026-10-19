# Painel local e console MQTT do XMQR Admin

O trabalho A23 acrescenta um console MQTT ao painel do `xmqr-admin`. Consulte
o registro A23 e os relatórios de validação antes de considerar essa entrega
aceita ou disponível em um pacote publicado. Uma instalação antiga do admin
não ganha o console apenas ao configurar as variáveis abaixo.

O navegador conversa com a API HTTP local do admin. O admin executa o
`mqtt-client` instalado usando perfis preparados no Ubuntu, sem enviar senhas
ao navegador. O broker continua recebendo e distribuindo mensagens MQTT.
Não é necessário abrir outra porta MQTT ou mudar o firmware do ESP32 para
usar o console.

| Serviço | Endereço habitual | Utilização |
| --- | --- | --- |
| Broker MQTT com TLS | `127.0.0.1:8883` | ESP32, clientes e processos MQTT do admin |
| Monitor do broker | `http://127.0.0.1:9090/ready` | Saúde do broker; resposta HTTP, sem painel |
| Painel do admin | `http://127.0.0.1:9080` | Interface que abre no navegador do Windows |

As portas são configuráveis. `9090/ready` não é o endereço do console MQTT.
O admin e o monitor deste guia ficam restritos a loopback. O acesso do ESP32
ao broker usa o endereço de rede e o certificado já preparados para o projeto;
`127.0.0.1` no ESP32 seria o próprio ESP32.

## 1. Abrir o Ubuntu e conferir os programas

No PowerShell do Windows:

```powershell
wsl -d Ubuntu-26.04
```

Esse comando abre a distribuição Ubuntu. Todos os comandos seguintes são
executados dentro dela, usando o mesmo usuário Linux que executará o admin.

```bash
mqtt-broker --version
mqtt-client --version
xmqr-admin --version
command -v mqtt-client
systemctl status xmqr-broker.service --no-pager
```

Os três primeiros mostram as versões instaladas. `command -v` mostra o
executável que será usado pelo console; o caminho absoluto será configurado
adiante. `systemctl status` verifica o serviço do broker. Não instale nem
substitua pacotes apenas para seguir este guia: use os artefatos do marco
aceito e confira seus hashes conforme a documentação de distribuição.

## 2. Conferir o monitor existente

No arquivo de ambiente do broker, preserve certificados, autenticação e ACL
que já funcionam:

```bash
sudoedit /etc/xmqr/broker.env
```

Se o monitor ainda não estiver habilitado, ajuste:

```dotenv
MQTT_MONITOR_ENABLED=true
MQTT_MONITOR_BIND=127.0.0.1:9090
```

Só após uma alteração nesse arquivo, reinicie o broker:

```bash
sudo systemctl restart xmqr-broker.service
curl -i http://127.0.0.1:9090/ready
```

O reinício interrompe brevemente as conexões MQTT. `curl -i` mostra os
cabeçalhos e o corpo da resposta de saúde; HTTP 200 indica prontidão. Se o
serviço falhar, consulte as últimas mensagens:

```bash
journalctl -u xmqr-broker.service -n 50 --no-pager
```

`-u` seleciona a unidade; `-n 50` limita às últimas 50 linhas;
`--no-pager` imprime diretamente no terminal.

## 3. Preparar credenciais e perfis privados do console

Use os usuários MQTT existentes no projeto: por exemplo, `dht22-api` pode
assinar `dht22/readings`; `dht22-device` pode publicar nesse tópico. As ACL
do broker devem autorizar as operações correspondentes. O console não cria
usuários nem amplia permissões automaticamente.

Prefira um usuário de publicação exclusivo para o painel em um ambiente
compartilhado, em vez de reutilizar a identidade de um dispositivo real.
Os nomes abaixo apenas aproveitam o exemplo do laboratório.

```bash
install -d -m 700 "$HOME/.config/xmqr-admin"
install -d -m 700 "$HOME/.config/xmqr-admin/mqtt"
```

`install -d` cria os diretórios, se necessário. O modo `700` permite acesso
somente ao seu usuário. Mantenha esses arquivos no sistema Linux, não em
`/mnt/d`, onde as permissões dependem da configuração do WSL.

Copie para o diretório privado a CA que valida o certificado do broker:

```bash
install -m 600 "$HOME/.config/xmqr-dht22/ca.crt" \
  "$HOME/.config/xmqr-admin/mqtt/ca.crt"
```

Esse exemplo pressupõe que a CA existente está no caminho mostrado. Ajuste
o caminho de origem ao arquivo que você já usa; não gere outra CA para um
broker que já está configurado.

Crie os arquivos de senha sem colocar os valores no histórico:

```bash
(
  umask 077
  read -r -s -p 'Senha MQTT de dht22-api: ' senha_api
  printf '\n'
  printf '%s' "$senha_api" > "$HOME/.config/xmqr-admin/mqtt/sub.password"
  unset senha_api
  read -r -s -p 'Senha MQTT de dht22-device: ' senha_device
  printf '\n'
  printf '%s' "$senha_device" > "$HOME/.config/xmqr-admin/mqtt/pub.password"
  unset senha_device
)
chmod 600 "$HOME/.config/xmqr-admin/mqtt/"*.password
```

`read -s` oculta a digitação; `-r` preserva barras invertidas. `umask 077`
restringe os arquivos novos. `printf '%s'` grava a senha sem uma quebra de
linha adicional. As variáveis são descartadas ao terminar o bloco. Esses
arquivos guardam a senha MQTT original, e não o hash Argon2id do broker.

Crie o perfil de assinatura:

```bash
nano "$HOME/.config/xmqr-admin/mqtt/sub.toml"
```

Para o listener TLS com senha e sem certificado de cliente:

```toml
version = 1
[connection]
tls_password = true
host = "127.0.0.1"
port = 8883
username = "dht22-api"
ca = "ca.crt"
password_file = "sub.password"
protocol_version = "3.1.1"
```

Use `host = "localhost"` caso esse seja o nome validado pelo SAN do
certificado existente. O nome ou IP precisa constar no certificado; não
desative a validação TLS.

Crie o perfil de publicação:

```bash
nano "$HOME/.config/xmqr-admin/mqtt/pub.toml"
```

```toml
version = 1
[connection]
tls_password = true
host = "127.0.0.1"
port = 8883
username = "dht22-device"
ca = "ca.crt"
password_file = "pub.password"
protocol_version = "3.1.1"
```

Os caminhos relativos de CA e senha partem da pasta de cada perfil.
Para mTLS, remova `tls_password = true` e informe `cert = "client.crt"` e
`key = "client.key"` com os arquivos adequados a cada identidade. Não troque
o modo de autenticação do broker para adaptar o exemplo.

```bash
chmod 600 "$HOME/.config/xmqr-admin/mqtt/sub.toml" \
  "$HOME/.config/xmqr-admin/mqtt/pub.toml"
```

Os perfis devem ser arquivos regulares, privados e sem links simbólicos.
Use também modo `600` nas chaves e arquivos de senha. O console exige perfis
de até 64 KiB e nunca oferece campos HTTP para escolher arquivos arbitrários.

## 4. Conferir o client antes de abrir o painel

Em um terminal Ubuntu, deixe o subscriber ativo:

```bash
mqtt-client sub \
  --profile "$HOME/.config/xmqr-admin/mqtt/sub.toml" \
  --topic 'dht22/readings' --qos 1
```

`sub` assina o tópico; `--profile` carrega conexão e credenciais;
`--qos 1` solicita o nível de serviço da assinatura. Sem `--count`, o
processo permanece recebendo até você pressionar Ctrl+C.

Em outro terminal Ubuntu:

```bash
printf '%s' '{"temperature":24.6,"humidity":58.2}' | \
  mqtt-client pub \
    --profile "$HOME/.config/xmqr-admin/mqtt/pub.toml" \
    --topic 'dht22/readings' --qos 1 --stdin
```

`pub` publica uma mensagem; `--stdin` lê o JSON enviado pelo pipe. O
publisher encerra após concluir a operação. O subscriber deve mostrar o
mesmo JSON. QoS 1 confirma o protocolo com o broker; não comprova que a API,
o banco ou o Google Sheets processaram a leitura.

## 5. Criar o token e iniciar o admin

```bash
if [ ! -e "$HOME/.config/xmqr-admin/access.token" ]; then
  xmqr-admin token-init --file "$HOME/.config/xmqr-admin/access.token"
fi
```

O token autentica o navegador no admin. Ele é diferente das senhas MQTT.
O comando não deve substituir um token existente. Não coloque seu valor em
uma URL nem publique esse arquivo.

No terminal que manterá o admin em execução:

```bash
export XMQR_ADMIN_TOKEN_FILE="$HOME/.config/xmqr-admin/access.token"
export XMQR_ADMIN_BIND=127.0.0.1:9080
export XMQR_ADMIN_MONITOR_BIND=127.0.0.1:9090
export XMQR_ADMIN_MQTT_CLIENT_BIN="$(command -v mqtt-client)"
export XMQR_ADMIN_MQTT_SUB_PROFILE="$HOME/.config/xmqr-admin/mqtt/sub.toml"
export XMQR_ADMIN_MQTT_PUB_PROFILE="$HOME/.config/xmqr-admin/mqtt/pub.toml"
export XMQR_ADMIN_MQTT_LABEL='Laboratório DHT22'
xmqr-admin serve
```

Cada `export` define uma configuração para esse processo. `CLIENT_BIN`
seleciona o executável absoluto; `SUB_PROFILE` e `PUB_PROFILE` escolhem as
identidades; `LABEL` é o nome mostrado na interface, sem credenciais.
Não use um link simbólico para o executável configurado. Se seu comando
for um link, informe o caminho real do binário confiável instalado.

Sem os perfis, a visão do monitor continua disponível e o console informa
que está desabilitado. O processo permanece no terminal; Ctrl+C encerra o
admin e os processos MQTT que ele próprio criou. Nenhum serviço existente
é alterado por esses comandos.

## 6. Usar o navegador

Abra no Chrome, Edge ou Firefox do Windows:

```text
http://127.0.0.1:9080
```

Em outro terminal Ubuntu, leia o token local:

```bash
cat "$HOME/.config/xmqr-admin/access.token"
```

Copie-o apenas para o campo de acesso do painel e desbloqueie a interface.
O token permanece em memória do navegador durante a sessão, sem gravação em
`localStorage`. Não o envie pelo chat.

1. Na visão geral, confira se o monitor está disponível. Indisponibilidade
   deve aparecer como tal; não significa zero conexões ou zero mensagens.
2. No console MQTT, informe `dht22/readings` ou `dht22/#`, escolha QoS 1 e
   conecte. O estado passa a conectado depois da confirmação da assinatura.
3. Com o ESP32 publicando, acompanhe tópico, mensagem e horário de chegada.
   O horário é o recebimento pelo admin, não uma medição do relógio do sensor.
4. No editor de publicação, informe `dht22/readings`, um JSON de teste, QoS 1
   e publique. A identidade do perfil de publicação precisa de permissão.
5. Use desconectar antes de encerrar a sessão. A saída do painel também deve
   encerrar sua assinatura ativa; a assinatura expira se o navegador deixar
   de consultar o admin por 60 segundos.

O painel mantém até 200 mensagens recentes em memória. Ao atingir o limite,
as mais antigas saem da lista. Publicações de texto ficam limitadas a 4096
bytes. A publicação tem prazo de 6 segundos para concluir; uma falha ou prazo
esgotado não é contabilizado como sucesso. Mensagens binárias recebem uma
representação indicada pela interface.
O console não é um histórico durável, serviço de coleta ou substituto da API
Node.js/PostgreSQL do projeto. Uma assinatura é compartilhada pelo processo
local do admin; não há sessões MQTT independentes por aba ou por usuário web.

A visão administrativa continua sendo de consulta: alterar usuários, ACL,
sessões ou retained pelo canal administrativo é uma operação distinta. O
console acrescenta publicação e assinatura MQTT, usando as permissões dos
perfis configurados; não transforma todas as operações de CLI em botões web.

## 7. Diagnosticar falhas

- **Página não abre:** confira se `xmqr-admin serve` ainda está executando e
  se o navegador usa `9080`, em vez do endpoint de saúde `9090/ready`.
- **Token recusado:** use o arquivo indicado em `XMQR_ADMIN_TOKEN_FILE`, sem
  confundi-lo com a senha MQTT. Um token novo exige reiniciar o admin para
  carregá-lo e desbloquear novamente o painel.
- **Console desabilitado:** confira a versão realmente instalada e as
  variáveis de perfil no terminal que iniciou o admin.
- **Conexão ou publicação falhou:** teste o mesmo perfil diretamente com
  `mqtt-client`, verificando serviço, CA/SAN, senha e ACL. O painel mostra
  erros genéricos para evitar expor caminhos e diagnósticos sensíveis.
- **Permissão de arquivo:** mantenha diretórios privados no Linux e arquivos
  regulares com modo `600`; não use perfis ou executáveis via symlink.
- **A assinatura encerrou:** reconecte após corrigir a falha. O console não
  promete reconexão infinita; navegador ausente também encerra o lease.

## Validação reproduzível do A23

Use apenas binários que você acabou de compilar ou artefatos verificados.
O ensaio cria seus próprios diretórios, tokens e processos temporários.
Não acessa os perfis reais do operador nem o broker instalado.

```bash
python3 scripts/verify_mqtt_board.py \
  --admin /caminho/absoluto/xmqr-admin \
  --report /var/tmp/xmqr-admin-a23-novo.json
```

Esse modo verifica o contrato HTTP e o ciclo dos processos usando um client
falso determinístico. Não é evidência de interoperabilidade MQTT real nem de
renderização segura no navegador. O teste inclui a expiração real de 60
segundos, portanto leva mais de um minuto.

Para acrescentar broker e client reais em mTLS, em portas temporárias:

```bash
python3 scripts/verify_mqtt_board.py \
  --admin /caminho/absoluto/xmqr-admin \
  --client /caminho/absoluto/mqtt-client \
  --broker /caminho/absoluto/mqtt-broker \
  --report /var/tmp/xmqr-admin-a23-real-novo.json
```

A integração requer Python 3.11 ou superior, OpenSSL, PTY Linux e binários
compatíveis com perfis TOML/MQTT 5. O relatório registra hashes dos binários,
da fixture e os resultados dos cenários, sem tokens, senhas, certificados ou
payloads reais. Um relatório existente nunca é sobrescrito, mesmo se foi
FAIL. A validação no navegador e os gates Rust são etapas adicionais do
aceite, registradas separadamente.
