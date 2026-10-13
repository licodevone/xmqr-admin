# XMQR Admin 0.1.0 — experimental

Primeira distribuição independente do administrador XMQR, em Rust:

- CLI: criar/listar usuários, alterar senha, excluir usuário, hash/fingerprint
  e backup/verify/restore offline, compatível com o core XMQR0.11.5 pinado.
- Painel web local autenticado e somente leitura, com disponibilidade e oito
  agregados reais do monitor opcional. Atualização manual, sem expor payloads.
- Token privado em arquivo0600/pai privado; Host/Origin/Bearer, CSP/no-store,
  conexões limitadas32/10s e requisições16, limites/deadlines do monitor.
- Pacote novo Ubuntu26amd64, tar nativo e imagem separada DebianBookworm
  Linuxamd64, não-root10001. Textos/licenças/avisos originais de terceiros
  incluídos; código próprio continua MIT.

As evidências e checksums acompanham a release. Fonte por prompts em
[GitHub](https://github.com/licodevone/xmqr-admin),
[validação](https://github.com/licodevone/xmqr-admin/blob/v0.1.0/docs/validation.md),
[distribuição](https://github.com/licodevone/xmqr-admin/blob/v0.1.0/docs/distribution.md) e
[planejamento](https://github.com/licodevone/xmqr-admin/blob/main/docs/roadmap.md). A tag identifica o código aceito; registros
posteriores de publicação podem aparecer em commits documentais de main.

## Utilização

No Ubuntu26amd64, baixar o deb e seu SHA256 desta release, conferir checksum
antes de executar `sudo apt install ./xmqr-admin_0.1.0-1_amd64.deb`.
Não há serviço instalado/ativado automaticamente. `xmqr-admin --help`
explica os comandos; o [README](https://github.com/licodevone/xmqr-admin/blob/v0.1.0/README.md) explica token e painel.
O token deve ficar no filesystemLinux privado, fora do repositório.

Imagem: `lepinheiro100/xmqr-admin-ui:0.1.0`,
[Docker Hub](https://hub.docker.com/r/lepinheiro100/xmqr-admin-ui).
O arquivo `xmqr-admin-ui-0.1.0-docker-linux-amd64.tar.gz` é um exportDocker,
não o tar do programa nativo. O manifesto identifica ambos.
Conferir SHA256; usar `docker load` para o export, ou `docker pull` para a
imagem. A imagem mantém loopback: publicar uma portaDocker não é suficiente
para alcançar o painel. Namespace compartilhado foi testado apenas em
laboratório, sem portas publicadas. Não ampliar bind/firewall para contornar.

## Compatibilidade e limites

Não substitui `mqtt-admin` legado, o broker0.11.5 ou o client0.8.1. Nenhuma tag
ou imagem numérica antiga foi alterada. UsuáriosCLI são do modelo legado
laboratorial; arquivos alterados requerem restart coordenado do broker.
Backup exige broker parado e restore cria destino novo. Não editar WAL
manualmente ou por segundo escritor. Configuração/estado/tokens externos
não pertencem ao pacote, e remoção do pacote não os apaga.

Atualizações futuras: parar o admin, preservar token privado e configuração,
instalar pacote compatível aceito, conferir versão e reiniciar manualmente.
Rollback deste marco não migra o formato do core; preservar backup anterior
antes de operar estado. Nunca restaurar sobre o estado ativo.

Ainda não há controle online de grupos/papéis/ACL/sessões, canal administrativo
A09, TLS/RBAC do painel, acesso remoto, certificação Windowsnativo/Ubuntu24/
arm64 nem produção. A consulta ao monitor não conclui fases3/5 do broker.
O ESP32 continua consumidor MQTT, não servidor deste painel.
