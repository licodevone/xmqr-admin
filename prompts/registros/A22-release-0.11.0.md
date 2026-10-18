# A22 — admin0.11.0 publicado

2026-10-18, clone próprio/base0612c48, branchrelease/0.11.0. Prompts A21v1.4
04ad140 e A22v1.1 c7606b1 precederam mudanças. A21pin e3723b6 aceito com
27Rust/9contratos/9casos reais/canalCLI e demais gates, ainda como admin010.
Somente depois Cargo gerou o bump011, sem atualização de dependências.

ELFdebug011 congelado do cache próprio:
SHA25673222f355c77b5d8abe0964c1f24b6bcbc86db52e95a1fe4d07dad561d3d45e2,
--version011PASS. Gates finais/CI/distribuição/publicação ainda em execução;
nenhum item pendente é consideradoPASS. Release/Docker pesados aguardam
coordenação de memória com o broker; não reiniciar host nem apagar caches.

## Primeiro CI — FAIL de preparação preservado

Run39141063998/ba7de18, jobUbuntu26/build_distribution, falhou antes de
compilar: Git2.53 recusou status por dubious ownership (exit128, step1).
[Observação sanitizada](../../docs/evidence/admin-a22-ci-preflight-fail.json).
PromptA22v1.2 precedeu a correção. Somente env do job descartável recebe
safe.directory do workspace exato por GIT_CONFIG_COUNT, sem '*' ou config
global/host. Gate de fonte limpa e --allow-draft proibido permanecem.
Nova CI deve passar integralmente; não converter o primeiro resultado emPASS.

## Segundo CI — Git PASS, collector FAIL preservado

Run39141338423/source99a7248: validate PASS (fmt/Rust/Clippy/release/inventory/
11contratos-distribuição/3export); core-v16 PASS (9contratos/9reais/canalCLI).
Ubuntu26 preflightGit PASS e releasebuild PASS em2m36s; depois collector
falhou exit1 porque pharos0.5.3 não contém LICENSE separado e o admin ainda
não possuía sua regra exata de declaração original README. Nenhum pacote
desse run foi aceito/publicado; integração e licenças são gates distintos.

Local99 reproduziu o mesmo FAIL em dist/a22-expected-notices, saída parcial
preservada. PromptA22v1.3/37caea0 precedeu novos testes/fix. RED:7métodos,
dois subcasos positivos ERROR (pharos e ws_stream_tungstenite); os negativos
não eram prova de implementação. Depois das duas regras específicas iguais
às já revisadas em C131/broker, GREEN7/7 em0.142s: positivosoriginais e
recusas por commit, README alterado, licença/repositório, versão distinta,
symlink e path_in_vcs WS. Sem fallback genérico nem mudança Cargo/runtime.

Collector completo em saída nova dist/a22-expected-notices-v2 PASS:
225dependências/418textosoriginais; manifest SHA256
72d2fee41094f66167987cc4e46a995d1961196ca0cf6388d5f6822d083cc4fc.
Collector SHA2568772f4f8ec62291cb4fb47c6d2dc3d380c61acdfc0da93b6e70f3b7cdc7e9afd;
contrato novo SHA256cf7d2d104727e0798af38c6543038748d28a14f7ee3ac80d3a181abeef21938d.
11contratosdistribuição/3export/actionlint1.7.12 PASS após o fix. Nova CI da
fonte exata ainda necessária; não há tag, .deb ou imagem011 publicados.

## Aviso de artefato A21 detectado antes da publicação

