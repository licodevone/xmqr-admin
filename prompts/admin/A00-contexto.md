# A00 v1.1 — XMQR Admin independente

Preparado em2026-10-13, antes de qualquer funcionalidade. Pedido explícito:
criar software xmqr-admin, enviar ao GitHub e manter prompts -> código ->
validação -> commit/push -> tag de código aceito -> release quando pertinente.
Autorizações anteriores incluem .deb e Docker Hub lepinheiro100. Repositórios
existentes públicos licodevone/xmqr (HEADc678f8d, runtime0.11.5) e xmqr-client
(HEADd9d97d5, runtime0.8.1) preservados, inclusive dois artefatosclient0.7.1.
Não existe pasta/repositório xmqr-admin acessível; contaGitHub licodevone confirmada.

Criar terceiro Gitrepo, pacote xmqr-admin0.1.0, binário xmqr-admin, Rust2024/MSRV1.88,
MIT própria. Não remover o mqtt-admin legado nem mover tags/debs/imagens existentes.
Não criar workspace. Core compatível reutilizado como dependênciaGit imutável
licodevone/xmqr@3606ede844210997fa733408855099535145ac48 (v0.11.5), não path../xmqr
nem versão flutuante. Extrair main/users/state do CLI legado com provenance.
Versionamento independente do admin novo; MQTT e armazenamento do broker intactos.

Marcos desta solicitação: A01 CLI independente, A02 painel local de leitura,
A03 distribuição inicial. O canal de mutação ao vivo A04/A09 e gestão de
grupos/papéis/ACL/sessões permanecem no backlog, dependentes de aceites do plano.
Antecipar interface de leitura não conclui fase5 nem passa por cima de B07/B08.

Todas as mudanças posteriores exigem prompt específico antes do código,
registro real e negativos. Não publicar credenciais, token, payloads, estado,
certificados ou chaves privados. Sem instalação no host, serviços reais,
firewall, firmwareESP32 ou troca de permissões fora de recursos novos de teste.

Retomada confirmada: fontes agora em
D:/projects/my-project/acmebr-projects/acmebr-broker-client/xmqr-admin.
Pedido humano específico autorizou criar repositório e agentes paralelos.
GitHub licodevone/xmqr-admin criado público inicialmente vazio em2026-10-13;
originhttps://github.com/licodevone/xmqr-admin.git. Criação não prova código
publicado: commit/push/tag/release só registrados depois dos gates reais.
AgrupadorAGENTS pode receber referência ao terceiro projeto após aceite;
não é Gitroot e sua edição local não será anunciada como push de repositório.
