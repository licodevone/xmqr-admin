# A05 rev1 — primeira fatia U02: validar bundle seguro offline

2026-10-15. Entrada1c428bb/admin0.1.0, árvore limpa, coreGit pin3606ede preservado.
Objetivo: xmqr-admin bundle validate --file ARQUIVO valida bundle mTLS com
o DynamicPolicy do core pinado; saída fixa sem hashes, certificados, nomes ou ACL.

Escopo: src/bundle.rs/main.rs, testes processo Linux, docs/README/planejamento,
CHANGELOG/manifests0.2.0 e registroA05. Sem repin, WAL, mutação de bundle,
reload/SIGHUP, canal online ou alteração do painel somente leitura.

Contrato: validação Linux offline/read-only, arquivos privados/ownership/symlink
e quotas segundo core. Erro sanitizado sem conteúdo ou caminho do operador.
Version/help não leem arquivos; nenhum serviço instalado é consultado.
U02 continua parcial: CRUD/grupos/papéis e aplicação ainda pendentes.

Aceite: fixture válida com fingerprint+Argon2, inválida/dangling reference,
arquivo público/symlink recusados, arquivo idêntico após comando e nenhum secret
na saída. Gates fmt/test/clippy/build/inventário Ubuntu26, regressãoCLI/painel.
Compatibilidade: novo comando opt-in, estado e pin preservados. Tag0.2.0 só após
aceite; release opcional conforme pacotes. Revisar prompt antes de correções.

## rev2 — formatação

A compilação inicial passou; `cargo fmt --check` apontou apenas a formatação
do código e fixture novos. Aplicar formatter e repetir os gates antes de
avaliar o contrato de segurança e a documentação.

## rev3 — asserts do teste de processo

Os 23 testes passaram. Clippy recusou dois `assert!(...is_empty())` da fixture.
Usar `assert_eq!` com vetores vazios para preservar a verificação e continuar
os gates.

## rev4 — ambiente do inventário

O inventário não encontrou Cargo por falta de PATH no shell de teste; repetir
com o PATH do toolchain antes de interpretar o resultado. Não houve mudança
de dependência nem no pin do core.
