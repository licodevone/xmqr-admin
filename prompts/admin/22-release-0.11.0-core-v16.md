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
