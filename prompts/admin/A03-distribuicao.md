# A03 v1.5 — primeiro marco público independente

Entrada: A01/A02 precisam PASS. Publicar licodevone/xmqr-admin público, alinhado
aos repositórios existentes da conta confirmada, sob pedido direto do mantenedor.
Prompts e registros fazem parte da fonte. Criar commit aceito, pushmain, validar
commit exato limpo, tag anotadav0.1.0 e prerelease experimental se gates aceitos.
Não usarforce, sobrescrever tags, mexer em outros repos ou declarar produção.

Release inclui tar.gzLinuxamd64 com binário/assets embarcados, SHA256, licença,
inventário/provenance e evidências sanitizadas. .debUbuntu26amd64 instala
/usr/bin/xmqr-admin, sem mudar/ativar serviços ou conflitos com mqtt-admin legado.
Ciclo instalar/upgrade de revisãoDebian/rollback/remove somente container
descartável, preservando arquivos privados simulados fora do pacote.

DockerHub: lepinheiro100/xmqr-admin-ui:0.1.0, nome distinto do admin CLI legado
lepinheiro100/xmqr-admin:0.11.5. Linux/amd64, não-root10001, SIGTERM, labels versão/
commit, contextoallowlist sem credenciais. Este primeiro runtime de painel
permanece loopback: testar em namespace compartilhado com broker/cliente;
não ampliar bind para0.0.0.0 nem sugerir painel remoto como já suportado.
Sem latest. Exportar imagem, checksums e manifesto, conferir digest remoto e
assetsGitHub apóspush/publicação. Registrar bloqueios reais se houver.

Após publicação: registrar commits/hashes/URLs e atualizar referência do broker
porP88 documental. Histórico/broker0.11.5/client0.8.1 e suas tags intactos.

Retomada2026-10-13: pasta mudou para acmebr-projects/acmebr-broker-client;
fontes preservadas. Autorização explícita para criar GitHub e agentes paralelos.
A03-build detalha empacotamento/avisos; o coordenador mantém scripts/verify_docker.py,
docker_probe.py e export_image.py. Probe HTTP em container Python temporário,
namespace loopback do broker de teste e volume novo privado; nada exposto por
portas publicadas. Incluir consulta real, negativos, parada limpa e backup/lock
offline no binário novo. Exportador exige commit limpo, labels exatas e nomes
numéricos novos, usa criação exclusiva e hashes; não publica automaticamente.
Ignorar metadados locais da ferramenta (.serena) e todo dist/target/estado.
scripts/test_export.py verifica contratos de versão/commit/runtime/plataforma
antes de exportar. GitHub pode ser criado vazio antes do aceite do código;
isso não conta como commit/push/tag/release. Publicação de fonte continua
condicionada aos gates e revisão, explicitando status do repositório vazio.

Antes do commit: GitWindows avisou conversãoCRLF nos scripts e licença
complementar. Fixar eolLF para fonte textual em .gitattributes e -text para
packaging/notices/**, preservando bytes originais/hash em clonesWindows/Linux.
Revisar diffcheck e secret-scan dos arquivos staged antespush; scanner não é
auditoria integral de credenciais. CI inclui contratosPython da distribuição.

Dockerprobe do candidato saiuFAIL com erro genérico (recursos limpos).
Antes de corrigir, emitir só o nome fixo da etapa que falhou no probe,
nunca body/headers/token/logs brutos. Expor essa mensagem fixa no harness
somente quando corresponder ao prefixo conhecido. Investigar no namespace
temporário e corrigir ferramenta ou runtime sem relaxar asserção/segurança.

Diagnóstico fixo apontou foreign host. Reprodução isolada Node24 confirmou
fetch ignora Host customizado e envia o host da URL (127.0.0.1:porta), então
o negativo não atingia o backend com entrada incorreta. Trocar a sonda por
node:http.request para controlar bytes/cabeçalhos do negativo; manter Host
esperado403, limites/deadline e ausência de token nos outputs. Não alterar
backend nem aceitar200 como sucesso desse cenário. Reexecutar Docker completo.
