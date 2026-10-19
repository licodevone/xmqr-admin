# A24 v1.0 — usuários e sessões do painel web

2026-10-19. Entrada5f98f4a/A23, admin0.11.0 fonte; pacote instalado preservado.
Usuário solicita autenticação por usuários, com agentes paralelos autorizados.
Antes do código, definir login, bootstrap e autorização separados de MQTT.

Modo opt-in XMQR_ADMIN_WEB_USERS_FILE absoluto em diretório privado0700.
Sem variável, preservar tokenBearer/A23. Com variável, usuário/senha e cookie
HttpOnly/SameSite=Strict/Path=/; HTTP é somente loopback. TokenBearer antigo
não permite contornar login nesse modo. Tokenlocal existente só autoriza criar
o primeiro administrador; setup torna-se indisponível após primeiro usuário.
Sem senha padrão, sem senha gerada no chat, sem reaproveitar senhaMQTT.

Armazenamento versionadoJSON até64KiB/64usuários, arquivos0600 reais sem
symlinks, lock exclusivo enquanto servidor ativo, escrita temporária/rename/
fsync e validação estrita. Não apagar arquivo inválido para reinicializar.
Senhas8..128bytes sem NUL/controles, Argon2id com saltOsRng, KDF e disco em
spawn_blocking, sem senha original persistida/logada. UsuárioASCII3..32 com
letras/números/_/- normalizado para minúsculas; nome visível1..80bytes.

Papéis: viewer consulta métricas; operator consulta métricas e opera MQTT;
admin também cria/lista/ativa/desativa usuários. Contas web não alteram usuários
MQTT/ACL; perfisMQTT continuam preparados pelo operador. O console é compartilhado
entre operadores autorizados nesta fatia, informação explícita na interface.
Não desativar o último admin habilitado nem permitir usuário comum elevar papel.

Sessões opacas aleatórias32bytes, armazenar digest no servidor, máximo64,
expiração absoluta8h e inatividade30min; reinício exige login novo. CSRFrandom
por sessão devolvido ao browser emJSON e exigido em X-XMQR-CSRF paraPOST
autenticado, além de Origin/Host exatos. Logout revoga e expira cookie;
desativação revoga todas as sessões daquele usuário. Cookie nunca contém
senha/tokenlocal e não entra emURL/storage. Erro de credenciais genérico,
mesmo trabalho Argon2 para usuário ausente/desabilitado. Máximo2KDFs paralelos,
tentativas limitadas globalmente e memória/filas limitadas. Não implementar
recuperação por email; reset futuro é administrativo, sem recuperar senha antiga.

ContratoHTTP (JSON <=8192, Origin obrigatório em todosPOSTs):
- GET /api/auth/config público: {mode:"token"|"users",setup_required:bool}.
- POST /api/auth/setup {bootstrap_token,username,display_name,password}:
  somente primeiro usuário; validar tokenlocal com comparação constante,
  criar admin e retornar sessão como login. Demais tentativas409.
- POST /api/auth/login {username,password}:200{username,display_name,role,
  csrf_token} +Set-Cookie,401genérico,429limite. Nunca retornar hash/senha.
- GET /api/auth/session: retorna os quatro campos acima de sessão autenticada.
- POST /api/auth/logout {}:200{ok:true}, revoga cookie/sessão.
- GET /api/auth/users (admin):listaJSON [{username,display_name,role,enabled}],
  sem hashes.
- POST /api/auth/users {username,display_name,password,role} (admin):criação.
- POST /api/auth/users/state {username,enabled} (admin):ativar/desativar.
- APIsA23 são protegidas conformepapéis; viewer não acessa console/payload,
  operator não lista/cria/desativa usuários. Postsfora da allowlist405;
  JSON/schema inválido400, sem detalhes sensíveis.

Backend novo src/web_auth.rs; integrador controla rotas/middleware e Cargo.
API pública acordada: Auth::from_env()->io::Result<Auth> (rodar emspawn_blocking
no startup), Auth::token_only()->Auth, Auth::is_users()->bool,
asyncconfig()->AuthConfig, asyncsetup(SetupRequest,&Token)->Result<LoginReply,AuthError>,
asynclogin(LoginRequest)->Result<LoginReply,AuthError>,
asyncauthorize(&HeaderMap,write:bool)->Result<Identity,AuthError>,
asyncsession(&HeaderMap)->Result<SessionInfo,AuthError>,
asynclogout(&HeaderMap)->Result<Response,AuthError>,
asyncusers()->Result<Vec<UserInfo>,AuthError>,
asynccreate_user(CreateUserRequest)->Result<Vec<UserInfo>,AuthError>,
asyncset_enabled(StateRequest)->Result<Vec<UserInfo>,AuthError>.
Identityrole público com allows_mqtt()/is_admin(); AuthErrorIntoResponse,
LoginReplyIntoResponse; SessionInfoSerialize. Handler valida papel antes de
chamar métodos de usuários; módulo deve testá-los também onde aplicável.

Agentes: autenticação apenasweb_auth.rs; interface apenasassets; validação
apenas scripts/verify_web_auth.py e docs/web-auth.md. Sem commits dos agentes.
Integrador revê segredos/autorizações/races e preserva A23/fase8, cria registro,
CI e commits/push após gates. Promovertempfile a dependência runtime (mesmo
lock/inventário) se necessário. Sem alterarcore pin ou senhaMQTT.

Aceite: KDF/hash/validação/limites/lock/corrupção/session/CSRF/revogação,
bootstrap único, anonimato/credenciais erradas, perfis viewer/operator/admin
com negativos de cada API, logout/restart e conteúdo sem XSS. Browserreal
setup/login/roles/contas/logout emdesktop/mobile. RegressõesA23 e gates
fmt/test/Clippy/buildrelease/inventário, evidências sanitizadas/CIexata.
Preview9080 pode ser reiniciado após testes mantendo broker8883/monitor9090;
usuário definirá senha pessoal no navegador, não viachat/modelo.