Run39141889950/sourcefe8cff5, core-v16 PASS, mas upload-artifactv4 avisou que
.evidence/*.json não encontrou arquivos. A21 imprime JSON/marker somente
após escrever o relatório; action.yml oficial do action consultado viaAPI
explicita include-hidden-files=false por padrão. O aviso é de transporte de
evidência, não FAIL do protocolo. Sem aceitar anexo inexistente.

PromptA22v1.4 precede seleção do único .evidence/a21-v16.json sanitizado,
inclusão explícita de ocultos e erro obrigatório por ausência. Upload nativo
também exige arquivos, sem fallback. Nova fonte requer CI e download real
dos anexos. Nenhum runtime/Cargo/ELF alterado.

## CI2f e pacote nativo — PASS; reexecução do canal — FAIL separado

Run39142145338/source2f3609fb3e09bf54bc16ed38f0c57eb36b6edac9 completo
SUCCESS nos3jobs. AnexoA21 baixado:9/9PASS, harnessb6a07b5f igual à fonte,
versões043/011/legacy042. Manifest nativo: source2f/coreba6/draftfalse,
225dependências/418notices. .deb fba58aa6dd2d5c958f704f9f39dd73e537170d61afa351db65d11429c718488d
(1301328bytes); tar4b8a9f1e43a6b7acef9ff05582f5601afc957f2029b02760b8905aedea67581d
(1705044bytes). Checksums/sidecars, caminhos/links, ownership/permissões,
proveniência/lock e418textos de tar/deb conferidos PASS independentemente.

ELFreleaseCI90dfc90ae0e05669c6ecc2f3db87a30d66823f29a10d5317194583fea2e94e43
executou --version011 e A21real9/9 em3.269s com brokerfce/legacyD. Ciclos
Ubuntu26 descartáveis:010→011→010→remove/purge PASS e revisão011-1→2→1→remove
PASS, preservando hashes/permissões do token/fixture e nome legado. Revisão2
foi criada com --skip-build/ELFCI e não será publicada. Nada instalado no host.

Depois, canalCLI adicional com esse ELFrelease e brokerdebugfce retornou1:
ConnectionRefusedError em new-after-restart seguido por AssertionError em
cleanup. [FAIL sanitizado preservado](../../docs/evidence/admin-a22-channel-restart-fail.json).
Promptv1.5/b4a23a4 precede diagnóstico instrumentado, ainda sem fix. Não
classificar readiness/exitcode antes de evidência, nem publicar com esse gate
pendente. Fonte/ELFs/pacotesCI anteriores permanecem imutáveis.

Diagnóstico posterior sem editar o harness:15execuções completas PASS,
75start/stop com exit0/socket removido. Wrappers observaram apenas tempo,
status e flags de logs, sem inserir espera/retry ou mudar os oráculos.
Starts observaram admin.sock em aproximadamente0.405–0.600s. A falha original
não reapareceu; ausência de RUST_LOG deixou as flags de listener/shutdown
false, portanto não permite inferir esses estágios. Isto não prova a causa
do FAIL anterior nem a corrige retroativamente. Nenhum fix de readiness foi
aplicado. Próximo recorte: contrato determinístico de socketadmin disponível
antes da respostaMQTT, seguido de gate de resposta dentro do prazo e cleanup
que preserve a exceção primária. Publicação011 continua retida por esse gate.

## Contrato de lifecycle do harness — RED/GREEN independente

Promptv1.6/ac8ab84 precedeu teste e fix. RED determinístico: processo/socket
simulados, socketadmin disponível mas MQTT atrasado; start antigo retornou
sem probe, connect.call_count=0 em vez de3. Isto comprova uma lacuna do
contrato do harness, não reproduz a causa desconhecida do FAIL real anterior.

Novo start exige CONNACK5 à conexão anônima3.1.1 (sem criar sessão), PIDvivo
e deadlineabsoluto8s; stop continua exigindo exit0/socketgone e prazo, inclusive
recusando-15. Drain4096 em thread retém apenas63bytes para detectar tokens
sintéticos entre blocos, sem guardar logs. Erros de leitura/fechamento ou
criação do reader sãoFAIL/cleanuppróprio; finally não mascara exceção primária.
Nenhum cenário/adminassert, coreRust, versão, lock ou ELF alterado.

GREEN12contratos em0.016s e actionlintPASS. Dois smokes reais intermediários
(adminrelease90df e debug732 com brokerfce) CHANNEL_CLI_PASS, sem atribuição
causal retroativa. Fonte final será novamente exercitada e exigirá CI/pacotes
próprios; artefatos2f permanecem históricos. Hashes finais dos helpers:
verify_channel_cli.py d27d67d556b7292827ca8d548765620e9023286729f49db9e22b09f0fa3a3e14,
test_channel_lifecycle.py6fe70557ba366e9dc8d44c10cfffca34aa4e7fc1d2b26c8012eac9b71d850a57.

## Fonte final4483 — gates e publicação verificados

Fonte imutável4483a356196db93f99340bc7aa3b3cabfc4af810, [CI39143572778
SUCCESS3/3](https://github.com/licodevone/xmqr-admin/actions/runs/39143572778):
27Rust(21+3+3), fmt/Clippyalltargets/release/inventory;33contratosPython
(11distribuição+7notices+12lifecycle+3export),9contratosA21,9casosreais e
canalCLI. ArtefatoCIUbuntu26 e anexoA21 baixados; source4483/coreba6/draftfalse
verificados. Nenhum teste ignorado no conjunto próprio do admin.

Quatro smokes finais do canal (dois com admin732debug/dois90dfrelease) PASS;
A21 com ELFrelease extraído do pacote final:9/9PASS em3.1584s. Dois ciclos
finais descartáveis, usando o pacote exato4483:010→011→010→remove/purge e
011rev1→2→1→remove PASS, conservando token0600/fixture/legacy. Revisão2 apenas
teste, não publicada. Conteúdo, ownership/modos, ausência de scripts/serviços,
sidecars e418notices de tar/deb conferidos independentemente.

Docker compilado separadamente Bookworm-j1, sem ELFUbuntu copiado:
ELF31cc950e06dbd4347cab3f84ae1cc5844cba9e2286ac98f52231267717890bc5;
native90dfc90ae0e05669c6ecc2f3db87a30d66823f29a10d5317194583fea2e94e43.
225dependências/418textos originais, manifest72d2fee4 igual ao nativo.
ADMIN_DOCKER_PASS15 local e os mesmos15 novamente após pull pelo digestHub;
não contar como30cenários distintos. Fixtures usaram brokerE48b7855e e
client038b025a82a em namespace/volume privados, sem portas publicadas.

Hub numérico `lepinheiro100/xmqr-admin-ui:0.11.0`, digest remoto
`sha256:eadbf642fc99a1e6eb5ad15a020a211c31c1637f575f52c75617919af107b007`,
inspeção/pull/versão e smoke verificados. Tag anotada imutávelv0.11.0:
objeto69aa486bb2c68d3b24de34f6095f4ecc7193bcac → source4483. [Prerelease](https://github.com/licodevone/xmqr-admin/releases/tag/v0.11.0)
com14anexos baixados novamente em pasta nova: nomes, tamanhosAPI, bytes/SHA,
sidecars e manifestos iguais aos locais (A22_REMOTE_HASH_SIZE_PASS14).

.deb finalSHAfe03ffc858e7583e7d8ce6808b4f56495c688beb69d47870db7fe0c1b8b55469,
1301324bytes; taradde78e36462f470026baf2406600f7fd1ce211e14ef4bf5f0fe0b6cb251ce31,
1705041bytes. Dockerarchivefb1300e62da0cddbcd2865f3701e0e9a3c54489137930a12fbe30fb67cc3c9c6,
33014118bytes. Dois relatórios sanitizadosPASS e checksums acompanham anexos.
[Resumo de evidências](../../docs/evidence/admin-a22-release-0.11.0.json).

Falhas anteriores permanecemFAIL/aviso, sem reescrita ou atribuição indevida
de causa. O contrato determinístico de readiness foi corrigido separadamente.
Fase6 global/produção/Windowsnativo/arm64/perda de energia não são aceitos por
esta release. Não houve instalação/serviço no host, segredo real, latest,
substituição de mqtt-admin legado ou edição do clienteC128. Registros após
tag não movem a fonte aceita. Originaladmin poderá avançar somenteFF/clean.
