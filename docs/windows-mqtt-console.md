# Console MQTT no Windows - A28

O Admin pode receber os perfis do console diretamente no mesmo TOML de
`serve --config CAMINHO_ABSOLUTO` ou do servico Windows. A secao e opcional:
sem ela, as variaveis `XMQR_ADMIN_MQTT_*` continuam com o comportamento anterior.
Com ela, o Admin usa somente seus valores para configurar o console, sem
alterar o ambiente global e sem completar campos a partir de variaveis.

```toml
version = 1
bind = '127.0.0.1:9080'
monitor = '127.0.0.1:9090'
token_file = 'state/token'
users_file = 'state/web-users.json'

[mqtt_console]
client_bin = 'C:\Program Files\XMQR\Client\mqtt-client.exe'
sub_profile = 'console/sub.toml'
pub_profile = 'console/pub.toml'
label = 'XMQR local'
```

O exemplo apenas mostra o formato; preserve os caminhos de token e contas da
instalacao existente. `client_bin` e `sub_profile` sao obrigatorios nessa secao.
Sem `pub_profile`, a publicacao usa o mesmo perfil da assinatura. Sem `label`,
o nome e `XMQR local`. Caminhos relativos sao resolvidos a partir da pasta do
TOML; prefira caminhos absolutos no Windows, sem segmentos `..`. Cada caminho
deve ter de 1 a 4096 bytes e nenhum caractere de controle. O label tem de 1 a
80 bytes, tambem sem controles. Campos desconhecidos e tipos errados falham
fechados; o arquivo completo continua limitado a 64 KiB.

Os arquivos dos perfis devem existir, ter ate 64 KiB e passar a validacao
privada Windows, incluindo proprietario, DACL restrita do arquivo e da pasta
imediata e ausencia de reparse nos ancestrais. O servico LocalService precisa
ler os perfis e os arquivos de credenciais por eles referidos. Uma permissao
individual de outro usuario na pasta privada pode ser recusada pelo servico,
mesmo que esse usuario consiga abrir o arquivo em sua propria sessao.

Use perfis normais do `mqtt-client` (`--profile`), com referencias a CA,
certificado, chave e senha em arquivo. Nao coloque senha ou token web no TOML
do Admin. O perfil de console nao e o documento de lancamento
`--service-config` do Client e nao usa seu sink `messages.ndjson`.

Configurar o console deixa-o habilitado e desconectado; a assinatura somente
inicia pela acao autenticada do painel. Os controles de acesso, CSRF, origem
local, lease de 60 segundos, fila de 200 mensagens e payload de 4096 bytes
permanecem. PUB/SUB usam identidades de processo diferentes para nao substituir
a assinatura do Client instalado. Confirmacao MQTT nao comprova processamento
pelo dispositivo de destino. O Admin nao cria credenciais nem amplia ACLs do
Broker automaticamente.

O manifesto `packaging/windows-service` incorpora `assets/admin.ico` pelo
compilador `rc.exe` do Windows SDK no link de `xmqr-admin`. O build script usa
`RC`, PATH, WindowsSdkDir/WindowsSDKVersion ou SDK10.0.26100.0 instalado. Exige
target Windows MSVC, sem dependencia Rust nova ou alteracao PE depois do link.

Validacao e limites: [registro A28](../prompts/registros/A28-windows-console-configuration.md).
