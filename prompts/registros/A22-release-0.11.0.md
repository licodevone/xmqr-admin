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
