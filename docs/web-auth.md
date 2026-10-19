# Acesso ao painel por usuário e senha

O trabalho A24 introduz contas web no `xmqr-admin`, ativadas explicitamente
por configuração. Consulte o registro de aceite A24 e a versão distribuída:
um pacote antigo não implementa esse recurso apenas porque uma variável foi
adicionada ao ambiente.

As contas web são próprias do painel. Elas não criam usuários MQTT, não
alteram ACL do broker e não substituem os perfis usados para publicar ou
assinar tópicos. A configuração de broker, client e console continua descrita
no [guia do painel MQTT](mqtt-board.md).

| Acesso | Credencial | Finalidade |
| --- | --- | --- |
| Primeiro cadastro web | Token local existente | Autorizar a criação do primeiro administrador |
| Uso normal do painel | Usuário e senha web | Entrar com as permissões da conta |
| Publicação e assinatura MQTT | Perfis privados do client no Ubuntu | Autenticar os processos MQTT no broker |

Não há usuário ou senha padrão. Você escolhe sua senha pessoal no navegador.
Não envie senha, token ou arquivos de usuários pelo chat.

## 1. Preparar o Ubuntu/WSL

No PowerShell do Windows, abra sua distribuição:

```powershell
wsl -d Ubuntu-26.04
```

No Ubuntu, confira o executável e sua versão:

```bash
command -v xmqr-admin
xmqr-admin --version
```

Use um binário ou pacote do marco que inclua A24 e tenha aceite registrado.
O painel roda em `http://127.0.0.1:9080`; `http://127.0.0.1:9090/ready` é
somente a saúde do broker. Autenticação web não exige modificar o serviço
MQTT, o firmware do ESP32 ou a porta TLS 8883.

Crie os diretórios privados, no sistema Linux:

```bash
install -d -m 700 "$HOME/.config/xmqr-admin"
install -d -m 700 "$HOME/.config/xmqr-admin/auth"
```

`install -d` cria o diretório quando necessário; `-m 700` restringe o acesso
ao seu usuário. Evite armazenar a base em `/mnt/d`, onde a aplicação das
permissões Unix depende da configuração de montagem do WSL.

Use o token já existente. Caso ainda não tenha criado um token:

```bash
if [ ! -e "$HOME/.config/xmqr-admin/access.token" ]; then
  xmqr-admin token-init --file "$HOME/.config/xmqr-admin/access.token"
fi
```

Esse bloco cria o token somente se o arquivo não existir. O token autoriza
o primeiro cadastro; depois disso, o acesso diário usa a conta web.

## 2. Ativar o modo de contas web

Encerre com Ctrl+C a instância do admin que está no terminal, caso esteja
usando a porta 9080. Isso encerra a assinatura MQTT pertencente ao console;
o serviço do broker continua independente.

No terminal Ubuntu que iniciará o admin:

```bash
export XMQR_ADMIN_TOKEN_FILE="$HOME/.config/xmqr-admin/access.token"
export XMQR_ADMIN_BIND=127.0.0.1:9080
export XMQR_ADMIN_MONITOR_BIND=127.0.0.1:9090
export XMQR_ADMIN_WEB_USERS_FILE="$HOME/.config/xmqr-admin/auth/users.json"
xmqr-admin serve
```

`XMQR_ADMIN_WEB_USERS_FILE` ativa o novo modo e aponta um caminho absoluto.
O admin cria e mantém sua base de usuários nesse local. Não crie manualmente
um arquivo vazio nem edite o JSON com o servidor em execução. O diretório
deve ser privado e o arquivo, quando existir, deve ter modo 600 e ser regular,
sem links simbólicos.

As variáveis do console MQTT, como `XMQR_ADMIN_MQTT_SUB_PROFILE` e
`XMQR_ADMIN_MQTT_PUB_PROFILE`, continuam sendo configuradas no mesmo terminal
conforme o guia MQTT. O modo de contas web sozinho não habilita publicação
nem assinatura se esses perfis não foram preparados.

Sem `XMQR_ADMIN_WEB_USERS_FILE`, permanece o modo anterior de token local.
Com a variável presente, o token Bearer antigo não permite contornar a tela
de login nem as permissões dos usuários.

## 3. Criar o primeiro administrador no navegador

Abra no navegador do Windows:

```text
http://127.0.0.1:9080
```

Se a base ainda não tiver usuários, a interface apresenta o cadastro inicial.
Em outro terminal Ubuntu, consulte seu token existente:

```bash
cat "$HOME/.config/xmqr-admin/access.token"
```

`cat` mostra o conteúdo apenas no seu terminal. Copie-o para o campo de
autorização do cadastro inicial. Não coloque o token no endereço da página.

Informe um nome de usuário, seu nome visível e uma senha exclusiva para o
painel. A senha é escolhida por você, diretamente no navegador. Não reutilize
a senha do ESP32 ou de outros serviços.

- Usuário: 3 a 32 caracteres ASCII, com letras, números, `_` e `-`; o
  armazenamento normaliza letras para minúsculas.
- Nome visível: 1 a 80 bytes.
- Senha: 8 a 128 bytes, sem caracteres de controle. Caracteres acentuados
  podem ocupar mais de um byte; prefira uma senha longa e exclusiva.

O primeiro cadastro cria um administrador. Após essa criação, o cadastro
inicial é fechado e não pode criar outro administrador com o token local.
Não existe senha padrão escondida no programa.

