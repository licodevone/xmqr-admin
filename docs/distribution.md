# Distribuição experimental 0.1.0

O `xmqr-admin` é independente do broker e do `mqtt-admin` legado. O pacote novo
não remove nem substitui esse binário. A distribuição só é aceita depois dos
gates registrados: presença destes arquivos não prova publicação nem produção.

## Ubuntu 26.04 amd64 e WSL

Na raiz de um commit limpo, com Rust, Python3, dpkg-dev e compiladores disponíveis:

```sh
python3 scripts/build_distribution.py --output dist/release-0.1.0
```

Gera `xmqr-admin-0.1.0-ubuntu26-amd64.tar.gz`,
`xmqr-admin_0.1.0-1_amd64.deb`, SHA256 e manifesto de origem. O tar contém
`bin/xmqr-admin` e documentação. Seu ELF foi compilado no Ubuntu26: não há
certificação de Ubuntu24, outras distribuições, arm64 ou Windows nativo.

```sh
sha256sum -c xmqr-admin_0.1.0-1_amd64.deb.sha256
sudo apt install ./xmqr-admin_0.1.0-1_amd64.deb
xmqr-admin --version
sudo apt remove xmqr-admin
```

Estes comandos são para execução consciente pelo operador. Os testes do projeto
nunca instalam no host: usam um container descartável. O deb instala apenas
`/usr/bin/xmqr-admin` e `/usr/share/doc/xmqr-admin`. Não fornece units, serviços,
conffiles, usuário de sistema ou scripts de manutenção. Configuração, tokens,
usuários e estado são externos e continuam sob responsabilidade do operador;
remoção não os apaga. A revisão Debian2 existe somente no teste de upgrade,
rollback para1 e remoção, não significa nova versão funcional.

Gates de construção, sem instalar nada no host:

```sh
python3 packaging/test_distribution.py
python3 scripts/build_distribution.py --skip-build --revision 2 --output dist/package-revision2
docker run --rm --network none \
  -v "$PWD/dist/release-0.1.0:/first:ro" \
  -v "$PWD/dist/package-revision2:/second:ro" \
  -v "$PWD/packaging/lifecycle.sh:/lifecycle.sh:ro" \
  ubuntu:26.04 sh /lifecycle.sh /first/xmqr-admin_0.1.0-1_amd64.deb \
  /second/xmqr-admin_0.1.0-2_amd64.deb
```

Use Docker da plataforma que possui esses diretórios (no Windows, caminhos do
Windows com DockerDesktop); não presumir socket Docker no WSL. O script exige
container, Ubuntu26 e root interno, monta artefatos somente leitura e cria
fixtures dentro do container. `--allow-draft` é só laboratório, marca manifesto
como não aceito; releases exigem source_revision real de commit limpo.

A imagem Ubuntu26 minimalista tem exclusões dpkg para documentação. O gate
usa `--path-include=/usr/share/doc/xmqr-admin/*` apenas dentro desse container
para também verificar os avisos instalados; não altera políticas do host.
O arquivo `copyright` contém também a MIT própria verbatim, no caminho
convencional Debian retido por essa política; não é a licença das dependências.
Se um operador configurar exclusões semelhantes, deverá preservar os textos
de licença distribuídos no arquivo deb e tar. A imagem Docker própria copia
esses textos diretamente e não os instala por esse caminho dpkg.

## Docker

```sh
docker build --platform linux/amd64 --build-arg VERSION=0.1.0 \
  --build-arg VCS_REF=COMMIT_COMPLETO \
  -t lepinheiro100/xmqr-admin-ui:0.1.0 .
docker run --rm --network none --read-only --cap-drop=ALL \
  --security-opt no-new-privileges lepinheiro100/xmqr-admin-ui:0.1.0 --version
```

A compilação da imagem é separada: Rust1.99 Bookworm → runtime Bookworm. Não
copiamos um ELF Ubuntu26 para Bookworm, nem presumimos sua compatibilidade ABI.
Execução não-root UID/GID10001, parada SIGTERM, sem credenciais embutidas.
`org.opencontainers.image.licenses=MIT` descreve o código próprio; os textos das
dependências conservam suas licenças originais. Não é relicenciamento do conjunto.

O painel continua loopback. `-p 9080:9080` **não torna esse listener acessível**
pela interface de rede do container. O gate usa namespace compartilhado com
broker e sonda, apenas no laboratório. Não configurar `0.0.0.0`, rede host ou
firewall como atalho. Implantação remota/reverse proxy ainda não é homologada.
Token privado deve ser montado pelo operador como arquivo regular0600 acessível
ao UID10001, e indicado por `XMQR_ADMIN_TOKEN_FILE`; não vai em imagem/envvalor,
argumento, URL ou logs. O token web não substitui as credenciais MQTT.

A imagem nova se chama `lepinheiro100/xmqr-admin-ui:0.1.0`; a imagem legada
`lepinheiro100/xmqr-admin:0.11.5` permanece intacta. Sem tag `latest`. Publicação,
exportação, digest remoto e release ficam com o coordenador após gates limpos.

## Licenças e origem

`collect_notices.py` usa Cargo.metadata/lock e copia LICENSE/LICENCE/COPYING/
NOTICE/COPYRIGHT/AUTHORS/CONTRIBUTORS e license_file declarados byte a byte, incluindo subdiretórios
vendorizados do AWS-LC. `third-party-notices/manifest.json` contém hashes relativos,
origem e declarações sem caminhos privados. O conjunto inclui dependências de
desenvolvimento e outras plataformas; é um superset, não afirma composição exata
do ELF. Arquivos originais e avisos são incluídos em tar, deb e imagem. Inventário
e coletor não substituem auditoria jurídica, SBOM binário ou auditoria completa de
todos os cabeçalhos de fontes vendorizados.

O crate valuable0.1.1 omite seu LICENSE: incluímos o original do
[commit de origem](https://github.com/tokio-rs/valuable/blob/9efc29b6e58cef28f6566a47aa7e142a55fead77/LICENSE),
fixado pela metadata Cargo. O complemento local é validado pelo SHA256 e o
coletor continua offline, sem fabricar licença de dependência.

Os importadores winapi Windows0.4.0 também omitem textos no archive. O superset
conserva LICENSE-MIT/APACHE do winapi0.3.9 do mesmo lock/repositório e registra
esse donor explicitamente; não infere que donor e importadores têm o mesmo commit
nem que importadoresWindows fazem parte do ELFLinux.
