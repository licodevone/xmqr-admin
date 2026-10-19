// SPDX-License-Identifier: MIT
use crate::{
    config::Config,
    monitor,
    mqtt_console::{Board, BoardError, ConnectRequest, PublishRequest, Snapshot},
    token::Token,
    web_auth::{
        Auth, AuthConfig, AuthError, CreateUserRequest, LoginRequest, SessionInfo, SetupRequest,
        StateRequest, UserInfo,
    },
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::{future::IntoFuture, io, sync::Arc, time::Duration};
use tokio::sync::{Semaphore, oneshot};

#[derive(Clone)]
struct App {
    config: Config,
    token: Arc<Token>,
    client: reqwest::Client,
    slots: Arc<Semaphore>,
    board: Board,
    auth: Auth,
}

fn routes(
    config: Config,
    token: Token,
    board: Board,
    auth: Auth,
) -> Result<Router, reqwest::Error> {
    let app = App {
        config,
        token: Arc::new(token),
        client: monitor::client()?,
        slots: Arc::new(Semaphore::new(16)),
        board,
        auth,
    };
    Ok(build(app))
}

fn build(app: App) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/app.css", get(css))
        .route("/assets/app.js", get(js))
        .route("/favicon.svg", get(favicon))
        .route("/api/status", get(status))
        .route("/api/mqtt", get(mqtt_snapshot))
        .route("/api/mqtt/connect", post(mqtt_connect))
        .route("/api/mqtt/disconnect", post(mqtt_disconnect))
        .route("/api/mqtt/publish", post(mqtt_publish))
        .route("/api/auth/config", get(auth_config))
        .route("/api/auth/setup", post(auth_setup))
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/session", get(auth_session))
        .route("/api/auth/logout", post(auth_logout))
        .route("/api/auth/users", get(auth_users).post(auth_create_user))
        .route("/api/auth/users/state", post(auth_user_state))
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(middleware::from_fn_with_state(app.clone(), guard))
        .layer(DefaultBodyLimit::max(8192))
        .with_state(app)
}

async fn guard(State(app): State<App>, request: Request, next: Next) -> Response {
    let response = guarded(app, request, next).await;
    secure_headers(response)
}