## 4. Entrar e administrar contas

Nas próximas visitas, entre usando seu usuário e sua senha web. O token de
cadastro não faz parte do login diário.

| Papel | Consulta métricas | Usa console MQTT | Gerencia contas web |
| --- | --- | --- | --- |
| `viewer` | Sim | Não | Não |
| `operator` | Sim | Sim | Não |
| `admin` | Sim | Sim | Sim |

O administrador pode listar, criar, ativar e desativar contas na área de
usuários. Ao criar uma conta, escolha o papel correspondente ao trabalho da
pessoa. A listagem não mostra senhas nem hashes. Alteração de senha e
recuperação por e-mail não fazem parte desta entrega.

Desativar uma conta revoga suas sessões existentes e impede novos logins.
O sistema recusa desativar o último administrador habilitado, para evitar
deixar a instalação sem administração.

O console MQTT é compartilhado pelos operadores e administradores da mesma
instância. Eles utilizam os perfis MQTT configurados no servidor; esta versão
não cria uma conexão MQTT ou credencial de broker diferente para cada usuário
web. Um `viewer` não pode consultar payloads ou operar esse console.

Ao sair, use o botão de encerramento da sessão. Logout revoga o cookie web.
A limpeza da assinatura MQTT do console segue o comportamento indicado na
interface e o lease local do A23; encerrar uma conta web não altera o serviço
do broker nem as conexões MQTT de outros programas.

## 5. Como o acesso é protegido

As senhas são armazenadas como hashes Argon2id com salt aleatório, sem a
senha original. O arquivo de usuários não serve para recuperar uma senha
esquecida; ele deve permanecer privado mesmo contendo hashes.

O navegador recebe um cookie de sessão aleatório, com `HttpOnly`,
`SameSite=Strict` e `Path=/`. O cookie não contém sua senha ou o token local.
Ele não é colocado em URL, `localStorage` ou `sessionStorage`. O servidor
guarda o digest da sessão em memória; reiniciar o admin exige novo login.

A sessão expira após 8 horas, ou depois de 30 minutos sem atividade. As
operações de escrita também exigem uma proteção CSRF por sessão e verificam
o endereço de origem. A interface envia esses dados automaticamente; você
não precisa copiar um CSRF para usar os botões.

O acesso HTTP deste guia é restrito ao próprio computador, em loopback.
Não abra essa porta na rede nem use redirecionamento para expor login HTTP.
Uma implantação remota com HTTPS exige planejamento e validação próprios.

Há limites de tentativas e de trabalho simultâneo para cálculo das senhas.
Se receber indicação de excesso de tentativas, aguarde antes de tentar de
novo. O banco permite até 64 contas e tem limite de 64 KiB.

## 6. Arquivos, reinício e diagnóstico

```bash
ls -ld "$HOME/.config/xmqr-admin/auth"
ls -l "$HOME/.config/xmqr-admin/auth/users.json"
```

O primeiro comando deve indicar permissões de diretório `drwx------`; o
segundo deve indicar arquivo `-rw-------`. Não exiba o conteúdo da base em
capturas de tela ou relatórios públicos.

Somente uma instância pode manter o mesmo banco de usuários aberto. Se
outro processo já o está usando, encerre a instância correta antes de
iniciar outra. Não apague o lock para tentar contornar a proteção.

Se o arquivo estiver inválido, o admin recusa sua abertura. Não o apague
para liberar um novo cadastro: isso perderia contas e poderia mascarar um
problema. Preserve uma cópia privada, pare o admin e restaure um backup
confiável do banco conforme o procedimento operacional da instalação.

Para copiar uma base saudável com o admin parado:

```bash
(
  umask 077
  cp -- "$HOME/.config/xmqr-admin/auth/users.json" \
    "$HOME/.config/xmqr-admin/auth/users.backup-$(date +%Y%m%d-%H%M%S).json"
)
```

`umask 077` restringe arquivos novos; `cp --` copia o arquivo e trata os
caminhos como argumentos; `date` gera um nome de backup com data e hora.
Guarde também o token local de forma privada, separada do uso cotidiano do
navegador. Este guia não oferece um comando para redefinir senha web.

Se perdeu a única senha administrativa, preserve a base e solicite um
procedimento administrativo específico. Os comandos legados `user create`
e `user update-password` destinam-se aos usuários MQTT; não são uma ferramenta
de reset das contas web do A24.

## Validação do A24

Na pasta do repositório, com um binário compilado e verificado:

```bash
python3 scripts/verify_web_auth.py \
  --admin /caminho/absoluto/xmqr-admin \
  --report /var/tmp/xmqr-admin-a24-novo.json
```

A fixture usa Python padrão no Linux/WSL, cria credenciais aleatórias em
diretórios temporários privados e inicia somente processos próprios em
portas temporárias. Não toca no broker instalado, nos perfis MQTT ou em
usuários reais.

O relatório novo registra hashes do executável e da fixture e os cenários
PASS/FAIL. Não contém senhas, cookies, tokens, CSRFs ou hashes de senha.
Relatórios existentes são preservados, inclusive os FAIL.

O ensaio cobre bootstrap único, login, papéis, CSRF, revogação, logout,
reinício, lock, arquivos inválidos e limites. Não substitui os testes internos
dos prazos de sessão, dos parâmetros KDF e do limite preciso de sessões, nem
a validação da interface em um navegador real. Esses resultados e as
regressões do console A23 devem constar separadamente no aceite.
