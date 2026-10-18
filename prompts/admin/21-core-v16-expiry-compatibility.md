# A21 v1.0 — compatibilidade com expiração durável v16

Preparado em2026-10-18 antes de alterar Cargo/código. Pedido: concluir fase6
dos três sistemas, mantendo prompts, testes, commits e distribuição próprios.
Entrada: admin0.10.0 publicado, HEAD17c3cff, coreGit9215d4c/v15; árvore limpa.
BrokerP227E está implementando clock lógico e intervalo original de expiração
com formato16. Não tratar proposta, testes dirigidos ou candidato como release.

## Escopo e dependências

Somente depois de validar/commitar o coreP227E: fixar seu SHA exato no
Cargo.toml, atualizar Cargo.lock pelo Cargo e conferir origem/versão/licenças.
Não usar pathdependency, HEAD móvel, crate duplicado ou pin fictício.
Preservar CLI, painel de leitura, canal autenticado, locks,0600,no-follow,
fsync,backup offline e destinos novos. Nenhum serviço/estado real do operador.

Arquivos autorizados: Cargo.toml/lock, fixture nova
scripts/verify_core_v16_expiry.py, CI proporcional, docs de compatibilidade
e registroA21. Preservar fixturev15 e seus critérios históricos, sem mudar
v15para16 apenas para obterPASS. Publicação do pin depende do commit real
do broker disponível; esta preparação não prova execução ou publicação.

## Aceite funcional

Produzir estado16 descartável pelo broker candidato, com mensagens retidas,
assinatura durável/offline e Will pendente, guardando intervalo original e
watermark. Backup/verify/restore via admin novo deve preservar bytes/hashes,
campos e entrega MQTT após abrir destino novo. Backup de estado ativo deve
continuar recusado, assim como destino existente, corrupção/symlink e lock.
Testar leitura/migração do estado15 em fixture própria sem inventar intervalo
original ausente; broker antigo deve recusar estado16. Distinguir transporte
de backup bruto por admin antigo de compatibilidade de execução do broker.

Gates Rustfmt/test/Clippy/build locked/inventárioMIT+notices, integração canal
e fixture16, com fontes/hashes/versões. Falha exige revisão antes de corrigir.
Não alterar globalmente Git, WSL, firewall, firmware ou credenciais.

## Marco de distribuição, após aceite

Compatibilidade com novo formato é mudança real, justificando marco próprio
admin0.11.0 se ainda livre. Antes de bump, reconferir tags/remotos. Preparar
prompt de release separado, gerar.debUbuntu26amd64 e imagemHub numerada
lepinheiro100/xmqr-admin-ui, preservar notices e checksums/proveniência,
testar upgrade/rollback em ambiente descartável e artefato remoto por digest.
Tag imutável/prerelease só da fonte validada; não moverlatest/tags antigas.
Rollback executável do broker exige backup15 anterior à primeira gravação16,
não apenas instalar binário antigo sobre estado16. Não anunciar produção.
