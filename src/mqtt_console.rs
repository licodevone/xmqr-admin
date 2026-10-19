// SPDX-License-Identifier: MIT
//! Console MQTT local. Somente perfis definidos pelo operador chegam ao processo.
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    env, io,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, Command},
    sync::{Mutex as AsyncMutex, oneshot},
    task::JoinHandle,
};

const MAX_PAYLOAD: usize = 4096;
const MAX_LINE: usize = 32768;
const MAX_MESSAGES: usize = 200;
const LEASE: Duration = Duration::from_secs(60);
const CONNECT_DEADLINE: Duration = Duration::from_secs(12);
const PUBLISH_DEADLINE: Duration = Duration::from_secs(6);

fn client_id(bytes: [u8; 8]) -> String {
    let mut id = String::from("xw-");
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut id, "{byte:02x}").expect("escrita em String");
    }
    id
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectRequest {
    pub topic: String,
    pub qos: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishRequest {
    pub topic: String,
    pub payload: String,
    pub qos: u8,
    pub retain: bool,
}

#[derive(Clone, Serialize)]
pub struct Message {
    id: u64,
    received_at: u64,
    topic: String,
    payload: String,
    qos: u8,
    retain: bool,
    direction: &'static str,
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    enabled: bool,
    label: String,
    status: &'static str,
    error: Option<&'static str>,
    topic: Option<String>,
    qos: Option<u8>,
    messages: Vec<Message>,
    received_total: u64,
    published_total: u64,
    dropped_total: u64,
}

/// Agregado público sem tópicos, mensagens ou acesso aos perfis privados.
#[derive(Clone, Copy, Serialize)]
pub struct Availability {
    pub online: bool,
    pub reason: &'static str,
}

pub struct BoardError {
    pub status: StatusCode,
    pub code: &'static str,
}
impl BoardError {
    const fn new(status: StatusCode, code: &'static str) -> Self {
        Self { status, code }
    }
    const fn invalid() -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_request")
    }
    const fn unavailable() -> Self {
        Self::new(StatusCode::BAD_GATEWAY, "client_unavailable")
    }
}
impl IntoResponse for BoardError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({"error": self.code}))).into_response()
    }
}

struct Settings {
    binary: PathBuf,
    sub_profile: PathBuf,
    pub_profile: PathBuf,
    client_id: String,
}
struct State {
    snapshot: Snapshot,
    messages: VecDeque<Message>,
    next_message: u64,
    touched: Instant,
    closed: bool,
}
struct Subscription {
    cancel: oneshot::Sender<()>,
    task: JoinHandle<()>,
}
struct Inner {
    settings: Option<Settings>,
    state: Mutex<State>,
    subscription: AsyncMutex<Option<Subscription>>,
}

#[derive(Clone)]
pub struct Board {
    inner: Arc<Inner>,
}

fn invalid_config() -> io::Error {
    io::Error::other("configuracao local do console MQTT invalida")
}

fn private_profile(path: &Path) -> io::Result<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid_config())?;
    if !path.is_absolute() || !metadata.is_file() || metadata.len() > 65536 {
        return Err(invalid_config());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o777 != 0o600 {
            return Err(invalid_config());
        }
    }
    Ok(())
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn valid_topic(value: &str, filter: bool) -> bool {
    if !valid_text(value, 1024) {
        return false;
    }
    if !filter {
        return !value.contains(['+', '#']);
    }
    let levels = value.split('/').collect::<Vec<_>>();
    levels.iter().enumerate().all(|(index, level)| {
        (!level.contains('+') || *level == "+")
            && (!level.contains('#') || (*level == "#" && index + 1 == levels.len()))
    })
}

impl Board {
    pub fn disabled() -> Self {
        Self::new(None, "XMQR local".into())
    }

    fn new(settings: Option<Settings>, label: String) -> Self {
        let enabled = settings.is_some();
        Self {
            inner: Arc::new(Inner {
                settings,
                subscription: AsyncMutex::new(None),
                state: Mutex::new(State {
                    snapshot: Snapshot {
                        enabled,
                        label,
                        status: if enabled { "disconnected" } else { "disabled" },
                        error: None,
                        topic: None,
                        qos: None,
                        messages: Vec::new(),
                        received_total: 0,
                        published_total: 0,
                        dropped_total: 0,
                    },
                    messages: VecDeque::new(),
                    next_message: 1,
                    touched: Instant::now(),
                    closed: false,
                }),
            }),
        }
    }

