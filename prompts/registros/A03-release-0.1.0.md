# A03 — primeira publicação independente concluída

Data2026-10-13. Pedido humano autorizou repositórioGitHub, prompts antes das
mudanças, testes/commit/push/tag/release e .deb/DockerHub; paralelização também
explicitamente autorizada. A03-distribuicaov1.5/A03-buildv1.5/A02v1.4.

Repositório: https://github.com/licodevone/xmqr-admin .
Tag anotadaimutávelv0.1.0 aponta7085d7b1996cb54cb70e588ca73b104eb948a324.
Prerelease: https://github.com/licodevone/xmqr-admin/releases/tag/v0.1.0 .
22anexos verificados contraGitHubAPI por nome/tamanho/SHA256. Debbaixado
novamente confirmou mesmoSHA256, não apenas existência remota.

## Gates reais do código e dos artefatos

GitHubCI do commit7085: PASS
https://github.com/licodevone/xmqr-admin/actions/runs/38452940347 .
Clonepúblico440 criado limpo, atualizadoff-only7085 e novamentegates:
fmt21testesRust(18unit3integração),Clippy-Dwarnings,buildrelease,inventário,
9contratosdistribuição/3export e18cenáriosHTTP/MQTT PASS. Semeditarfonte e
Gitlimpo antes/depois. RuntimereleaseSHA256
b6edea14c7214ce4574f08d954a8a2e4833995f0f5b6bd054d133f412d0f473e,
idêntico ao browser8PASS e revisãoTCP4casos (32/64recusadas,10s,recuperação,
shutdown5.049s); alterações7085 fortaleceram fixtures, não runtime.

Native7085Ubuntu26amd64:tar/debrev1/rev2 construídos semdirty/draft,180
dependências/326avisos eMIT/copyrightoriginais conferidos byte a byte/hash.
Conteúdos/provenance/ABI/ELF/checksums e ausênciaunits/maintainerscriptsPASS.
Lifecycle emUbuntu26Dockerpróprio:install→upgradeDebianrev2→rollback1→remove,
token0600/estado/hashsimulado e mqtt-admin legado preservados. Rev2 não é
release pública. ImagemUbuntuminimal exclui docs: path-include específico no
gate, registradoexplicitamente; não inferir notices disponíveis no default
minimal. CopyrightMIT padrãoDebian retido. Seminstalação ou serviçohost.

Dockerfinal7085:15cenários reaisPASS, namespace loopbackshared com broker
0.11.5/client0.8.1 e volumes temporários privados, semportas publicadas.
UID/GID10001, SIGTERM, labelsversão/commit, tokenprivado, agregados reais,
negativosHTTP/backup/lock/destinoexistente, unavailable nãozero e shutdown.
Runtime separadoBookworm; não copia ELFUbuntu26.326notices+manifest presentes.

Fonte staged Gitleaksredact/diffcheckPASS antes primeirocommit,semsegredos
detectados; não é auditoria integral. Falhas anteriores/harness/fixtures/
licençascontainer e revisãoantesfix estão nos prompts/registros. Checkpoints
440 e rascunhos preservados, não anexados como entrega7085. Windowsnegative
symlinkBLOCKED privilégio1314; aceiteUnixPASS não certificaWindowsnativo.

## Artefatos e publicação

- Deb: xmqr-admin_0.1.0-1_amd64.deb,1157374bytes,
  SHA2564a1f6ef0735814736aea55fe4b2b4ac82867c526e4e9d46e423469074ae75f8a.
- Tar: xmqr-admin-0.1.0-ubuntu26-amd64.tar.gz,1500457bytes,
  SHA2567c35f50b7a117d3213ae5993ed3f011fb7c0771beafac08b60a9b81f0a371c9f.
- ExportDocker: xmqr-admin-ui-0.1.0-docker-linux-amd64.tar.gz,32814327bytes,
  SHA256cc146bf8afba69e624cecbe1f455c9850fd3093e46931999645ec10523e1fde1.
- Hub: lepinheiro100/xmqr-admin-ui:0.1.0,
  https://hub.docker.com/r/lepinheiro100/xmqr-admin-ui .
  Digestremoto sha256:b6ba41cbc60f575fea89e5804809425d172e67d5ca98c2adf767c57689e33f1c.
  Imagetools conferiuLinuxamd64; pullpordigest e --version PASSxmqr-admin0.1.0.

Manifestos/SHA/evidências fazem parte dos22anexos. RelatórioexatoSHA256
c13dd7eb605cdec8942e1159283ed48668efc842a86b9e156b6ad94d98d621f7;
pacoteaceiteSHA2567090e9562f7df52c3328c2e9c10449a6a19bfb0463ecc3a5a98c385ba3073143.
README e notasWeb por linksGitHub absolutos após publicação; pacote contém
seleção de documentação, referências relativas assumem checkout. Documentação
integral está no repositório; não anunciar distribuiçãooffline do curso.

ApósHubpush, digests antigosbroker/admin0.11.5/client0.8.1 conferidos idênticos
às baselines P87/C42. Não usouforce, moveu tag antiga ou latest. Fontesclient
intactas; seus dois artefatos locais0.7.1 preservados. BrokerP88 será apenas
referência documental do terceiro projeto, sembump/tag extra. AgrupadorAGENTS
foraGitroot receberá referência local, não será apresentado como push.

## Limites

Painel local somenteleitura e CLI usuárioslab/backupoffline; sem canal online
A09/grupos/ACL/sessões/RBAC, certprovisioning, remotoHTTPS, firmware/ESP32 ou
produção. CoreGit3606ede imutável, formatos/quotasbroker/client eMITpreservados.
Fases3/5permanecem abertas. Recursosde testeprocessos/token/volumecontainer
limpos; nenhuma credencial ou estado real foi lido/enviado. Próxima evolução
exige prompt próprio; este marco não executou todo o backlog.
