// SPDX-License-Identifier: MIT
//! Contas web locais: independentes de usuários MQTT e do token de bootstrap.
use crate::token::Token;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    future::{Ready, ready},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use tokio::sync::Semaphore;
use zeroize::{Zeroize, Zeroizing};

const MAX_USERS: usize = 64;
const MAX_SESSIONS: usize = 64;
const MAX_FILE: usize = 65536;
const ABSOLUTE_LIFETIME: Duration = Duration::from_secs(8 * 3600);
const IDLE_LIFETIME: Duration = Duration::from_secs(30 * 60);
const RATE_WINDOW: Duration = Duration::from_secs(60);
const RATE_LIMIT: u32 = 20;
const COOKIE: &str = "xmqr_session";

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Viewer,
    Operator,
    Admin,
}
impl Role {
    pub const fn allows_mqtt(self) -> bool {
        matches!(self, Self::Operator | Self::Admin)
    }
    pub const fn is_admin(self) -> bool {
        matches!(self, Self::Admin)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupRequest {
    pub bootstrap_token: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
}
impl Drop for SetupRequest {
    fn drop(&mut self) {
        self.password.zeroize();
        self.bootstrap_token.zeroize();
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
impl Drop for LoginRequest {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub role: Role,
}
impl Drop for CreateUserRequest {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateRequest {
    pub username: String,
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct AuthConfig {
    pub mode: &'static str,
    pub setup_required: bool,
}
#[derive(Clone, Serialize)]
pub struct SessionInfo {
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub csrf_token: String,
}
#[derive(Clone, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub enabled: bool,
}
pub struct Identity {
    pub role: Role,
}

#[derive(Debug)]
pub struct AuthError {
    pub status: StatusCode,
    pub code: &'static str,
}
impl AuthError {
    const fn new(status: StatusCode, code: &'static str) -> Self {
        Self { status, code }
    }
    const fn invalid() -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_request")
    }
    const fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "invalid_credentials")
    }
    const fn unavailable() -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, "auth_unavailable")
    }
    const fn limited() -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, "auth_rate_limited")
    }
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({"error":self.code}))).into_response()
    }
}
pub struct LoginReply {
    pub info: SessionInfo,
    cookie: String,
}
impl IntoResponse for LoginReply {
    fn into_response(self) -> Response {
        let mut response = Json(self.info).into_response();
        if let Ok(cookie) = HeaderValue::from_str(&self.cookie) {
            response.headers_mut().insert(header::SET_COOKIE, cookie);
        }
        response
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct User {
    username: String,
    display_name: String,
    password_hash: String,
    role: Role,
    enabled: bool,
}
impl User {
    fn info(&self) -> UserInfo {
        UserInfo {
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            role: self.role,
            enabled: self.enabled,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFile {
    version: u32,
    users: Vec<User>,
}
struct Session {
    username: String,
    csrf: String,
    created: Instant,
    last_seen: Instant,
}
impl Session {
    fn active(&self, now: Instant) -> bool {
        now.duration_since(self.created) < ABSOLUTE_LIFETIME
            && now.duration_since(self.last_seen) < IDLE_LIFETIME
    }
}
struct State {
    file: UserFile,
    sessions: HashMap<[u8; 32], Session>,
    attempts: u32,
    window: Instant,
}
struct Inner {
    path: Option<PathBuf>,
    _lock: Option<File>,
    state: Mutex<State>,
    writes: Mutex<()>,
    kdf: Arc<Semaphore>,
    dummy: String,
    unhealthy: AtomicBool,
}
#[derive(Clone)]
pub struct Auth {
    inner: Arc<Inner>,
}

fn invalid_store() -> io::Error {
    io::Error::other("arquivo local de usuarios web invalido ou indisponivel")
}
fn username(value: &str) -> Option<String> {
    (value.len() >= 3
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')))
    .then(|| value.to_ascii_lowercase())
}
fn valid_name(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 80 && !value.chars().any(char::is_control)
}
fn valid_password(value: &str) -> bool {
    (8..=128).contains(&value.len()) && !value.chars().any(char::is_control)
}
fn random_secret() -> Result<Zeroizing<String>, AuthError> {
    let mut bytes = Zeroizing::new([0_u8; 32]);
    OsRng
        .try_fill_bytes(bytes.as_mut())
        .map_err(|_| AuthError::unavailable())?;
    let mut value = Zeroizing::new(String::with_capacity(64));
    for byte in bytes.iter() {
        use std::fmt::Write as _;
        write!(&mut *value, "{byte:02x}").expect("escrita em String");
    }
    Ok(value)
}
fn hash_password(password: &str) -> Result<String, AuthError> {
    let mut salt = [0_u8; 16];
    OsRng
        .try_fill_bytes(&mut salt)
        .map_err(|_| AuthError::unavailable())?;
    let salt = SaltString::encode_b64(&salt).map_err(|_| AuthError::unavailable())?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AuthError::unavailable())
}
fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
}

fn valid_hash(value: &str) -> bool {
    if value.len() > 256 || !value.starts_with("$argon2id$v=19$m=19456,t=2,p=1$") {
        return false;
    }
    PasswordHash::new(value).is_ok_and(|hash| {
        let mut salt = [0; 16];
        hash.salt.is_some_and(|value| {
            value
                .decode_b64(&mut salt)
                .is_ok_and(|bytes| bytes.len() == 16)
        }) && hash.hash.is_some_and(|value| value.as_bytes().len() == 32)
    })
}
fn digest(value: &[u8]) -> [u8; 32] {
    Sha256::digest(value).into()
}
fn cookie(value: &str) -> String {
    format!("{COOKIE}={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age=28800")
}
fn cookie_digest(headers: &HeaderMap) -> Result<[u8; 32], AuthError> {
    if headers.get_all(header::COOKIE).iter().count() != 1 {
        return Err(AuthError::unauthorized());
    }
    let values = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(AuthError::unauthorized)?;
    let mut found = None;
    for pair in values.split(';') {
        if let Some((name, value)) = pair.trim().split_once('=')
            && name == COOKIE
        {
            if found.is_some()
                || value.len() != 64
                || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(AuthError::unauthorized());
            }
            found = Some(digest(value.as_bytes()));
        }
    }
    found.ok_or_else(AuthError::unauthorized)
}

impl Auth {
    pub fn token_only() -> Self {
        Self::new(
            None,
            None,
            UserFile {
                version: 1,
                users: Vec::new(),
            },
            String::new(),
        )
    }
    fn new(path: Option<PathBuf>, lock: Option<File>, file: UserFile, dummy: String) -> Self {
        Self {
            inner: Arc::new(Inner {
                path,
                _lock: lock,
                state: Mutex::new(State {
                    file,
                    sessions: HashMap::new(),
                    attempts: 0,
                    window: Instant::now(),
                }),
                writes: Mutex::new(()),
                kdf: Arc::new(Semaphore::new(2)),
                dummy,
                unhealthy: AtomicBool::new(false),
            }),
        }
    }
    pub fn from_env() -> io::Result<Self> {
        let Some(path) = std::env::var_os("XMQR_ADMIN_WEB_USERS_FILE") else {
            return Ok(Self::token_only());
        };
        Self::from_path(PathBuf::from(path))
    }
    #[cfg(unix)]
    fn from_path(path: PathBuf) -> io::Result<Self> {
        use fs2::FileExt;
        use std::os::unix::fs::OpenOptionsExt;
        private_parent(&path)?;
        let mut name = path.as_os_str().to_os_string();
        name.push(".lock");
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(PathBuf::from(name))
            .map_err(|_| invalid_store())?;
        private_metadata(&lock)?;
        FileExt::try_lock_exclusive(&lock).map_err(|_| invalid_store())?;
        let file = match fs::symlink_metadata(&path) {
            Ok(_) => read_store(&path)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => UserFile {
                version: 1,
                users: Vec::new(),
            },
            Err(_) => return Err(invalid_store()),
        };
        let password = random_secret().map_err(|_| invalid_store())?;
        let dummy = hash_password(&password).map_err(|_| invalid_store())?;
        Ok(Self::new(Some(path), Some(lock), file, dummy))
    }
    #[cfg(not(unix))]
    fn from_path(_: PathBuf) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "usuarios web privados requerem Unix",
        ))
    }
    pub fn is_users(&self) -> bool {
        self.inner.path.is_some()
    }
    pub fn config(&self) -> Ready<AuthConfig> {
        ready(AuthConfig {
            mode: if self.is_users() { "users" } else { "token" },
            setup_required: self.is_users()
                && self
                    .inner
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .file
                    .users
                    .is_empty(),
        })
    }
    fn available(&self) -> Result<(), AuthError> {
        if !self.is_users() || self.inner.unhealthy.load(Ordering::Acquire) {
            return Err(AuthError::unavailable());
        }
        Ok(())
    }
    fn attempt(&self) -> Result<(), AuthError> {
        self.available()?;
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.window.elapsed() >= RATE_WINDOW {
            state.window = Instant::now();
            state.attempts = 0;
        }
        if state.attempts >= RATE_LIMIT {
            return Err(AuthError::limited());
        }
        state.attempts += 1;
        Ok(())
    }
    fn commit(&self, file: &UserFile) -> Result<(), AuthError> {
        self.available()?;
        let result = self
            .inner
            .path
            .as_ref()
            .ok_or_else(AuthError::unavailable)
            .and_then(|path| write_store(path, file).map_err(|_| AuthError::unavailable()));
        if result.is_err() {
            self.inner.unhealthy.store(true, Ordering::Release);
        }
        result
    }
    fn issue(&self, state: &mut State, name: &str) -> Result<LoginReply, AuthError> {
        self.available()?;
        let now = Instant::now();
        state.sessions.retain(|_, session| session.active(now));
        if state.sessions.len() >= MAX_SESSIONS {
            return Err(AuthError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "session_limit",
            ));
        }
        let user = state
            .file
            .users
            .iter()
            .find(|user| user.username == name && user.enabled)
            .ok_or_else(AuthError::unauthorized)?;
        let secret = random_secret()?;
        let csrf = random_secret()?;
        let info = SessionInfo {
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            role: user.role,
            csrf_token: csrf.to_string(),
        };
        state.sessions.insert(
            digest(secret.as_bytes()),
            Session {
                username: name.into(),
                csrf: csrf.to_string(),
                created: now,
                last_seen: now,
            },
        );
        Ok(LoginReply {
            info,
            cookie: cookie(&secret),
        })
    }
    pub async fn setup(
        &self,
        mut request: SetupRequest,
        token: &Token,
    ) -> Result<LoginReply, AuthError> {
        self.available()?;
        if !self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .file
            .users
            .is_empty()
        {
            return Err(AuthError::new(StatusCode::CONFLICT, "setup_complete"));
        }
        self.attempt()?;
        if !token.matches(request.bootstrap_token.as_bytes()) {
            return Err(AuthError::unauthorized());
        }
        let name = username(&request.username).ok_or_else(AuthError::invalid)?;
        if !valid_name(&request.display_name) || !valid_password(&request.password) {
            return Err(AuthError::invalid());
        }
        let password = Zeroizing::new(std::mem::take(&mut request.password));
        let display_name = std::mem::take(&mut request.display_name);
        let permit = self
            .inner
            .kdf
            .clone()
            .try_acquire_owned()
            .map_err(|_| AuthError::limited())?;
        let auth = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let password_hash = hash_password(&password)?;
            let _write = auth
                .inner
                .writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let state = auth
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !state.file.users.is_empty() {
                return Err(AuthError::new(StatusCode::CONFLICT, "setup_complete"));
            }
            drop(state);
            let file = UserFile {
                version: 1,
                users: vec![User {
                    username: name.clone(),
                    display_name,
                    password_hash,
                    role: Role::Admin,
                    enabled: true,
                }],
            };
            auth.commit(&file)?;
            let mut state = auth
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.file = file;
            auth.issue(&mut state, &name)
        })
        .await
        .map_err(|_| AuthError::unavailable())?
    }
    pub async fn login(&self, mut request: LoginRequest) -> Result<LoginReply, AuthError> {
        self.attempt()?;
        let name = username(&request.username);
        let password = Zeroizing::new(std::mem::take(&mut request.password));
        let permit = self
            .inner
            .kdf
            .clone()
            .try_acquire_owned()
            .map_err(|_| AuthError::limited())?;
        let auth = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let (hash, enabled) = {
                let state = auth
                    .inner
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let user = name
                    .as_ref()
                    .and_then(|name| state.file.users.iter().find(|user| user.username == *name));
                user.filter(|user| user.enabled).map_or_else(
                    || (auth.inner.dummy.clone(), false),
                    |user| (user.password_hash.clone(), true),
                )
            };
            // A conta ausente ou desabilitada executa o mesmo Argon2id da conta real.
            let bounded_password = if valid_password(&password) {
                password.as_str()
            } else {
                "invalid-password"
            };
            let verified = verify_password(bounded_password, &hash);
            if !enabled || !verified || !valid_password(&password) {
                return Err(AuthError::unauthorized());
            }
            auth.available()?;
            let mut state = auth
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            auth.issue(
                &mut state,
                name.as_deref().ok_or_else(AuthError::unauthorized)?,
            )
        })
        .await
        .map_err(|_| AuthError::unavailable())?
    }
    fn lookup(&self, headers: &HeaderMap, write: bool) -> Result<SessionInfo, AuthError> {
        self.available()?;
        let key = cookie_digest(headers)?;
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        state.sessions.retain(|_, session| session.active(now));
        let session = state
            .sessions
            .get(&key)
            .ok_or_else(AuthError::unauthorized)?;
        if write {
            let csrf = headers
                .get("x-xmqr-csrf")
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| AuthError::new(StatusCode::FORBIDDEN, "csrf_required"))?;
            if headers.get_all("x-xmqr-csrf").iter().count() != 1
                || csrf.len() != session.csrf.len()
                || !bool::from(csrf.as_bytes().ct_eq(session.csrf.as_bytes()))
            {
                return Err(AuthError::new(StatusCode::FORBIDDEN, "csrf_required"));
            }
        }
        let user = state
            .file
            .users
            .iter()
            .find(|user| user.username == session.username && user.enabled)
            .ok_or_else(AuthError::unauthorized)?;
        let info = SessionInfo {
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            role: user.role,
            csrf_token: session.csrf.clone(),
        };
        if let Some(session) = state.sessions.get_mut(&key) {
            session.last_seen = now;
        }
        Ok(info)
    }
    pub fn authorize(
        &self,
        headers: &HeaderMap,
        write: bool,
    ) -> Ready<Result<Identity, AuthError>> {
        ready(
            self.lookup(headers, write)
                .map(|session| Identity { role: session.role }),
        )
    }
    pub fn session(&self, headers: &HeaderMap) -> Ready<Result<SessionInfo, AuthError>> {
        ready(self.lookup(headers, false))
    }
    pub fn logout(&self, headers: &HeaderMap) -> Ready<Result<Response, AuthError>> {
        ready((|| {
            self.lookup(headers, true)?;
            self.inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .sessions
                .remove(&cookie_digest(headers)?);
            let mut response = Json(serde_json::json!({"ok":true})).into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_static(
                    "xmqr_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
                ),
            );
            Ok(response)
        })())
    }
    pub fn users(&self) -> Ready<Result<Vec<UserInfo>, AuthError>> {
        ready(self.available().map(|()| {
            self.inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .file
                .users
                .iter()
                .map(User::info)
                .collect()
        }))
    }
    pub async fn create_user(
        &self,
        mut request: CreateUserRequest,
    ) -> Result<Vec<UserInfo>, AuthError> {
        self.available()?;
        let name = username(&request.username).ok_or_else(AuthError::invalid)?;
        if !valid_name(&request.display_name) || !valid_password(&request.password) {
            return Err(AuthError::invalid());
        }
        let password = Zeroizing::new(std::mem::take(&mut request.password));
        let display_name = std::mem::take(&mut request.display_name);
        let role = request.role;
        let permit = self
            .inner
            .kdf
            .clone()
            .try_acquire_owned()
            .map_err(|_| AuthError::limited())?;
        let auth = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let password_hash = hash_password(&password)?;
            let _write = auth
                .inner
                .writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            auth.available()?;
            let mut file = auth
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .file
                .clone();
            if file.users.len() >= MAX_USERS {
                return Err(AuthError::new(StatusCode::CONFLICT, "user_limit"));
            }
            if file.users.iter().any(|user| user.username == name) {
                return Err(AuthError::new(StatusCode::CONFLICT, "user_exists"));
            }
            file.users.push(User {
                username: name,
                display_name,
                password_hash,
                role,
                enabled: true,
            });
            auth.commit(&file)?;
            let mut state = auth
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.file = file;
            Ok(state.file.users.iter().map(User::info).collect())
        })
        .await
        .map_err(|_| AuthError::unavailable())?
    }
    pub async fn set_enabled(&self, request: StateRequest) -> Result<Vec<UserInfo>, AuthError> {
        self.available()?;
        let name = username(&request.username).ok_or_else(AuthError::invalid)?;
        let permit = self
            .inner
            .kdf
            .clone()
            .try_acquire_owned()
            .map_err(|_| AuthError::limited())?;
        let auth = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let _write = auth
                .inner
                .writes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            auth.available()?;
            let mut file = auth
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .file
                .clone();
            let index = file
                .users
                .iter()
                .position(|user| user.username == name)
                .ok_or_else(|| AuthError::new(StatusCode::NOT_FOUND, "user_not_found"))?;
            let user = &file.users[index];
            if !request.enabled
                && user.enabled
                && user.role.is_admin()
                && file
                    .users
                    .iter()
                    .filter(|user| user.enabled && user.role.is_admin())
                    .count()
                    == 1
            {
                return Err(AuthError::new(StatusCode::CONFLICT, "last_admin"));
            }
            file.users[index].enabled = request.enabled;
            auth.commit(&file)?;
            let mut state = auth
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.file = file;
            if !request.enabled {
                state.sessions.retain(|_, session| session.username != name);
            }
            Ok(state.file.users.iter().map(User::info).collect())
        })
        .await
        .map_err(|_| AuthError::unavailable())?
    }
}

