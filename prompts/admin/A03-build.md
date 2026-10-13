# A03-build v1.5 — construção e gates de distribuição

Preparado e apresentado em 2026-10-13 antes dos arquivos. Entrada: xmqr-admin
0.1.0 independente, A00/A03 v1; core Git imutável
3606ede844210997fa733408855099535145ac48. Não há commit inicial nem publicação.
O coordenador só publicará após A01/A02 e estes gates aceitos.

Escopo exclusivo: Dockerfile/.dockerignore, packaging/, scripts/build_distribution.py,
scripts/collect_notices.py e docs/distribution.md. Não alterar broker/client,
integração CLI/painel, versões, credenciais reais nem serviços do host.

Construir tar nativo Linuxamd64 e deb Ubuntu26.04 amd64 standalone que instala
somente /usr/bin/xmqr-admin e documentação. Sem usuários de sistema, conffiles,
units, maintainer scripts, enable/start/restart ou colisão mqtt-admin. Extrair
dependências ABI do ELF com dpkg-shlibdeps; rejeitar plataforma/revisão inválida.
Revisão Debian 2 somente para gate local upgrade/rollback, não release nova.

Coletar de cargo metadata --locked textos LICENSE/COPYING/NOTICE e license_file
originais, byte a byte, dentro de cada raiz de dependência, recursivamente para
fontes vendorizados (incluindo AWS-LC); rejeitar symlinks, preservar hashes e
caminhos relativos sem caminhos privados do host. Inventário não é relicenciamento
MIT das dependências nem parecer jurídico. Incluir inventário/provenance/licenças
nos três formatos. Falta de notices resulta em gate falho, não promessa automática.

Docker: build Rust1.99 Bookworm e runtime Bookworm, ambos linux/amd64; não usar
binário Ubuntu26 no runtime Bookworm, pois ABI não é presumida. UID/GID10001,
SIGTERM, entrypoint xmqr-admin, labels versão/commit, contexto allowlist com
exclusões finais de segredos. Namespace compartilhado só em laboratório;
loopback do painel preservado. Nome novo lepinheiro100/xmqr-admin-ui:0.1.0,
sem latest e sem publicação por este subagente.

Aceite proporcional: sintaxe/scripts, notices hash/source, negativos no coletor,
ELF/versionamento, tar/sha, metadados deb/conteúdo sem serviços, instalação,
upgrade revisão2, rollback1, remove em container descartável Ubuntu26 preservando
token/estado simulados fora do pacote. Docker versão/user/sinal/labels/notices,
painel autenticado e negativos via recursos próprios; execução exata limpa fica
com coordenador. Artefatos em dist/ ignorado. Nunca instalar no host. Não afirmar
PASS de gate não executado; registrar bloqueios/falhas antes de corrigi-los.

Revisão v1.1 após FAIL real do coletor: r-efi6.0.0 não distribui arquivos
LICENSE/COPYING separados; README aponta para AUTHORS, que contém textos e
copyright originais. Incluir AUTHORS/CONTRIBUTORS na seleção preservada,
sem substituir por licença inventada ou enfraquecer ausência de texto. Novo
negativo/fixture comprova seleção AUTHORS. Draft1 parcial não é artefato aceito;
executar em output novo. Nenhum segredo envolvido.

Revisão v1.2 após FAIL real seguinte: valuable0.1.1 omite LICENSE no crate
publicado. Cargo.toml aponta tokio-rs/valuable, .cargo_vcs_info.json fixa commit
9efc29b6e58cef28f6566a47aa7e142a55fead77. Obtido texto original desse commit
por URL raw GitHub; 1065bytes/SHA256
ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44.
Salvar complemento byte a byte em packaging/notices/valuable-0.1.1/LICENSE,
coletor offline verifica nome/versão/commit/sha/declaraçãoMIT e registra URL.
Sem fallback automático de rede ou licença genérica; outros casos sem texto
continuam falhos. AllowlistDocker inclui só esse complemento; draft2 descartado
como gate. Recompilação funcional paralela em andamento: drafts não validam
o novo listener do coordenador, gates finais exigem binário do commit exato.

Revisão v1.3 após FAIL real superset: winapi-i686-pc-windows-gnu0.4.0 exclui
licenças na lista include do archiveCargo, informa MIT/Apache-2.0 e mesmo
repositório retep998/winapi-rs, orienta depender de winapi. Incluir também o
análogo x86_64 no fallback estrito. Copiar os textos originais LICENSE-MIT/APACHE
do winapi0.3.9 do mesmo Cargo.lock, com donor/package/repository/commit
796a8e6c2971dc2ff1bcff166e6671284f9b5b6b no manifesto. Não inventar que
esse commit é a origem do pacote0.4.0, nem afirmar esses componentesWindows
integram o ELFLinux. Outras dependências sem texto continuam falhando.

Revisão v1.4 após FAIL real lifecycleDraft: instalação e versão/token-init
executaram, script saiu1 antes de upgrade. Diagnosticar somente mode/existence/
configuração dpkg do container, nunca imprimir token ou dados privados. Se
Ubuntu26 Docker excluir docs na configuração minimalista, aplicar path-include
específico ao pacote apenas nas invocações dpkg do container descartável e
registrar condição; não enfraquecer exigência de notices nem mudar o host.

Revisão v1.5 por revisão do coordenador, antes da implementação: também copiar
LICENSE próprio verbatim para /usr/share/doc/xmqr-admin/copyright, convenção
Debian preservada pela imagem Ubuntu minimalista. Não substituir avisos de
dependências por MIT própria; arquivos completos continuam em third-party-notices
e gate path-include permanece explícito. Lifecycle draft corrigido passou
install/upgrade2/rollback1/remove e preservação fixtures/legacy, mas gate do
commit exato e copyright novo continuam pendentes até reconstrução.
