# A22 — candidato admin0.11.0

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