    pub fn from_env() -> io::Result<Self> {
        let Some(sub_profile) = env::var_os("XMQR_ADMIN_MQTT_SUB_PROFILE") else {
            return Ok(Self::disabled());
        };
        let sub_profile = PathBuf::from(sub_profile);
        let pub_profile = env::var_os("XMQR_ADMIN_MQTT_PUB_PROFILE")
            .map_or_else(|| sub_profile.clone(), PathBuf::from);
        private_profile(&sub_profile)?;
        private_profile(&pub_profile)?;
        let binary = env::var_os("XMQR_ADMIN_MQTT_CLIENT_BIN")
            .map_or_else(|| PathBuf::from("/usr/bin/mqtt-client"), PathBuf::from);
        if !binary.is_absolute()
            || !std::fs::symlink_metadata(&binary).is_ok_and(|metadata| metadata.is_file())
        {
            return Err(invalid_config());
        }
        let label = env::var("XMQR_ADMIN_MQTT_LABEL").unwrap_or_else(|_| "XMQR local".into());
        if !valid_text(&label, 80) {
            return Err(invalid_config());
        }
        let mut identity = [0; 8];
        rand_core::OsRng
            .try_fill_bytes(&mut identity)
            .map_err(|_| invalid_config())?;
        Ok(Self::new(
            Some(Settings {
                binary,
                sub_profile,
                pub_profile,
                client_id: client_id(identity),
            }),
            label,
        ))
    }

