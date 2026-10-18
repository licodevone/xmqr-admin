# A22 v1.0 — distribuir admin0.11.0 compatível com estado16

Preparado em2026-10-18 antes do bump e publicação. Pedido vigente inclui
prompts antes de funcionalidade, commit/push, tag imutável por marco aceito,
release quando pertinente, .debUbuntu26amd64 e DockerHub numerado próprio.

## Entrada

Admin0.10.0 publicado e core pin9215d4c/estado15. A21 preparou fixture16 com
nove contratos aprovados e seed15 real; integração16 continua NOT_RUN.
BrokerP227E ainda resolve revisão/gates finais. Consulta remota não achou
refs/tags/v0.11.0 nesta preparação; reconferir antes da publicação. Não
apresentar este prompt como versão existente, teste realizado ou compatibilidade.

## Pré-condição e escopo

Primeiro concluir core funcionalP227E com testes completos e commit público
exato; A21 autoriza pinCargoGit e fixture. Não usar pathdependency nem HEAD
móvel. Após A21 real aprovado, bump0.11.0 por compatibilidade funcional nova,
Cargo.lock gerado peloCargo, Dockerfile/version declarations, CI proporcional,
README/VERSIONING/docs/releases/0.11.0.md e registroA22. Preservar três
repositórios independentes, legacy mqtt-admin e licenças/notices originais.

## Gates

Fmt/test locked/Clippy all-targets -Dwarnings/release/inventário, export e
distribuição existentes. A21 com broker16, admin novo e broker15 congelados:
backup/verify/restore, bytes/intervalos/watermark, MQTT após restore, migração
real15 sem inventar intervalo, recusa16 pelo15 sem mudar snapshot/WAL e
negativos de lock/ativo/corrupção/symlink/destino existente. Canal CLI real
com broker novo. Identificar fonte, ELFs, versões/SHA e duração; sem skip.

Fase6 integrada é gate separado, não inferida pelo número0.11.0. Falhas ficam
no registro e exigem revisão do prompt antes de correção. Sem tocar serviços,
firewall, ESP32, certificados, tokens ou estado real do operador.

## Pacotes e publicação

.debUbuntu26amd64 com notices e proveniência; ciclo instalação/upgrade/
rollback/remoção em container próprio preservando configurações/estado.
Downgrade executável de broker16 requer backup15 anterior à migração, não
somente instalar versão antiga; admin antigo transportar bytes não é prova
de compatibilidade do core. Painel web permanece somente leitura local.

Imagem lepinheiro100/xmqr-admin-ui:0.11.0 numerada, nonroot/config segura,
smokes por digest após push. Tag anotada imutável v0.11.0 e prerelease
somente da fonte validada por CI, pacote, inventário e integrações. Conferir
tamanho/SHA dos anexos remotos e digestHub. Não mover latest/tags anteriores,
não publicar segredo ou estado, não anunciar produção/certificação integral.

## v1.1 — distribuição em clone próprio e CI de compatibilidade

Execução autorizada no clone `.artifacts/p227-admin-release011`, branch
release/0.11.0; original0612c48 continua preservado. A21v1.4 fixou core
público ba6ef50204dec2c6385b272e595260b9def1e794. Não bump antes de A21
real aprovado. Depois do aceite, Cargo gera o lock011 sem atualizar outras
dependências. Atualizar Dockerdefault011/-j1, README/VERSIONING/distribution,
CHANGELOG se existente, nota e registros, com limites de rollback explícitos.

CI manterá os gates atuais e acrescentará jobA21 (checkouts exatos do coreba6
e legacy24d219816fdac3e47115bcb2e59edd55b0c11407, debugELFs próprios,
novecontratos+novecasos reais e canalCLI) e jobUbuntu26 empacotando fonte
limpa, conferindo versão e ausência de serviços/legacy replacement. Todos
os pins públicos, sem pathdependency; registrar hashes e revision do lock.

Novo packaging/test_version_lifecycle.sh pode complementar, não substituir,
o ciclo histórico de revisãoDebian1→2→1: instalação010 publicada, upgrade011,
rollback010 e remove/purge em containerUbuntu26 próprio sem rede, preservando
token0600/configuração/fixture privada e o nome legacy mqtt-admin. Isso não
converte estado16 para15; compatibilidade MQTT/backup é gateA21 separado.
Nenhuma instalação no WSL ou serviçohost. Versão/tagHub011 reconferida livre
antes de publicar; runtimeDocker separadoBookworm e nonroot10001 preservados.

Nativebuild/notices/revisão1e2 e Dockerlabels/export devem usar o mesmo commit
limpo exato. CIubuntu/Rust/A21 e smokeDocker por digest antes de tag/prerelease.
Exportes/manifests/checksums remotos conferidos por download em pasta nova.
Registros pós-tag só documentam publicação, não movem fonte/tag aceitas.

## v1.2 — FAIL do Git protegido no containerCI, antes do build

Run39141063998/ba7de18 jobUbuntu26 falhou antes da compilação:
build_distribution.source_revision → git status exit128/dubious ownership.
Git2.53 vê o workspace do runner com outro proprietário; o safe.directory
temporário de actions/checkoutv4 não ficou efetivo no processo Python.
Preservar esse FAIL; não usar --allow-draft nem remover o gate de fonte limpa.

Corrigir somente o ambiente do job descartável: GIT_CONFIG_COUNT/KEY/VALUE
adiciona safe.directory igual ao GITHUB_WORKSPACE exato para os processos do
job. Sem '*', sem configglobal ou alteração de ownership do host. Verificar
git rev-parse e status antes do builder; manter fonte limpa/revision/lock
obrigatórios. Repetir CI completo da nova fonte. Runtime/pin/ELF011 não mudam.
