# A28 — Console MQTT configurável no serviço Windows

Versao1.2, 22/10/2026. Base86f0dd917783da26ac1654526dd0c583d161733c;
árvoreAdmin limpa na entrada. BrokerP252 coordena Manager e atualização.

Entrada: Board::from_env retorna disabled semXMQR_ADMIN_MQTT_SUB_PROFILE;
TOML de serviçoAdmin não configura console. Client executávelP248 disponível,
credenciaisP247 privadas e Brokerlocal8883; contasweb/token existentes preservar.

Objetivo: opcional [mqtt_console] no TOML com client_bin,sub_profile,pub_profile,
label; caminhos resolvidos contra config, bounded/strictschema e defaults
compatíveis. Fonte explícita não alteraenvglobal; manter modoenvlegado quando
campoausente. Board valida perfis privados/executável local e só inicia criança
nas açõesautenticadas. Preservar limitespayload/tempo/filas/lease/CSRF/RBAC.

Escopo: src/config.rs,mqtt_console.rs,web.rs e fixtures relacionados; docsA28,
registro; build.rs/assetAdmin no manifestWindowsservice para incorporar ícone
original durante link semPEpatch/RUSTFLAGSglobais/dependências novas. Testes
com segredoefêmero e peerlocal; não ler token/senha/chave real nas ferramentas.
Não mudar protocolo/client/broker/versão ou iniciaralterar serviço do host.

Aceite: config/opcionais/negativos e console backwardcompatibility; fmt,
testWindowsservice,clippyalltargets-Dwarnings,lockedofflinebuild; capabilities
e version; íconeAdminoriginal readonly. Prova isolamento PUB/SUB mTLS se ambiente
permitir, senão declarar gatebloqueado. Instalação/contas existentes não tocadas
pelo agente; scriptoperador combackup/hashes da fatiaP252.
Antes de corrigir falhas, revisar prompt. Semcommit/push/tag/release.

## Revisao1.1 - fixture Windows

Primeiro Clippy all-targets compilou dependencias do perfil dev, mas falhou
E0308 no novo fixture de ACL: SetSecurityInfo requer handle mutavel. Corrigir
somente teste para handle mutavel com READ_CONTROL/WRITE_DAC, necessario para
aplicar ACL sintetica; preservar validadores e dados reais. Repetir gates.

## Revisao1.2 - posicao do import de teste

Segunda rodada Clippy encontrou items_after_statements no import Windows do
fixture. Mover use para inicio do teste e repetir Clippy estrito, sem allow.
