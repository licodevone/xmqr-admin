# A01 — execução e aceite do CLI independente

Data: 2026-10-13. Prompt de entrada:
[A01 v1.3](../admin/A01-cli-independente.md), preparado antes da implementação;
contexto [A00](../admin/A00-contexto.md). Versão candidata: 0.1.0.

## Entrega e compatibilidade

CLI `xmqr-admin` separado do `mqtt-admin` legado, sem remoção ou alteração do
broker/client. Main/users/state extraídos do broker v0.11.5 com blobs,
adaptações e copyright preservados em `ORIGIN.json`. O core foi reutilizado
por commit Git fixo, não copiado ou resolvido por caminho de checkout.

Mantidos user create/list/update-password/delete, hash-password, fingerprint,
state backup/verify/restore. O novo entrypoint também apresenta versão/help
sem exigir arquivo privado. Operações de usuário permanecem laboratoriais;
não constituem CRUD do secure bundle ou canal administrativo ao vivo.

## Evidência executada

Gates finais Linux/WSL recompilados após A02 v1.3 no diretório atual:
fmt PASS, 21 testes Rust PASS (18 unitários + 3 integração),
Clippy all-targets/Dwarnings PASS,
build release locked PASS e inventário de licenças PASS.

Os testes cobriram CRUD com arquivo privado, rejeição de symlink, escritores
concorrentes sem perda, preservação dos bytes em erro, preservação de vínculo
de certificado ao trocar senha, argumentos inválidos sem eco de valor
sensível e help/version sem configuração. O backup por subprocesso recusou
lock ativo, destino existente e adulteração, e permitiu roundtrip em estado
temporário com dono encerrado.

Integração Python retomada e repetida após reconstrução: 18 cenários PASS,
incluindo arquivo de usuários criado pelo novo CLI e consumido por um broker
independente 0.11.5. Senha inicial aceitou conexão MQTT; após troca/reinício,
senha antiga foi recusada e nova aceita. Delete/list também foram verificados.
Tudo ocorreu em estado e portas próprios, sem dados reais do operador.

## Falhas e resolução

O help inicialmente não explicitava "somente leitura" no recorte do painel;
A01 foi revisado antes de clarear o texto. Clippy solicitou `assert_eq` para
saídas vazias; as exigências permaneceram, sem relaxar warnings.

O PTY inicialmente produziu diagnóstico sanitizado
`exit: 0, prompts: 2, echoed: true`. A01 v1.3 autorizou verificar a corrida
entre prompt e ECHO. O harness agora aguarda ECHO desativado com deadline
antes de enviar a senha de teste, mantendo a asserção de não vazamento.
Não atribuir essa correção ao código do CLI: foi sincronização da automação.

Na retomada após mudança de diretório, três subprocessos Rust falharam ENOENT
porque `CARGO_BIN_EXE_xmqr-admin` embutido em teste compilado apontava para
o diretório antigo. Confirmado o target atual dentro do repositório, apenas
artefatos regeneráveis do pacote foram limpos e recompilados. Os 18 testes
passaram sem mudança de código ou asserções. Detalhes em
[validação](../../docs/validation.md).

## Status e limites

A01: PASS para o escopo CLI/Linux de teste descrito. Origem, MIT própria e
declarações das dependências preservadas; inventário não licencia todos os
terceiros sob MIT. Não há homologação de estado/permissões Windows nativas,
produção ou instalação no host. Novo admin e broker independente foram
executados sem modificar o repositório legado ou remover `mqtt-admin`;
coexistência e lifecycle de pacotes pertencem a A03.

Este registro precede a publicação inicial. Não comprova commit/push,
repositório remoto, tag, release, pacote .deb ou Docker Hub. Saídas locais:
`dist/qa-final/integration.json`; campanhas anteriores preservadas em
`dist/integration-resumed.json` e `dist/qa-resumed/integration.json`.