    fn settings(&self) -> Result<&Settings, BoardError> {
        if self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closed
        {
            return Err(BoardError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "console_closed",
            ));
        }
        self.inner
            .settings
            .as_ref()
            .ok_or_else(|| BoardError::new(StatusCode::SERVICE_UNAVAILABLE, "console_disabled"))
    }

    fn snapshot_now(&self, renew: bool) -> Snapshot {
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if renew {
            state.touched = Instant::now();
        }
        let mut snapshot = state.snapshot.clone();
        snapshot.messages = state.messages.iter().cloned().collect();
        snapshot
    }

    pub fn snapshot(&self) -> std::future::Ready<Snapshot> {
        std::future::ready(self.snapshot_now(true))
    }

    pub fn availability(&self) -> Availability {
        let state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reason = match state.snapshot.status {
            "connected" => "connected",
            "connecting" => "connecting",
            "disconnected" => "disconnected",
            "disabled" => "disabled",
            _ => "error",
        };
        Availability {
            online: reason == "connected",
            reason,
        }
    }

    pub async fn connect(&self, request: ConnectRequest) -> Result<Snapshot, BoardError> {
        let settings = self.settings()?;
        if !valid_topic(&request.topic, true) || request.qos > 2 {
            return Err(BoardError::invalid());
        }
        let mut subscription = self
            .inner
            .subscription
            .try_lock()
            .map_err(|_| BoardError::new(StatusCode::TOO_MANY_REQUESTS, "busy"))?;
        self.settings()?;
        if subscription
            .as_ref()
            .is_some_and(|value| !value.task.is_finished())
        {
            return Err(BoardError::new(StatusCode::CONFLICT, "already_connected"));
        }
        if let Some(previous) = subscription.take() {
            let _ = previous.task.await;
        }
        let mut command = settings.command(false)?;
        command.args(["--topic", &request.topic, "--qos", &request.qos.to_string()]);
        let mut child = command.spawn().map_err(|_| BoardError::unavailable())?;
        let Some(stdout) = child.stdout.take() else {
            stop_child(&mut child).await;
            return Err(BoardError::unavailable());
        };
        let Some(stderr) = child.stderr.take() else {
            stop_child(&mut child).await;
            return Err(BoardError::unavailable());
        };
        {
            let mut state = self
                .inner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.snapshot.status = "connecting";
            state.snapshot.error = None;
            state.snapshot.topic = Some(request.topic);
            state.snapshot.qos = Some(request.qos);
            state.touched = Instant::now();
        }
        let (cancel, cancelled) = oneshot::channel();
        let weak = Arc::downgrade(&self.inner);
        let task = tokio::spawn(subscribe_loop(weak, child, stdout, stderr, cancelled));
        *subscription = Some(Subscription { cancel, task });
        Ok(self.snapshot_now(false))
    }

    pub async fn disconnect(&self) -> Result<Snapshot, BoardError> {
        let mut subscription = self.inner.subscription.lock().await;
        if let Some(subscription) = subscription.take() {
            let _ = subscription.cancel.send(());
            let _ = subscription.task.await;
        }
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.snapshot.status = if self.inner.settings.is_some() {
            "disconnected"
        } else {
            "disabled"
        };
        state.snapshot.error = None;
        state.snapshot.topic = None;
        state.snapshot.qos = None;
        drop(state);
        Ok(self.snapshot_now(false))
    }

    pub async fn publish(&self, request: PublishRequest) -> Result<Snapshot, BoardError> {
        let settings = self.settings()?;
        if !valid_topic(&request.topic, false)
            || request.qos > 2
            || request.payload.len() > MAX_PAYLOAD
        {
            return Err(BoardError::invalid());
        }
        let _operation = self
            .inner
            .subscription
            .try_lock()
            .map_err(|_| BoardError::new(StatusCode::TOO_MANY_REQUESTS, "busy"))?;
        self.settings()?;
        let mut command = settings.command(true)?;
        command
            .args([
                "--topic",
                &request.topic,
                "--qos",
                &request.qos.to_string(),
                "--retain",
                if request.retain { "true" } else { "false" },
                "--stdin",
            ])
            .stdin(Stdio::piped());
        let mut child = command.spawn().map_err(|_| BoardError::unavailable())?;
        let result = tokio::time::timeout(
            PUBLISH_DEADLINE,
            publish_child(&mut child, request.payload.as_bytes(), request.qos),
        )
        .await;
        if !matches!(result, Ok(Ok(()))) {
            stop_child(&mut child).await;
            return Err(BoardError::new(StatusCode::BAD_GATEWAY, "publish_failed"));
        }
        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.snapshot.published_total = state.snapshot.published_total.saturating_add(1);
        state.push(
            request.topic,
            request.payload,
            request.qos,
            request.retain,
            "published",
        );
        drop(state);
        Ok(self.snapshot_now(false))
    }

    pub async fn shutdown(&self) {
        self.inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closed = true;
        let _ = self.disconnect().await;
    }
}

impl Settings {
    fn command(&self, publish: bool) -> Result<Command, BoardError> {
        let profile = if publish {
            &self.pub_profile
        } else {
            &self.sub_profile
        };
        private_profile(profile).map_err(|_| BoardError::unavailable())?;
        let mut command = Command::new(&self.binary);
        for (key, _) in env::vars_os() {
            if key.to_string_lossy().starts_with("MQTT_") {
                command.env_remove(key);
            }
        }
        command
            .arg(if publish { "pub" } else { "sub" })
            .arg("--profile")
            .arg(profile)
            .args([
                "--output",
                "jsonl",
                "--client-id",
                &format!("{}-{}", self.client_id, if publish { "pub" } else { "sub" }),
                "--reconnect-attempts",
                "0",
                "--timeout-seconds",
                "8",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        Ok(command)
    }
}

impl State {
    fn push(
        &mut self,
        topic: String,
        payload: String,
        qos: u8,
        retain: bool,
        direction: &'static str,
    ) {
        if self.messages.len() == MAX_MESSAGES {
            self.messages.pop_front();
            self.snapshot.dropped_total = self.snapshot.dropped_total.saturating_add(1);
        }
        let received_at = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        )
        .unwrap_or(u64::MAX);
        self.messages.push_back(Message {
            id: self.next_message,
            received_at,
            topic,
            payload,
            qos,
            retain,
            direction,
        });
        self.next_message = self.next_message.saturating_add(1);
    }
}

async fn stop_child(child: &mut Child) {
    let _ = child.start_kill();
    let _ = tokio::time::timeout(Duration::from_secs(3), child.wait()).await;
}

async fn read_bounded(mut reader: impl tokio::io::AsyncRead + Unpin) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    (&mut reader)
        .take((MAX_LINE + 1) as u64)
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() > MAX_LINE {
        return Err(io::Error::other("limite de saida"));
    }
    Ok(bytes)
}