fn validate_store(file: &UserFile) -> io::Result<()> {
    if file.version != 1 || file.users.len() > MAX_USERS {
        return Err(invalid_store());
    }
    let mut names = HashSet::new();
    for user in &file.users {
        if username(&user.username).as_deref() != Some(user.username.as_str())
            || !names.insert(&user.username)
            || !valid_name(&user.display_name)
            || !valid_hash(&user.password_hash)
        {
            return Err(invalid_store());
        }
    }
    if !file.users.is_empty()
        && !file
            .users
            .iter()
            .any(|user| user.enabled && user.role.is_admin())
    {
        return Err(invalid_store());
    }
    Ok(())
}

#[cfg(unix)]
fn private_parent(path: &Path) -> io::Result<&Path> {
    use std::os::unix::fs::PermissionsExt;
    let parent = path
        .parent()
        .filter(|parent| path.is_absolute() && !parent.as_os_str().is_empty())
        .ok_or_else(invalid_store)?;
    let metadata = fs::symlink_metadata(parent).map_err(|_| invalid_store())?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o777 != 0o700 {
        return Err(invalid_store());
    }
    Ok(parent)
}
#[cfg(unix)]
fn private_metadata(file: &File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = file.metadata().map_err(|_| invalid_store())?;
    if !metadata.is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.len() > MAX_FILE as u64
    {
        return Err(invalid_store());
    }
    Ok(())
}
#[cfg(unix)]
fn read_store(path: &Path) -> io::Result<UserFile> {
    use std::os::unix::fs::OpenOptionsExt;
    private_parent(path)?;
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| invalid_store())?;
    private_metadata(&file)?;
    let mut bytes = Vec::new();
    file.take((MAX_FILE + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid_store())?;
    if bytes.len() > MAX_FILE {
        return Err(invalid_store());
    }
    let file: UserFile = serde_json::from_slice(&bytes).map_err(|_| invalid_store())?;
    validate_store(&file)?;
    Ok(file)
}
#[cfg(unix)]
fn write_store(path: &Path, file: &UserFile) -> io::Result<()> {
    private_parent(path)?;
    validate_store(file)?;
    if fs::symlink_metadata(path).is_ok() {
        let _ = read_store(path)?;
    }
    let parent = path.parent().ok_or_else(invalid_store)?;
    let bytes = serde_json::to_vec(file).map_err(|_| invalid_store())?;
    if bytes.len() > MAX_FILE {
        return Err(invalid_store());
    }
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|_| invalid_store())?;
    temporary.write_all(&bytes).map_err(|_| invalid_store())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|_| invalid_store())?;
    temporary.persist(path).map_err(|_| invalid_store())?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| invalid_store())
}
#[cfg(not(unix))]
fn write_store(_: &Path, _: &UserFile) -> io::Result<()> {
    Err(invalid_store())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn fixture() -> (tempfile::TempDir, Auth, Token) {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let token_path = directory.path().join("token");
        crate::token::create(&token_path).unwrap();
        let token = crate::token::read(&token_path).unwrap();
        let auth = Auth::from_path(directory.path().join("users.json")).unwrap();
        (directory, auth, token)
    }
    async fn bootstrap(auth: &Auth, token: &Token) -> LoginReply {
        match auth
            .setup(
                SetupRequest {
                    bootstrap_token: token.as_str().into(),
                    username: "Admin".into(),
                    display_name: "Admin".into(),
                    password: "test-password-42".into(),
                },
                token,
            )
            .await
        {
            Ok(reply) => reply,
            Err(error) => panic!("bootstrap: {}", error.code),
        }
    }
    fn headers(reply: &LoginReply, csrf: bool) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(reply.cookie.split(';').next().unwrap()).unwrap(),
        );
        if csrf {
            headers.insert(
                "x-xmqr-csrf",
                HeaderValue::from_str(&reply.info.csrf_token).unwrap(),
            );
        }
        headers
    }

    #[test]
    fn roles_names_and_strict_schemas() {
        assert!(!Role::Viewer.allows_mqtt());
        assert!(Role::Operator.allows_mqtt());
        assert!(!Role::Operator.is_admin());
        assert!(Role::Admin.is_admin());
        assert_eq!(username("User_Name-42").as_deref(), Some("user_name-42"));
        for name in ["a", "../../etc", "ábc", "a b"] {
            assert!(username(name).is_none());
        }
        assert!(!valid_password("short"));
        assert!(!valid_password("abcdefgh\0"));
        assert!(!valid_password(&"a".repeat(129)));
        assert!(
            serde_json::from_str::<LoginRequest>(
                r#"{"username":"user","password":"test-password","role":"admin"}"#
            )
            .is_err()
        );
        assert!(serde_json::from_str::<CreateUserRequest>(r#"{"username":"user","display_name":"User","password":"test-password","role":"root"}"#).is_err());
    }

    #[test]
    fn password_hashes_use_independent_salts_and_bounded_costs() {
        let first = hash_password("test-password-42").unwrap();
        let second = hash_password("test-password-42").unwrap();
        assert_ne!(first, second);
        assert!(valid_hash(&first));
        assert!(verify_password("test-password-42", &first));
        assert!(!verify_password("wrong-password", &first));
        assert!(!valid_hash(&first.replace("m=19456", "m=4294967295")));
        assert!(!valid_hash(&first.replace("t=2", "t=4294967295")));
        assert!(!valid_hash(&first.replace("argon2id", "argon2i")));
        assert!(!valid_hash("invalid"));
    }

    #[test]
    fn version_duplicates_and_user_count_are_bounded_without_kdf() {
        let hash = hash_password("test-password-42").unwrap();
        let admin = User {
            username: "admin".into(),
            display_name: "Admin".into(),
            password_hash: hash.clone(),
            role: Role::Admin,
            enabled: true,
        };
        let mut file = UserFile {
            version: 1,
            users: vec![admin.clone()],
        };
        assert!(validate_store(&file).is_ok());
        file.version = 2;
        assert!(validate_store(&file).is_err());
        file.version = 1;
        file.users.push(admin);
        assert!(validate_store(&file).is_err());
        file.users.pop();
        for index in 1..MAX_USERS {
            file.users.push(User {
                username: format!("user{index}"),
                display_name: "Viewer".into(),
                password_hash: hash.clone(),
                role: Role::Viewer,
                enabled: true,
            });
        }
        assert!(validate_store(&file).is_ok());
        file.users.push(User {
            username: "overflow".into(),
            display_name: "Extra".into(),
            password_hash: hash,
            role: Role::Viewer,
            enabled: true,
        });
        assert!(validate_store(&file).is_err());
        file.users.pop();
        file.users[0].enabled = false;
        assert!(validate_store(&file).is_err());
    }

    #[test]
    fn absolute_and_idle_session_expiry_are_independent() {
        let now = Instant::now();
        let session = Session {
            username: "admin".into(),
            csrf: "test".into(),
            created: now.checked_sub(ABSOLUTE_LIFETIME).unwrap(),
            last_seen: now,
        };
        assert!(!session.active(now));
        let session = Session {
            username: "admin".into(),
            csrf: "test".into(),
            created: now.checked_sub(IDLE_LIFETIME).unwrap(),
            last_seen: now.checked_sub(IDLE_LIFETIME).unwrap(),
        };
        assert!(!session.active(now));
        let session = Session {
            username: "admin".into(),
            csrf: "test".into(),
            created: now,
            last_seen: now,
        };
        assert!(session.active(now));
    }

    #[tokio::test]
    async fn bootstrap_is_single_private_hashed_and_sessions_are_revoked() {
        let (directory, auth, token) = fixture();
        assert!(auth.config().await.setup_required);
        let reply = bootstrap(&auth, &token).await;
        assert!(!auth.config().await.setup_required);
        assert!(matches!(
            auth.setup(
                SetupRequest {
                    bootstrap_token: token.as_str().into(),
                    username: "second".into(),
                    display_name: "Second".into(),
                    password: "test-password".into()
                },
                &token
            )
            .await,
            Err(AuthError {
                status: StatusCode::CONFLICT,
                ..
            })
        ));
        let bytes = fs::read(directory.path().join("users.json")).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("$argon2id$"));
        assert!(!text.contains("test-password-42"));
        assert!(!text.contains(token.as_str()));
        assert!(reply.cookie.contains("HttpOnly; SameSite=Strict; Path=/"));
        assert!(matches!(
            auth.authorize(&headers(&reply, false), true).await,
            Err(AuthError {
                status: StatusCode::FORBIDDEN,
                ..
            })
        ));
        assert!(
            auth.authorize(&headers(&reply, true), true)
                .await
                .unwrap()
                .role
                .is_admin()
        );
        assert!(auth.logout(&headers(&reply, true)).await.is_ok());
        assert!(auth.session(&headers(&reply, false)).await.is_err());
        assert!(Auth::from_path(directory.path().join("users.json")).is_err());
        drop(auth);
        let restarted = Auth::from_path(directory.path().join("users.json")).unwrap();
        assert!(restarted.session(&headers(&reply, false)).await.is_err());
        assert!(!restarted.config().await.setup_required);
    }

    #[tokio::test]
    async fn login_creation_disable_and_last_admin_protection() {
        let (_directory, auth, token) = fixture();
        let _ = bootstrap(&auth, &token).await;
        assert!(
            auth.create_user(CreateUserRequest {
                username: "Viewer".into(),
                display_name: "Viewer".into(),
                password: "test-viewer-password".into(),
                role: Role::Viewer
            })
            .await
            .is_ok()
        );
        for name in ["absent", "viewer"] {
            assert!(matches!(
                auth.login(LoginRequest {
                    username: name.into(),
                    password: "wrong-password".into()
                })
                .await,
                Err(AuthError {
                    status: StatusCode::UNAUTHORIZED,
                    ..
                })
            ));
        }
        let reply = match auth
            .login(LoginRequest {
                username: "VIEWER".into(),
                password: "test-viewer-password".into(),
            })
            .await
        {
            Ok(reply) => reply,
            Err(error) => panic!("login: {}", error.code),
        };
        assert!(!reply.info.role.allows_mqtt());
        assert!(
            auth.set_enabled(StateRequest {
                username: "viewer".into(),
                enabled: false
            })
            .await
            .is_ok()
        );
        assert!(auth.session(&headers(&reply, false)).await.is_err());
        assert!(
            auth.login(LoginRequest {
                username: "viewer".into(),
                password: "test-viewer-password".into()
            })
            .await
            .is_err()
        );
        assert!(matches!(
            auth.set_enabled(StateRequest {
                username: "admin".into(),
                enabled: false
            })
            .await,
            Err(AuthError {
                code: "last_admin",
                ..
            })
        ));
        let users = serde_json::to_string(&auth.users().await.unwrap()).unwrap();
        assert!(!users.contains("password"));
        assert!(!users.contains("argon2"));
    }

    #[tokio::test]
    async fn expiry_session_cap_rate_and_duplicate_cookie_are_bounded() {
        let (_directory, auth, token) = fixture();
        let reply = bootstrap(&auth, &token).await;
        let mut duplicate = headers(&reply, false);
        duplicate.append(
            header::COOKIE,
            duplicate.get(header::COOKIE).unwrap().clone(),
        );
        assert!(auth.session(&duplicate).await.is_err());
        {
            let mut state = auth.inner.state.lock().unwrap();
            let now = Instant::now();
            let key = cookie_digest(&headers(&reply, false)).unwrap();
            state.sessions.get_mut(&key).unwrap().last_seen =
                now.checked_sub(IDLE_LIFETIME).unwrap();
        }
        assert!(auth.session(&headers(&reply, false)).await.is_err());
        {
            let mut state = auth.inner.state.lock().unwrap();
            for _ in 0..MAX_SESSIONS {
                assert!(auth.issue(&mut state, "admin").is_ok());
            }
            assert!(matches!(
                auth.issue(&mut state, "admin"),
                Err(AuthError {
                    code: "session_limit",
                    ..
                })
            ));
            state.attempts = RATE_LIMIT;
        }
        assert!(matches!(
            auth.login(LoginRequest {
                username: "admin".into(),
                password: "test-password-42".into()
            })
            .await,
            Err(AuthError {
                status: StatusCode::TOO_MANY_REQUESTS,
                ..
            })
        ));
    }

    #[test]
    fn malformed_or_public_store_is_preserved_and_never_bootstrapped() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = directory.path().join("users.json");
        fs::write(&path, b"invalid json").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(Auth::from_path(path.clone()).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"invalid json");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(Auth::from_path(path.clone()).is_err());
        let link = directory.path().join("linked.json");
        symlink(&path, &link).unwrap();
        assert!(Auth::from_path(link).is_err());
    }
}
