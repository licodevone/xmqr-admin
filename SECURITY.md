# Segurança

Versão inicial experimental. Relate problemas privadamente pelo formulário
de segurança do repositório; se indisponível, contate o mantenedor sem publicar
credenciais/exploração. Sem declaração de prontidão para produção.

Backend paraLinux/WSL, semroot. Bind e destino do monitor somenteloopbackIP:
porta; URLs vêm da configuração, não do browser. Sem redirects/proxy doHTTP
client. Respostas do monitor até64KiB, total3s, requisiçõesAPI até16 simultâneas.
Até32 conexões TCP aceitas; vida absoluta10s por conexão incluindo headers,
keepalive e peer não leitor. Excesso fecha socket antes da tarefaHTTP. Sem
streaming: navegador deve reabrir conexão nas consultas posteriores. Shutdown5s.
SóGET/HEAD sem corpo/query. AutorizaçãoBearer64hex aleatório, comparação CT,
Host/Origin exatos, sem CORS, no-store/CSP/frame-ancestorsnone/nosniff.

Token separado da autenticaçãoMQTT. Arquivo regular não-symlink0600; leitura
O_NOFOLLOW/O_NONBLOCK e validação do mesmo descritor, pai privado. Ancestros
do diretório devem ser confiáveis; não é sandbox contra o próprio usuário/root.
Criação
em diretório privado, somente destino novo. Sem token em storage/URL/logs.
Config/token carregados ao iniciar, rotação requer restart. Uma conta com
acesso ao mesmo usuário doSO pode ler memória/token; loopback não é isolamento
contra processos locais privilegiados. Sem TLS/identidades web por usuário/RBAC
no painel de leitura inicial; acesso remoto permanece fora do escopo.

Não editarWAL no painel nem apresentar erro/ausência como contagem zero.
Não retornar corpo/erro bruto do monitor. Apenas nomes fixos de agregados,
sem usernames/ClientIDs/tópicos/payloads. Readiness não garante disco livre,
energia, rede do dispositivo ou processamento de negócio. Métricas não são
snapshot transacional e contadores zeram no restart.

CLI mantémCRUDlab/Unix com lock/validação/rename/sync. Backupoffline com lock
exclusivo; restore somente destino novo. Não criar permissões MQTT implícitas,
mutar bundle/destino ativo ou iniciar serviços. CoreGit pinado e formatos
preservados. Não expor como produção por ter testes. Novos canais de escrita
precisam contrato/RBAC/auditoria e negativos antes da implementação.
