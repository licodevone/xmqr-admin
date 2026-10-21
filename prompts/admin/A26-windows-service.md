# A26 v1.3 - adaptador Windows Service do painel

Revisão preventiva v1.1: configuração explícita de serviço evita depender do
ambiente da sessão interativa; serve --config também permite verificar a mesma
configuração em console. Configuração de autenticação/console opcional continua
sob contrato existente; nenhum segredo é aceito no TOML de endereços.

Entrada: 0.11.0, 2b6f245c30efb6d57af85b70d56e645d7a3b9b71, árvore limpa.
Pedido: controle do admin instalado pela bandeja. Objetivo: modo de serviço
opt-in para serve, usando o shutdown HTTP/console já existente.
Escopo: Cargo.toml/lock, main.rs, web.rs, windows_service.rs, docs/licenças.
Core Git pinado e comandos locais preservados; nenhum path dependency.

Contrato P239: --windows-service NOME LOG_DIR --config CAMINHO_ABSOLUTO;
TOML version=1, bind, monitor e token_file (relativo ao arquivo), limitado a
64 KiB e sem campos desconhecidos; config.rs entra no escopo. Eventos SCM, erros seguros e
parada cooperativa. Token/auth continuam falhando fechado onde ACL Windows não
está implementada; este adaptador não equivale a homologar painel Windows.
Sem abertura de porta remota, migração, alteração de versão, instalação no host,
commit/push/tag/release. Segredos não entram em código, logs ou argumentos.

Aceite: fmt/test/clippy/build/inventário e regressão CLI; teste de recusa
prévia ao Running por token incompatível; serviço instalado requer VM/elevacão.
Registro real em prompts/registros/A26-windows-service.md.

## Revisão v1.2 antes da correção do bloqueio de dependência

`cargo check --offline --all-targets` não encontra windows-service no cache;
a resolução online também está bloqueada por DNS no ambiente. Manter a nova
dependência no manifest principal impediria os gates --locked anteriores.
Isolar o build de serviço em packaging/windows-service/Cargo.toml, com fonte
compartilhada e feature windows-service-runtime apenas no manifest experimental.
O manifest principal e Cargo.lock mantêm as dependências anteriores. O build
experimental permanece BLOCKED até obter a dependência e gerar lock real.
Não fabricar checksum, copiar artefato antigo como novo nem declarar build PASS.

Revisão v1.3: smoke de config foi bloqueado ao escrever no diretório temporário
redirecionado do sandbox. Repetir com fixture sem segredo em target/a26-smoke,
sob a raiz autorizada; não mudar permissões nem repetir checks já aprovados.

## Revisão v1.4 — comprovação integrada P239 (2026-10-21)

Pedido: resolver a pendência dos adaptadores/bandeja do registro P239 do broker.
Entrada: 2b6f245, alterações A26 preservadas. Gerar lock experimental real;
compilar e testar o adaptador; corrigir somente erros dessa integração com
revisão prévia do prompt. Arquivos: packaging/windows-service/Cargo.lock,
src/windows_service.rs se necessário, docs/windows-service.md e registro A26.
Aceite: fmt, build --locked, testes de configuração/estado, marker real,
negativo fora do SCM e Clippy; gates globais se houver mudança funcional.
Evidências com saídas e hashes agregadas pelo P239. Preservar versão, core Git
pin e guarda de token Unix. Não alterar serviços/permissões do host nem
inferir controle SCM ou aceitação nativa do painel pelo build.

### v1.5 — lint do adaptador compartilhado por cópia

Inspeção confirmou o mesmo if let aninhado cujo Clippy falhou no cliente.
Aplicar let-chain no callback STOP, sem mudança semântica nem IO no callback;
repetir gates do manifest experimental e preservar resultados iniciais.

O primeiro gate também identificou needless_return no bloco Windows de
web::shutdown. Como os blocos cfg são mutuamente exclusivos, remover
o return final redundante preserva o encerramento. Incluir src/web.rs nessa
correção de lint, sem mudar handlers HTTP, autenticação ou política de token.