async fn publish_child(child: &mut Child, payload: &[u8], qos: u8) -> io::Result<()> {
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("stderr"))?;
    let ((), output, _, status) = tokio::try_join!(
        async move {
            input.write_all(payload).await?;
            input.shutdown().await?;
            drop(input);
            Ok::<(), io::Error>(())
        },
        read_bounded(stdout),
        read_bounded(stderr),
        child.wait()
    )?;
    if !status.success() || !publish_complete(&output, qos) {
        return Err(io::Error::other("publicacao recusada"));
    }
    Ok(())
}

fn publish_complete(output: &[u8], qos: u8) -> bool {
    output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .any(|line| {
            serde_json::from_slice::<serde_json::Value>(line).is_ok_and(|value| {
                value["event"] == "publish_complete"
                    && value["qos"] == qos
                    && value["confirmation"]
                        == if qos == 0 {
                            "sent"
                        } else {
                            "broker_protocol_ack"
                        }
            })
        })
}

fn append_lines(buffer: &mut Vec<u8>, bytes: &[u8]) -> Result<Vec<Vec<u8>>, ()> {
    let mut lines = Vec::new();
    for byte in bytes {
        if *byte == b'\n' {
            lines.push(std::mem::take(buffer));
        } else if buffer.len() < MAX_LINE {
            buffer.push(*byte);
        } else {
            return Err(());
        }
    }
    Ok(lines)
}

fn receive(state: &mut State, bytes: &[u8]) -> Result<(), ()> {
    #[derive(Deserialize)]
    struct Received {
        event: String,
        topic: String,
        payload_bytes: Vec<u8>,
        qos: u8,
        retain: bool,
    }
    let message: Received = serde_json::from_slice(bytes).map_err(|_| ())?;
    if message.event != "publish" || !valid_topic(&message.topic, false) || message.qos > 2 {
        return Err(());
    }
    state.snapshot.received_total = state.snapshot.received_total.saturating_add(1);
    if message.payload_bytes.len() > MAX_PAYLOAD {
        state.snapshot.dropped_total = state.snapshot.dropped_total.saturating_add(1);
        return Ok(());
    }
    let payload =
        String::from_utf8(message.payload_bytes).unwrap_or_else(|_| "[payload binario]".into());
    state.push(
        message.topic,
        payload,
        message.qos,
        message.retain,
        "received",
    );
    Ok(())
}

