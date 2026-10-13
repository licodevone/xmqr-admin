# Planejamento integrado do XMQR broker client e admin

O ecossistema tem três programas independentes. O broker aceita conexões e
decide o que pode ser publicado e recebido. O client usa MQTT para enviar e
receber mensagens. O admin deve administrar identidades, permissões e operação
do broker, com uma interface adequada ao responsável pelo sistema.

O próximo objetivo do admin é passar do painel de leitura para uma gestão
segura de usuários, grupos, papéis e ACL. Primeiro precisamos fechar a fase 3
de robustez e aprovar o contrato do canal administrativo. Este planejamento
não transforma essas funções futuras em funcionalidades disponíveis.

## Situação dos três sistemas

Atualização de 13 de outubro de 2026. Broker 0.11.5 e client 0.8.1 são as
releases anteriores; os candidatos 0.11.6 e 0.9.0 estão nos gates da fase 3.
O admin independente 0.1.0 já está publicado e conserva o pin de core por
commit. Planejamento documental não muda sua versão nem esse pin.

| Sistema | O que já existe | Responsabilidade nas próximas entregas |
| --- | --- | --- |
| XMQR broker | MQTT 3.1.1, TLS/mTLS, ACL, bundle dinâmico com grupos e papéis, reload SIGHUP, estado durável e monitor local | Validar e aplicar alterações, autorizar operações e preservar estado e QoS |
| XMQR client | CLI pub/sub com QoS, certificados, arquivos e JSONL; candidato com prazo total e reconexão limitada | Continuar cliente de dados e comprovar permissões, revogação e recuperação |
| XMQR admin | CLI de usuários legados, hash/fingerprint e backup offline; painel local autenticado de agregados | Tornar a administração acessível por CLI e painel, com permissões e auditoria próprias |

O `mqtt-admin` legado permanece no repositório do broker. O novo programa
`xmqr-admin` não o apaga. A CLI atual de usuários manipula arquivos dos modos
de laboratório; ela não gerencia o bundle do modo seguro. O painel atual não
lista pessoas/dispositivos nem modifica sessões, ACL ou configurações.

ESP32 é um consumidor MQTT: suas credenciais e permissões poderão ser
administradas, mas firmware, lógica de automação e atualização remota da placa
não entram automaticamente no broker ou no admin.

## Referência administrativa do Mosquitto

