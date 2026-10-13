// SPDX-License-Identifier: MIT
use crate::{config::Config, monitor, token::Token};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use std::{future::IntoFuture, io, sync::Arc, time::Duration};
use tokio::sync::{Semaphore, oneshot};

#[derive(Clone)]
struct App {
    config: Config,
    token: Arc<Token>,
    client: reqwest::Client,
    slots: Arc<Semaphore>,
}

fn routes(config: Config, token: Token) -> Result<Router, reqwest::Error> {
    let app = App {
        config,
        token: Arc::new(token),
        client: monitor::client()?,
        slots: Arc::new(Semaphore::new(16)),
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
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(middleware::from_fn_with_state(app.clone(), guard))
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
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    if request.uri().query().is_some()
        || headers.contains_key(header::TRANSFER_ENCODING)
        || headers.get_all(header::CONTENT_LENGTH).iter().count() > 1
        || headers
            .get(header::CONTENT_LENGTH)
            .is_some_and(|v| v != "0")
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if request.uri().path().starts_with("/api/") {
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
    next.run(request).await
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
    let router = routes(config, token)
        .map_err(|_| io::Error::other("unable to initialize local HTTP client"))?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let (stop, stopped) = oneshot::channel::<()>();
    let server = axum::serve(crate::listener::BoundedListener::new(listener), router)
        .with_graceful_shutdown(async {
            let _ = stopped.await;
        })
        .into_future();
    tokio::pin!(server);
    println!("XMQR Admin local: http://{address} — painel autenticado somente leitura");
    tokio::select! {
        result = &mut server => result,
        () = shutdown() => {
            let _ = stop.send(());
            // A peer keeping an HTTP connection open cannot pin process shutdown.
            tokio::time::timeout(Duration::from_secs(5), &mut server).await.unwrap_or(Ok(()))
        }
    }
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
}