async fn guarded(app: App, request: Request, next: Next) -> Response {
    let Ok(_permit) = app.slots.clone().try_acquire_owned() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let headers = request.headers();
    let host = headers.get(header::HOST).and_then(|v| v.to_str().ok());
    if headers.get_all(header::HOST).iter().count() != 1
        || !host.is_some_and(|host| app.config.allows_host(host))
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    if headers.get_all(header::ORIGIN).iter().count() > 1
        || headers
            .get(header::ORIGIN)
            .is_some_and(|v| !v.to_str().is_ok_and(|o| app.config.allows_origin(o)))
        || headers
            .get("sec-fetch-site")
            .is_some_and(|v| v == "cross-site")
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mqtt_write = request.method() == Method::POST
        && matches!(
            request.uri().path(),
            "/api/mqtt/connect"
                | "/api/mqtt/disconnect"
                | "/api/mqtt/publish"
                | "/api/auth/setup"
                | "/api/auth/login"
                | "/api/auth/logout"
                | "/api/auth/users"
                | "/api/auth/users/state"
        );
    if !matches!(*request.method(), Method::GET | Method::HEAD) && !mqtt_write {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    if request.uri().query().is_some()
        || headers.contains_key(header::TRANSFER_ENCODING)
        || headers.get_all(header::CONTENT_LENGTH).iter().count() > 1
        || (!mqtt_write
            && headers
                .get(header::CONTENT_LENGTH)
                .is_some_and(|v| v != "0"))
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if mqtt_write {
        if headers.get(header::ORIGIN).is_none() {
            return StatusCode::FORBIDDEN.into_response();
        }
        if headers.get_all(header::CONTENT_TYPE).iter().count() != 1
            || !headers.get(header::CONTENT_TYPE).is_some_and(|value| {
                value.to_str().is_ok_and(|value| {
                    value
                        .split(';')
                        .next()
                        .is_some_and(|mime| mime.trim() == "application/json")
                })
            })
        {
            return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
        }
        if headers.get(header::CONTENT_LENGTH).is_some_and(|value| {
            !value
                .to_str()
                .is_ok_and(|text| text.parse::<usize>().is_ok_and(|length| length <= 8192))
        }) {
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
    }
    let public_auth = matches!(
        (request.method(), request.uri().path()),
        (&Method::GET | &Method::HEAD, "/api/auth/config")
            | (&Method::POST, "/api/auth/setup" | "/api/auth/login")
    );
    if request.uri().path().starts_with("/api/") && !public_auth && app.auth.is_users() {
        let identity = match app.auth.authorize(headers, mqtt_write).await {
            Ok(identity) => identity,
            Err(error) => return error.into_response(),
        };
        if (request.uri().path().starts_with("/api/mqtt") && !identity.role.allows_mqtt())
            || (request.uri().path().starts_with("/api/auth/users") && !identity.role.is_admin())
        {
            return StatusCode::FORBIDDEN.into_response();
        }
    } else if request.uri().path().starts_with("/api/") && !public_auth {
        let candidate = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));
        if headers.get_all(header::AUTHORIZATION).iter().count() != 1
            || !candidate.is_some_and(|s| app.token.matches(s.as_bytes()))
        {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }
    tokio::time::timeout(Duration::from_secs(15), next.run(request))
        .await
        .unwrap_or_else(|_| StatusCode::REQUEST_TIMEOUT.into_response())
}

fn secure_headers(mut response: Response) -> Response {
    for (name, value) in [
        ("cache-control", "no-store"),
        (
            "content-security-policy",
            "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; base-uri 'none'; frame-ancestors 'none'; form-action 'none'",
        ),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "no-referrer"),
    ] {
        response
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
    response
}

async fn status(State(app): State<App>) -> Json<monitor::Status> {
    Json(monitor::status(&app.client, app.config.monitor).await)
}

async fn mqtt_snapshot(State(app): State<App>) -> Json<Snapshot> {
    Json(app.board.snapshot().await)
}

async fn mqtt_connect(
    State(app): State<App>,
    request: Result<Json<ConnectRequest>, JsonRejection>,
) -> Result<Json<Snapshot>, BoardError> {
    let Json(request) = request.map_err(|_| invalid_json())?;
    app.board.connect(request).await.map(Json)
}

async fn mqtt_disconnect(
    State(app): State<App>,
    request: Result<Json<serde_json::Value>, JsonRejection>,
) -> Response {
    let Ok(Json(request)) = request else {
        return invalid_json().into_response();
    };
    if !request.as_object().is_some_and(serde_json::Map::is_empty) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match app.board.disconnect().await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn mqtt_publish(
    State(app): State<App>,
    request: Result<Json<PublishRequest>, JsonRejection>,
) -> Result<Json<Snapshot>, BoardError> {
    let Json(request) = request.map_err(|_| invalid_json())?;
    app.board.publish(request).await.map(Json)
}

fn invalid_json() -> BoardError {
    BoardError {
        status: StatusCode::BAD_REQUEST,
        code: "invalid_request",
    }
}

async fn auth_config(State(app): State<App>) -> Json<AuthConfig> {
    Json(app.auth.config().await)
}

async fn auth_setup(
    State(app): State<App>,
    request: Result<Json<SetupRequest>, JsonRejection>,
) -> Response {
    let Ok(Json(request)) = request else {
        return invalid_json().into_response();
    };
    match app.auth.setup(request, &app.token).await {
        Ok(reply) => reply.into_response(),
        Err(error) => error.into_response(),
    }
}

async fn auth_login(
    State(app): State<App>,
    request: Result<Json<LoginRequest>, JsonRejection>,
) -> Response {
    let Ok(Json(request)) = request else {
        return invalid_json().into_response();
    };
    match app.auth.login(request).await {
        Ok(reply) => reply.into_response(),
        Err(error) => error.into_response(),
    }
}

async fn auth_session(
    State(app): State<App>,
    headers: HeaderMap,
) -> Result<Json<SessionInfo>, AuthError> {
    app.auth.session(&headers).await.map(Json)
}

async fn auth_logout(
    State(app): State<App>,
    headers: HeaderMap,
    request: Result<Json<serde_json::Value>, JsonRejection>,
) -> Response {
    let Ok(Json(request)) = request else {
        return invalid_json().into_response();
    };
    if !request.as_object().is_some_and(serde_json::Map::is_empty) {
        return invalid_json().into_response();
    }
    let current = match app.auth.authorize(&headers, true).await {
        Ok(identity) => identity,
        Err(error) => return error.into_response(),
    };
    if current.role.allows_mqtt() {
        let _ = app.board.disconnect().await;
    }
    match app.auth.logout(&headers).await {
        Ok(reply) => reply,
        Err(error) => error.into_response(),
    }
}

async fn auth_users(State(app): State<App>) -> Result<Json<Vec<UserInfo>>, AuthError> {
    app.auth.users().await.map(Json)
}

async fn auth_create_user(
    State(app): State<App>,
    request: Result<Json<CreateUserRequest>, JsonRejection>,
) -> Response {
    let Ok(Json(request)) = request else {
        return invalid_json().into_response();
    };
    match app.auth.create_user(request).await {
        Ok(users) => Json(users).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn auth_user_state(
    State(app): State<App>,
    request: Result<Json<StateRequest>, JsonRejection>,
) -> Response {
    let Ok(Json(request)) = request else {
        return invalid_json().into_response();
    };
    match app.auth.set_enabled(request).await {
        Ok(users) => Json(users).into_response(),
        Err(error) => error.into_response(),
    }
}

async fn favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        include_str!("../assets/favicon.svg"),
    )
}

async fn index() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../assets/index.html"),
    )
}
async fn css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../assets/app.css"),
    )
}
async fn js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../assets/app.js"),
    )
}