O Dynamic Security, disponível desde Mosquitto 2.0, oferece identidades,
grupos, papéis e regras de acesso alteráveis durante a execução. Usa uma API
em tópico reservado; `mosquitto_ctrl dynsec` é uma interface de administração.
Desabilitar ou excluir uma identidade desconecta seus dispositivos e impede
novo acesso. A documentação atual também distingue recursos de inicialização
da série 2.1. Nossa referência executável permanece 2.0.22.
[Dynamic Security](https://mosquitto.org/documentation/dynamic-security/),
[mosquitto_ctrl](https://mosquitto.org/man/mosquitto_ctrl-1.html).

| Referência Mosquitto | Evolução proposta para XMQR | Situação no admin |
| --- | --- | --- |
| Identidades, grupos e papéis | Gerenciar o bundle seguro por CLI e painel | Planejado |
| ACL por operação e prioridade | Política explícita, simulador e exemplos de conflitos | Planejado; semântica não será copiada implicitamente |
| Habilitar, desabilitar e excluir | Revogação coordenada com conexões e estado QoS | Planejado |
| Administração autenticada online | Canal próprio com versão, escopos e respostas verificáveis | Contrato a aprovar |
| Configuração do controlador e TLS | Perfis de conexão sem senha em argumentos | Planejado |

Essas linhas se apoiam na
[documentação do plugin](https://mosquitto.org/documentation/dynamic-security/).
A interface gráfica será uma entrega própria do XMQR. Ferramentas web
externas citadas pelo Mosquitto não fazem parte do seu broker; sua existência
não implica uma GUI administrativa embutida ou licença MIT para reutilização.

Não prometemos compatibilidade com JSON, nomes de comandos ou arquivos do
Dynamic Security. O formato de segurança e as regras atuais do XMQR precisam
de migração explícita para qualquer extensão. Terceiros conservam suas licenças;
o código próprio dos três projetos continua MIT.

## Separação entre dados e administração

Proposta inicial: manter o monitor atual como leitura e criar, depois de um
ADR aprovado, um canal local privado entre admin e broker. O navegador fala
com o backend do admin; nunca recebe a chave de administração do broker.

```text
Navegador do operador
        | autenticação e escopo administrativo
        v
XMQR admin  ------ leitura ------> monitor local do broker
        |
        | canal privado autenticado proposto
        v
XMQR broker ---> fila limitada ---> ator ---> escritor durável
        ^
        | MQTT de aplicação com TLS/mTLS e ACL
XMQR client ou ESP32
```

O broker continua sendo o único escritor do estado MQTT. O admin solicita uma
operação e recebe seu resultado; não abre o WAL ativo, não altera mapas do
roteador e não executa comandos do shell recebidos do navegador. O client
comum não adquire autoridade administrativa por conhecer um tópico reservado.

A primeira opção de transporte a avaliar é IPC por socket Unix privado no
Linux, com permissões de diretório/socket e identidade do processo verificadas.
HTTPS/mTLS ou MQTT/TLS reservado são alternativas para uma expansão remota,
com autenticação, SAN, limites e autorização por operação. A decisão precisa
comparar isolamento, recuperação e portabilidade Windows; ainda não escolhe
uma API implementada nem abre listeners novos.

Se a alternativa MQTT for aprovada, o comando administrativo terá namespace
próprio e será interceptado pelo broker. Não deverá virar publicação comum,
retained, Will ou fila offline. Certificados de um dispositivo de aula não
concederão direitos de administrador. O contrato respeitará o limite atual de
payload; comandos grandes exigirão outra estratégia aprovada, não uma quota
maior inserida silenciosamente.

## Marcos de execução

Os identificadores U abaixo pertencem a esta trilha. A02, A03, A04 e A09
citados nas dependências pertencem à matriz global do broker. Os prompts
locais A01 a A04 do repositório admin têm outro significado e não fecham
automaticamente aquelas linhas globais.

| Marco | Entrega | Componentes envolvidos | Dependência e aceite |
| --- | --- | --- | --- |
| U01 | CLI e painel local de leitura 0.1.0 | Admin e monitor do broker | Já aceito na release existente; não é controle online |
| U02 | Editor CLI do bundle seguro: listar, validar, criar, alterar e excluir usuários, grupos e papéis | Admin e validador do broker | Depois da fase 3; testes de referências, arquivo privado, lock e atualização atômica; contribui para A02 |
| U03 | Simulador de ACL e comparação antes/depois | Admin, política do broker e client de teste | U02; explicar qual regra permitiu/negou sem ler payloads; resolver conflitos explicitamente |
| U04 | ADR do canal administrativo e contrato versionado | Broker, admin e testes independentes | U02/U03; decisão global A04, sem chamar o desenho de canal funcional |
| U05 | API privada, CLI e operações autenticadas online | Broker e admin; client prova ausência de privilégios | U04; aceite global A09, filas/prazos/RBAC/idempotência e falhas reais |
| U06 | Habilitar, revogar, alterar senha e aplicar política | Broker e admin, client e ESP32 de laboratório | U05; aceite A03 com efeitos definidos sobre conexões, sessões e QoS2 durável |
| U07 | Painel operacional de usuários, grupos, papéis e ACL | Admin e API do broker | U05/U06; escopos por operador, preview/diff e confirmação; não depender de leitura direta de arquivos privados |
| U08 | Auditoria e diagnóstico administrativo | Broker registra resultado, admin apresenta/exporta | U05; A05/A08, limites de retenção e falha de gravação testados |
| U09 | Sessões e retained administráveis | Broker, admin e client de prova | U05/U08; A07, paginação e metadados; ações destrutivas separadas e auditadas |
| U10 | Perfis, certificados e operação remota | Três sistemas | Coordenar com listeners e rotação da fase 4; validar CA/SAN/revogação sem emitir certificados automaticamente |
| U11 | Pacotes e matriz de compatibilidade por marco | Três sistemas | Gates do marco, commits exatos, versões independentes, tags imutáveis e releases proporcionais |

U02 pode começar com validação e edição offline. Aplicar uma política em
execução é outra entrega: exige confirmação verificável da revisão ativa,
não apenas salvar o arquivo ou enviar SIGHUP. Nenhuma dessas etapas começa
automaticamente durante o fechamento da fase 3.

## Regras de acesso e revogação

Uma identidade representa um usuário ou dispositivo autenticado. Um grupo
reúne identidades. Um papel reúne permissões. ACL é a regra que autoriza ou
nega uma operação sobre tópico ou filtro. Para o curso, o exemplo será um
ESP32 que publica temperatura, um client que consulta essa informação e um
operador que gerencia o cadastro. O operador não precisa receber acesso aos
payloads para realizar esse trabalho.

A evolução não pode trocar a política atual por prioridades ao estilo
Mosquitto sem desenho e migração. O simulador mostrará operação, regra,
revisão e resultado; casos com wildcard, namespace `$`, regra mais específica,
conflito e identidade sem papel terão resultados definidos por testes.
Permissões de publicar, assinar e receber precisam de semântica explícita,
incluindo mudanças depois de uma assinatura persistente.

Revogar deverá impedir novas conexões e definir o encerramento das atuais.
O planejamento preserva a obrigação QoS2 já aceita de forma durável: revogação
não pode apagar silenciosamente esse compromisso. Efeitos sobre Will,
offline, inflight e retained serão escritos no prompt e verificados em
reconexão/restart antes do aceite. Excluir cadastro não significa apagar dados
MQTT ou certificados de uma CA.

## Contrato e proteção do operador

Cada operação futura terá identificador, versão de contrato, revisão esperada
da política, escopo, prazo e resultado. Dois operadores que alteram a mesma
revisão não devem se sobrescrever: um conflito devolve diagnóstico e exige
novo preview. Uma resposta perdida não autoriza repetir uma mutação incerta;
idempotência e consulta do resultado devem tornar esse caso recuperável.

O primeiro desenho de papéis de operador separará observação, gestão de
segurança e gestão de estado. O token atual do painel é um acesso local de
leitura, não um sistema multiusuário de operadores nem senha MQTT. A passagem
para escrita exige autenticação/escopos próprios, proteção contra CSRF e
negação de origens indevidas, além das proteções existentes de Host, CSP e
limites de conexões. Não guardar segredos no navegador, em URLs ou nos logs.

Senhas serão solicitadas sem eco ou lidas de arquivo privado; certificados
e chaves não serão emitidos ou exportados pelo painel sem uma nova entrega
específica. Campos de política inválidos, identidades duplicadas, referências
pendentes, symlinks, arquivos com permissões abertas e operações sem escopo
devem falhar sem atualização parcial.

A auditoria registrará operador, operação, revisão, horário e resultado, sem
senha, token, chave, payload ou hash de senha. IDs sensíveis exigem acesso
restrito. O desenho deverá definir rotação, retenção, integridade e o que
acontece se a auditoria falhar; não prometer resistência a adulteração antes
desse mecanismo existir.

## Sessões retained e recuperação

Listagens futuras terão paginação, limites e filtros. Por padrão, mostrarão
metadados e contagens, não conteúdos das mensagens. Desconectar sessão,
excluir retained e excluir estado persistente serão operações diferentes,
com preview, escopo específico, confirmação e registro de resultado.

Backup e restore continuam privados e offline na primeira evolução. Não
restaurar por cima do diretório ativo. Um snapshot online consistente requer
comando coordenado pelo ator do broker e novo aceite. Pacote de diagnóstico
não deverá incluir banco MQTT, credenciais ou payloads automaticamente.

## Critérios de aceite dos três sistemas

Cada marco deverá ter um prompt anterior à implementação, negativos e
evidências. Uma entrega de segurança só será aceita quando o broker aplicar
a mudança, o admin confirmar a revisão e o client autorizado/não autorizado
comprovar o efeito por MQTT independente da interface.

Os gates incluem concorrência entre operadores, perda de resposta/retry,
fila cheia, prazo expirado, disco sem espaço, revisão incompatível, restart,
ACL negada e TLS/CA/SAN incorretos. Regressões de QoS, sessões, retained e
Will permanecem obrigatórias. O firmware físico do ESP32 é um gate separado
quando a mudança afetar a placa; um teste de desktop não o substitui.

Versões e releases são independentes. Alterar só o planejamento gera commit
documental, não nova tag ou pacote 0.1.1 do admin. Alterar código gera o marco
aceito correspondente, com `.deb` Ubuntu 26 amd64 e imagem Docker numerada,
hashes, provenance e compatibilidade registrada. O core Git só será repinado
em uma entrega testada; não atualizar por acompanhar um novo número do broker.

## Ordem sugerida depois da fase 3

Fechar robustez primeiro; depois executar a fase 4 de listeners/perfis/rotação
com os três componentes explicitados no backlog. A trilha administrativa da
fase 5 começa com U02/U03 e aprovação U04, entrega o canal U05 e então habilita
revogação, painel de escrita, auditoria e gestão de estado. Esse encadeamento
mantém uma entrega utilizável por vez e não mistura MQTT5, bridge ou serviço
Windows com a primeira API administrativa.

Referências do projeto: [matriz global](../../xmqr/docs/plano-paridade-mosquitto.md),
[evolução por fases](../../xmqr/docs/evolution-plan.md),
[segurança do admin](../SECURITY.md) e [prompt deste planejamento](../prompts/admin/A04-planejamento-tres-sistemas.md).
