# A04 v1.2 — planejamento integrado de broker client e admin

2026-10-13, antes do documento. Pedido adicional do mantenedor: lembrar dos
3sistemas e planejar administração inspirada no que Mosquitto oferece.
Manter fechamentofase3emcurso, sem executarfase4/5/canaladmin nesta fatia.

Entradaadmin ec67ce2/runtime0.1.0 publicado, coreGitpin3606ede eMITprópria;
brokerrelease0.11.5/candidato0.11.6; clientrelease0.8.1/candidato0.9.0.
AGENTS/README/ORIGIN e backlogglobalA01–A09 lidos. CLIuserslegado nãoedita
bundle seguro, web sóagregadosread-only. LocalA01/A02/A03 eesteA04não são
aceites automáticos dos IDs A02/A03/A04/A09 globais.

Usar doc-coauthoring para contexto/estrutura/revisão de leitor; write-page
para clareza de prosa salva no formatoMarkdown estabelecido do repo. Sem
perguntasbloqueantes: contexto e autonomia do mantenedor suficientes. Fontes
oficiaisMosquitto consultadas eContext7; DynSec existe2.0, documentaçãoatual
tambémdescreve2.1. Distinguirbaseline2.0.22 deextensões. Não copiar código,
UI, nomes deAPI como promessa decompatibilidade ou relicenciarterceiros.

Escopo: docs/planejamento-tres-sistemas.md, linkREADME, registroA04; master
broker/docs/plano-paridade-mosquitto.md deve ligar esta trilha. Responsabilidade
broker(singleactor/writer/aplicarpolítica), admin(operator/scopes/UI/auditoria),
client(pub/sub/provasautorização), ESP32consumidor. Backlogstatusgates,
dependênciasMúltiploslisteners/rotaçãofase4 eadminfase5, interfacelocalIPC
proposta com alternativasmQTTS/HTTPSemADRpendente. Nenhumcanalcontrolejáaceito.

PlanejarCRUDbundle/grupos/papéis/ACL, enable/revoke, efeitosQoS2durável,
dry-run/diff/expectedgeneration/idempotência/authRBAC/quotas/secrets/auditoria,
sessões/retainedpaginação/metadados/açõesconfirmadas, backupsprivadosoffline,
diagnósticoeexports. Não prometerpayloads, comandosESP32, GUIoufeatures2.1
comoMQTTnormativo. Cite fontes pertodeclaims; propostas própriasidentificadas.

Aceite: leitura cruzada, testar5–8perguntas de leitor comagentesemcontexto
quando slotdisponível, corrigirambiguidades, linksrelativos/referências/IDs
eGitdiffcheck. Apenas docs/prompt/registro=>commit/push, sem bump/tag/release
adminartificial. MIT/pin/binário/distro0.1.0intocados. Skill nãoautoriza
publicarcloudPages ou implementar backlog. Rastrearphase3comoemvalidação.

Revisão1.1 apósleitorsemcontexto eANTESdocfix: esclarecer fase4prioridade
vsU02offline; aceiteonline nãoexigirAPIparaU02; auditoriamínima jáU05,
U08sóevolução; distinguir credencialRAM/cookie da chave/segredosMQTT;
nomear2.1initautomatico e vincular fontes/releases/pin/LICENSE; todosAxx
globais distintos de prompts locais. Nenhuma implementação por essascorreções.

Revisão 1.2 antes da última correção documental: a releitura independente
confirmou o corpo, mas a conclusão ainda podia sugerir U02/U03 somente depois
da fase 4 e auditoria somente depois do canal. Repetir na conclusão a exceção
offline após fase 3 e a auditoria mínima como gate de U05. Sem código ou versão.