pub async fn serve(config: Config, token: Token) -> io::Result<()> {
    let address = config.bind;
    let board = Board::from_env()?;
    let auth = tokio::task::spawn_blocking(Auth::from_env)
        .await
        .map_err(|_| io::Error::other("autenticacao indisponivel"))??;
    let router = routes(config, token, board.clone(), auth)
        .map_err(|_| io::Error::other("unable to initialize local HTTP client"))?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let (stop, stopped) = oneshot::channel::<()>();
    let server = axum::serve(crate::listener::BoundedListener::new(listener), router)
        .with_graceful_shutdown(async {
            let _ = stopped.await;
        })
        .into_future();
    tokio::pin!(server);
    println!("XMQR Admin local: http://{address} — painel autenticado e console MQTT opcional");
    let result = tokio::select! {
        result = &mut server => result,
        () = shutdown() => {
            let _ = stop.send(());
            // A peer keeping an HTTP connection open cannot pin process shutdown.
            tokio::time::timeout(Duration::from_secs(5), &mut server).await.unwrap_or(Ok(()))
        }
    };
    board.shutdown().await;
    result
}

async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use axum::body::Body;
    use std::os::unix::fs::PermissionsExt;
    use tower::ServiceExt;

    fn test_state() -> App {
        let dir = tempfile::tempdir().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = dir.path().join("test.token");
        std::fs::write(&path, [b'a'; 64]).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        App {
            config: Config::parse("127.0.0.1:9080", "127.0.0.1:9090", path.clone()).unwrap(),
            token: Arc::new(crate::token::read(&path).unwrap()),
            client: monitor::client().unwrap(),
            slots: Arc::new(Semaphore::new(16)),
            board: Board::disabled(),
            auth: Auth::token_only(),
        }
    }

    fn application() -> Router {
        build(test_state())
    }

    #[tokio::test]
    async fn exhausted_request_slots_refuse_and_release_restores_service() {
        let state = test_state();
        let occupied = state.slots.clone().acquire_many_owned(16).await.unwrap();
        let router = build(state);
        let request = || {
            Request::builder()
                .uri("/")
                .header("host", "localhost:9080")
                .body(Body::empty())
                .unwrap()
        };
        let response = router.clone().oneshot(request()).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()["cache-control"], "no-store");
        drop(occupied);
        assert_eq!(
            router.oneshot(request()).await.unwrap().status(),
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn duplicate_host_auth_and_chunked_body_are_refused() {
        for (name, value, code) in [
            ("host", "localhost:9080", 403),
            ("authorization", "Bearer wrong", 401),
            ("transfer-encoding", "chunked", 400),
        ] {
            let response = application()
                .oneshot(
                    Request::builder()
                        .uri("/api/status")
                        .header("host", "localhost:9080")
                        .header("authorization", format!("Bearer {}", "a".repeat(64)))
                        .header(name, value)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status().as_u16(), code);
        }
    }

    #[tokio::test]
    async fn shell_is_public_but_every_api_requires_exact_bearer() {
        let response = application()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("host", "localhost:9080")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert!(
            response.headers()["content-security-policy"]
                .to_str()
                .unwrap()
                .contains("frame-ancestors 'none'")
        );
        assert!(
            !response
                .headers()
                .contains_key("access-control-allow-origin")
        );
        for value in ["", "Bearer wrong", "Basic secret"] {
            let response = application()
                .oneshot(
                    Request::builder()
                        .uri("/api/status")
                        .header("host", "localhost:9080")
                        .header("authorization", value)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn remote_origin_rebinding_query_body_and_write_methods_fail_closed() {
        for (method, host, origin, uri, length, code) in [
            (
                "GET",
                "evil.test:9080",
                "http://localhost:9080",
                "/",
                "0",
                403,
            ),
            ("GET", "localhost:9080", "http://evil.test", "/", "0", 403),
            ("GET", "localhost:9080", "null", "/", "0", 403),
            (
                "POST",
                "localhost:9080",
                "http://localhost:9080",
                "/",
                "0",
                405,
            ),
            (
                "GET",
                "localhost:9080",
                "http://localhost:9080",
                "/?token=bad",
                "0",
                400,
            ),
            (
                "GET",
                "localhost:9080",
                "http://localhost:9080",
                "/",
                "10",
                400,
            ),
        ] {
            let response = application()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .header("host", host)
                        .header("origin", origin)
                        .header("content-length", length)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status().as_u16(), code);
        }
    }

    #[tokio::test]
    async fn mqtt_writes_require_local_origin_json_limits_and_authentication() {
        for (origin, content_type, length, auth, expected) in [
            (None, "application/json", "2", true, 403),
            (Some("http://evil.test"), "application/json", "2", true, 403),
            (Some("http://localhost:9080"), "text/plain", "2", true, 415),
            (
                Some("http://localhost:9080"),
                "application/json",
                "8193",
                true,
                413,
            ),
            (
                Some("http://localhost:9080"),
                "application/json",
                "2",
                false,
                401,
            ),
        ] {
            let mut request = Request::builder()
                .method("POST")
                .uri("/api/mqtt/connect")
                .header("host", "localhost:9080")
                .header("content-type", content_type)
                .header("content-length", length);
            if let Some(origin) = origin {
                request = request.header("origin", origin);
            }
            if auth {
                request = request.header("authorization", format!("Bearer {}", "a".repeat(64)));
            }
            let response = application()
                .oneshot(request.body(Body::from("{}")).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status().as_u16(), expected);
            assert_eq!(response.headers()["cache-control"], "no-store");
        }
    }
}
