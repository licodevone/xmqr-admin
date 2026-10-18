# A21 v1.3 — compatibilidade com expiração durável v16

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

## Revisão v1.1 — fixture independente antes do código

Agente da fixture possui apenas scripts/verify_core_v16_expiry.py e registro
A21. Coordenador mantém Cargo/lock/pin/versão/CI/distribuição. CLI exige
--broker, --admin, --legacy-broker e --report, todos explícitos; relatório
novo não substitui FAIL. Usar Python3.11+stdllib e recursos próprios em
/var/tmp, ambientes MQTT isolados, loopback/portas efêmeras e processos
próprios com deadlines. Não usar defaults de binário/pasta dos scripts antigos.

Seed real: retainedTTL31, assinatura persistente offlineQoS1 com TTL31 e
Will pendente delay120/MessageExpiry37. Inspeção somente após parar o broker:
parser bounded de MQSNAP01/CRC32 e envelope TOML, não simples busca textual
por version=16. Conferir watermark/original nas mensagens novas, mensagem
do Will sem prazo até publicação, hashes e manifesto do backup/restore.
Preservar ausência do marcador de perfil no backup; reinstalar somente o
marcador criado pela própria fixture antes de abrir a cópia restaurada.

Gerar outro estado com ELF legado real e exigir formato15; não reescrever
versão nem inventar campo original. Backup/restore em cópia, abrir pelo novo
broker e conferir16/intervalo original ausente nas mensagens legadas, dados
preservados e MQTT após restore. Broker15 deve recusar16 sem mudar os hashes
snapshot/WAL da cópia. Fixturev15 existente permanece inalterada.

Negativos reais: backup ativo/lock, destino existente/symlink, snapshot
corrompido e entrada de backup symlink, sem saída/destino parcial aceito.
Relatório guarda apenas status/versões/hashes/contagens, sem caminhos privados,
payloads, chaves, certificados, senhas ou logs brutos. Marker inequívoco
A21_CORE_V16_EXPIRY_PASS só depois do conjunto completo real aprovado.
Contratos unittest ficam no próprio script: parser truncado/CRC/duplicata/
trailing/bounds, propriedades MQTT e manifesto adulterado. Contratos ou
--help/AST não são wirePASS. Aguardar ELF16 e admin pinado do coordenador
para executar integração; não instalar nem compilar por esta fixture.

## Revisão v1.2 — validar a representação persistida antes do oráculo

Review do coordenador encontrou possível erro na fixture: o serializer
hex_bytes do broker grava payload como string hexadecimal, mas a primeira
check_document e seu selftest construíam listas de inteiros. Esses oito
contratos não demonstravam correspondência com um snapshot real.

Antes da correção, produzir seed privado com o ELF P227D legado congelado,
SHA256 dca22b2624b7f36175d4e939c640b6f5730220bcc9510580eef9054a6af8ef57,
e admin0.10 existente, conferir documento15/original ausente e executar
check_document(document,15,None). Preservar resultado real FAIL, se houver,
em relatório novo sanitizado com hashes, tipos e motivo, sem dump de estado
ou payload. Não chamar isto de teste de execução do formato16.

Se confirmado, corrigir apenas representação no oráculo: exigir string hex
de comprimento par, caracteres0-9a-f, limite explícito e bytes decodificados
iguais aos sintéticos enviados. Recusar lista, espaços, sinal, hex inválido,
ímpar e bytes diferentes. Selftest estruturado deve usar literais hex
independentes atravessando parser snapshot/TOML, não repetir list(payload).
Manter v15/16/intervalos e negativos anteriores, repetir contratos e seed15
em relatório GREEN separado. Nenhum runtime, Cargo, pin, CI ou serviço real
é alterado; integração16 permanece NOT_RUN até ELFs do coordenador.

## Revisão v1.3 — launcher do selftest, não resultado MQTT

Seed15 real confirmou FAIL fixture_message_bytes_and_qos e tipos str nas
três mensagens. Novo selftest de TOML hex literal também falhou antes do
fix,1FAIL/0.005s. A primeira edição de representação introduziu um parêntese
extra no dict aninhado do selftest anterior; unittest parou com SyntaxError,
exit1 antes de coletar testes. Corrigir somente essa sintaxe e repetir os
mesmos nove contratos e seed15; não chamar erro de import de regressão no
broker nem apagar o FAIL anterior. Oráculos e escopo v1.2 preservados.
