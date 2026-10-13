# A04 — planejamento integrado dos três sistemas

13 de outubro de 2026. Prompt A04 v1.3, preparado antes das alterações e
revisado antes das correções. Pedido: conservar broker, client e admin como
três sistemas e planejar administração inspirada no Mosquitto.

Documento em `docs/planejamento-tres-sistemas.md`: responsabilidades, situação
atual versus proposta, comparação com Dynamic Security/mosquitto_ctrl,
separação dados/controle, marcos U01–U11 e critérios de aceite. Fontes oficiais
consultadas; baseline executável 2.0.22 separado da inicialização descrita para
2.1. Nenhuma compatibilidade de API ou licença MIT de terceiros foi presumida.

As skills doc-coauthoring e write-page orientaram a estrutura e a revisão de
clareza. Um leitor independente, sem histórico da conversa, respondeu oito
perguntas. A primeira leitura apontou ambiguidades sobre ordem das fases,
gates offline/online, auditoria mínima, credenciais do navegador, versão da
referência e IDs globais. A releitura confirmou o corpo e pediu repetir duas
distinções na conclusão; essas correções foram feitas sob A04 v1.2.
O alinhamento v1.3 tornou explícitas as dependências na matriz global:
contrato de revogação precede o ADR, execução online depende do canal aceito,
e auditoria básica já integra a primeira mutação desse canal.

Verificação documental: links relativos devem resolver para os três repositórios,
referências oficiais estão junto às afirmações, e diff sem erros de whitespace.
O leitor avaliou clareza e consistência, não comprovou comportamento de código.
Não executar testes Rust somente por este planejamento.

Limites: administração online continua proposta. Nenhum novo canal, CRUD de
bundle, revogação ou auditoria implementado por esta entrega. Admin conserva
runtime 0.1.0 e core pin 3606ede844210997fa733408855099535145ac48. A fase 3
continua em validação; este documento não a encerra. Somente commit/push
documental, sem nova tag, pacote, release ou alteração das imagens existentes.