async fn subscribe_loop(
    inner: Weak<Inner>,
    mut child: Child,
    mut stdout: tokio::process::ChildStdout,
    mut stderr: tokio::process::ChildStderr,
    mut cancelled: oneshot::Receiver<()>,
) {
    let started = Instant::now();
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let mut output_chunk = [0; 2048];
    let mut error_chunk = [0; 2048];
    let mut stdout_open = true;
    let mut stderr_open = true;
    let mut failure = None;
    loop {
        tokio::select! {
            _ = &mut cancelled => break,
            _ = tick.tick() => {
                let Some(inner) = inner.upgrade() else { break; };
                let state = inner.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.closed { break; }
                if state.touched.elapsed() >= LEASE { failure = Some("lease_expired"); break; }
                if state.snapshot.status == "connecting" && started.elapsed() >= CONNECT_DEADLINE { failure = Some("connect_timeout"); break; }
            },
            result = child.wait() => { let _ = result; failure = Some("client_disconnected"); break; },
            result = stdout.read(&mut output_chunk), if stdout_open => {
                let Ok(length) = result else { failure = Some("client_output_invalid"); break; };
                if length == 0 { stdout_open = false; continue; }
                let Ok(lines) = append_lines(&mut output, &output_chunk[..length]) else { failure = Some("client_output_limit"); break; };
                let Some(inner) = inner.upgrade() else { break; };
                let mut state = inner.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                for line in lines { if receive(&mut state, &line).is_err() { failure = Some("client_output_invalid"); break; } }
                if failure.is_some() { break; }
            },
            result = stderr.read(&mut error_chunk), if stderr_open => {
                let Ok(length) = result else { failure = Some("client_output_invalid"); break; };
                if length == 0 { stderr_open = false; continue; }
                let Ok(lines) = append_lines(&mut errors, &error_chunk[..length]) else { failure = Some("client_output_limit"); break; };
                if lines.iter().any(|line| line.starts_with(b"Assinatura ativa")) {
                    let Some(inner) = inner.upgrade() else { break; };
                    let mut state = inner.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    state.snapshot.status = "connected";
                    state.snapshot.error = None;
                }
            },
        }
    }
    stop_child(&mut child).await;
    if let Some(inner) = inner.upgrade() {
        let mut state = inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.snapshot.status = if failure.is_some() {
            "error"
        } else {
            "disconnected"
        };
        state.snapshot.error = failure;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn availability_tracks_subscription_state_without_exposing_messages_or_renewing_lease() {
        let board = Board::disabled();
        let touched = Instant::now().checked_sub(LEASE).unwrap();
        {
            let mut state = board.inner.state.lock().unwrap();
            state.touched = touched;
            state.snapshot.topic = Some("private/topic".into());
            state.push(
                "private/topic".into(),
                "private-payload".into(),
                1,
                false,
                "received",
            );
        }
        for (status, online, reason) in [
            ("disabled", false, "disabled"),
            ("disconnected", false, "disconnected"),
            ("connecting", false, "connecting"),
            ("connected", true, "connected"),
            ("error", false, "error"),
            ("unexpected", false, "error"),
        ] {
            board.inner.state.lock().unwrap().snapshot.status = status;
            let aggregate = serde_json::to_value(board.availability()).unwrap();
            assert_eq!(
                aggregate,
                serde_json::json!({"online":online,"reason":reason})
            );
            let state = board.inner.state.lock().unwrap();
            assert_eq!(state.touched, touched);
            assert_eq!(state.messages.len(), 1);
            assert_eq!(state.messages.front().unwrap().payload, "private-payload");
        }
    }

    #[test]
    fn generated_client_ids_fit_mqtt311_with_role_suffixes() {
        for bytes in [[0; 8], [255; 8], [1, 2, 3, 4, 5, 6, 7, 8]] {
            let base = client_id(bytes);
            assert_eq!(base.len(), 19);
            for suffix in ["sub", "pub"] {
                assert_eq!(format!("{base}-{suffix}").len(), 23);
            }
            assert_eq!(base, client_id(bytes));
        }
        assert_ne!(client_id([0; 8]), client_id([1; 8]));
    }

    #[test]
    fn filters_and_topics_are_bounded_and_distinct() {
        for topic in ["", "x/#/z", "x+a", "x/##", "x/+a", "a\0b"] {
            assert!(!valid_topic(topic, true));
        }
        for topic in ["#", "x/+", "x/#", "/a//b"] {
            assert!(valid_topic(topic, true));
        }
        assert!(!valid_topic("a/+", false));
        assert!(!valid_topic(&"a".repeat(1025), true));
        assert!(valid_topic("a/b", false));
    }

    #[test]
    fn requests_reject_unknown_paths_and_wrong_types() {
        assert!(
            serde_json::from_str::<ConnectRequest>(r#"{"topic":"a","qos":1,"profile":"/tmp/x"}"#)
                .is_err()
        );
        assert!(
            serde_json::from_str::<PublishRequest>(
                r#"{"topic":"a","payload":"x","qos":1,"retain":false,"host":"evil"}"#
            )
            .is_err()
        );
        assert!(serde_json::from_str::<ConnectRequest>(r#"{"topic":"a","qos":-1}"#).is_err());
    }

    #[tokio::test]
    async fn disabled_is_honest_and_no_child_is_created() {
        let board = Board::disabled();
        assert!(!board.snapshot().await.enabled);
        assert!(matches!(
            board
                .connect(ConnectRequest {
                    topic: "a".into(),
                    qos: 0
                })
                .await,
            Err(BoardError {
                code: "console_disabled",
                ..
            })
        ));
        board.shutdown().await;
    }

    #[tokio::test]
    async fn invalid_payload_qos_and_filter_never_spawn_a_process() {
        let board = Board::new(
            Some(Settings {
                binary: PathBuf::from("/not-a-real-client"),
                sub_profile: PathBuf::from("/not-a-real-profile"),
                pub_profile: PathBuf::from("/not-a-real-profile"),
                client_id: "test".into(),
            }),
            "test".into(),
        );
        for (payload, qos, topic) in [
            ("x".repeat(MAX_PAYLOAD + 1), 0, "a"),
            (String::new(), 3, "a"),
            (String::new(), 1, "a/+"),
        ] {
            assert!(matches!(
                board
                    .publish(PublishRequest {
                        topic: topic.into(),
                        payload,
                        qos,
                        retain: false
                    })
                    .await,
                Err(BoardError {
                    code: "invalid_request",
                    ..
                })
            ));
        }
        assert!(matches!(
            board
                .connect(ConnectRequest {
                    topic: "a/#/b".into(),
                    qos: 0
                })
                .await,
            Err(BoardError {
                code: "invalid_request",
                ..
            })
        ));
        board.shutdown().await;
    }

    #[test]
    fn buffer_counts_only_received_and_confirmed_messages() {
        let board = Board::disabled();
        let mut state = board.inner.state.lock().unwrap();
        for _ in 0..201 {
            receive(&mut state, br#"{"event":"publish","topic":"a","payload_bytes":[60,115,99,114,105,112,116,62],"qos":1,"retain":false}"#).unwrap();
        }
        assert_eq!(state.messages.len(), 200);
        assert_eq!(state.snapshot.received_total, 201);
        assert_eq!(state.snapshot.published_total, 0);
        assert_eq!(state.snapshot.dropped_total, 1);
        assert_eq!(state.messages.front().unwrap().payload, "<script>");
        receive(
            &mut state,
            br#"{"event":"publish","topic":"b","payload_bytes":[255,0],"qos":0,"retain":true}"#,
        )
        .unwrap();
        assert_eq!(state.messages.back().unwrap().payload, "[payload binario]");
        assert!(receive(&mut state, b"segredo remoto").is_err());
        let oversized = serde_json::to_vec(&serde_json::json!({"event":"publish", "topic":"a", "payload_bytes":vec![0; MAX_PAYLOAD + 1], "qos":0, "retain":false})).unwrap();
        let dropped = state.snapshot.dropped_total;
        let received = state.snapshot.received_total;
        receive(&mut state, &oversized).unwrap();
        assert_eq!(state.snapshot.dropped_total, dropped + 1);
        assert_eq!(state.snapshot.received_total, received + 1);
    }

    #[test]
    fn lines_preserve_chunks_and_fail_before_unbounded_allocation() {
        let mut pending = Vec::new();
        assert_eq!(
            append_lines(&mut pending, b"abc").unwrap(),
            [] as [Vec<u8>; 0]
        );
        assert_eq!(
            append_lines(&mut pending, b"def\n").unwrap(),
            vec![b"abcdef".to_vec()]
        );
        assert!(append_lines(&mut pending, &vec![b'x'; MAX_LINE + 1]).is_err());
        assert_eq!(pending.len(), MAX_LINE);
        assert!(publish_complete(
            br#"{"event":"publish_complete","qos":1,"confirmation":"broker_protocol_ack"}"#,
            1
        ));
        assert!(!publish_complete(
            br#"{"event":"publish_complete","qos":0,"confirmation":"sent"}"#,
            1
        ));
        assert!(!publish_complete(b"status desconhecido", 0));
    }

    #[cfg(unix)]
    #[test]
    fn profiles_refuse_public_permissions_symlinks_and_oversize() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("perfil.toml");
        std::fs::write(&path, "version=1").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(private_profile(&path).is_ok());
        let link = directory.path().join("link");
        symlink(&path, &link).unwrap();
        assert!(private_profile(&link).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(private_profile(&path).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(&path, vec![0; 65537]).unwrap();
        assert!(private_profile(&path).is_err());
    }
}
