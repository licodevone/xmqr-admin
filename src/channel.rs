// SPDX-License-Identifier: MIT
//! Bounded CLI client for the broker's local administrative socket.

use std::{ffi::OsString, io};

#[cfg(target_os = "linux")]
mod linux {
    use super::{OsString, invalid, io};
    use rand_core::{OsRng, RngCore};
    use serde::Serialize;
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        fmt::Write as _,
        fs::{self, OpenOptions},
        io::{Read, Write},
        os::unix::{
            fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
            net::UnixStream,
        },
        path::Path,
        time::{Duration, Instant},
    };
    use zeroize::Zeroizing;

    const LIMIT: usize = 16 * 1024;
    const DEADLINE: Duration = Duration::from_secs(3);
    const BUNDLE_LIMIT: u64 = 1024 * 1024;

    #[derive(Serialize)]
    struct Request<'a> {
        version: u8,
        #[serde(rename = "request_id")]
        id: &'a str,
        operation: &'a str,
        token: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_generation: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bundle_sha256: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_request_id: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cursor: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u8>,
    }

    #[expect(
        clippy::too_many_lines,
        reason = "single bounded CLI dispatch and recovery output"
    )]
    pub fn run(args: &[OsString]) -> io::Result<()> {
        let text = |index: usize| -> io::Result<&str> {
            args.get(index)
                .and_then(|v| v.to_str())
                .ok_or_else(|| invalid("argumento UTF-8 obrigatorio"))
        };
        if args.len() < 6 || text(2)? != "--socket" || text(4)? != "--token-file" {
            return Err(invalid(
                "uso: channel status|receipt|reload --socket CAMINHO --token-file ARQUIVO [--id ID | --file BUNDLE --expected-generation N [--request-id ID]]",
            ));
        }
        let socket = Path::new(text(3)?);
        let token = crate::token::read(Path::new(text(5)?))?;
        let operation = text(1)?;
        let request_id = match (operation, args.len()) {
            ("status", 6)
            | ("receipt", 8)
            | ("reload", 10)
            | ("sessions" | "retained", 6 | 8 | 10) => new_id(),
            ("reload", 12) if text(10)? == "--request-id" => {
                let id = text(11)?;
                if !valid_id(id) {
                    return Err(invalid("request ID deve ter 32 digitos hexadecimais"));
                }
                id.to_owned()
            }
            _ => return Err(invalid("argumentos de channel invalidos")),
        };
        let mut request = Request {
            version: 1,
            id: &request_id,
            operation: match operation {
                "sessions" => "sessions_list",
                "retained" => "retained_list",
                other => other,
            },
            token: token.as_str(),
            expected_generation: None,
            bundle_sha256: None,
            target_request_id: None,
            cursor: None,
            limit: None,
        };
        let digest;
        match operation {
            "status" => {}
            "receipt" if text(6)? == "--id" && valid_id(text(7)?) => {
                request.target_request_id = Some(text(7)?);
            }
            "sessions" | "retained" => {
                let max = if operation == "sessions" { 16 } else { 8 };
                if args.len() >= 8 {
                    match text(6)? {
                        "--limit" => {
                            let size = text(7)?
                                .parse::<u8>()
                                .map_err(|_| invalid("limite invalido"))?;
                            if size == 0 || size > max {
                                return Err(invalid("limite invalido"));
                            }
                            request.limit = Some(size);
                        }
                        "--cursor" if args.len() == 8 => request.cursor = Some(text(7)?),
                        _ => return Err(invalid("opcoes de inventario invalidas")),
                    }
                }
                if args.len() == 10 {
                    if text(8)? != "--cursor" {
                        return Err(invalid("cursor invalido"));
                    }
                    request.cursor = Some(text(9)?);
                }
                if request.cursor.is_some_and(|value| value.len() > 1024) {
                    return Err(invalid("cursor excede limite"));
                }
            }
            "reload" if text(6)? == "--file" && text(8)? == "--expected-generation" => {
                let generation = text(9)?
                    .parse::<u64>()
                    .map_err(|_| invalid("geracao invalida"))?;
                if generation == 0 {
                    return Err(invalid("geracao invalida"));
                }
                let file = Path::new(text(7)?);
                crate::bundle::validate(file)?;
                digest = private_bundle_sha256(file)?;
                request.expected_generation = Some(generation);
                request.bundle_sha256 = Some(&digest);
                println!("{}", json!({"request_id":request_id,"state":"submitted"}));
                io::stdout().flush()?;
            }
            _ => return Err(invalid("argumentos de channel invalidos")),
        }
        let response = match send(socket, &request) {
            Ok(response) => response,
            Err(error) => {
                if operation == "reload" {
                    eprintln!(
                        "Resultado desconhecido: consulte channel receipt --id {request_id} antes de repetir."
                    );
                }
                return Err(error);
            }
        };
        println!("{response}");
        if response["ok"] == false {
            if operation == "reload" && response["error"] == "result_unknown" {
                eprintln!(
                    "Resultado desconhecido: consulte channel receipt --id {request_id} antes de repetir."
                );
            }
            return Err(io::Error::other("broker recusou operacao administrativa"));
        }
        Ok(())
    }

    fn new_id() -> String {
        let mut bytes = [0_u8; 16];
        OsRng.fill_bytes(&mut bytes);
        let mut id = String::with_capacity(32);
        for byte in bytes {
            write!(&mut id, "{byte:02x}").expect("String formatting cannot fail");
        }
        id
    }

    fn valid_id(id: &str) -> bool {
        id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit())
    }

    fn private_bundle_sha256(path: &Path) -> io::Result<String> {
        let before = fs::symlink_metadata(path)?;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)?;
        let after = file.metadata()?;
        if !before.is_file()
            || !after.is_file()
            || before.dev() != after.dev()
            || before.ino() != after.ino()
            || after.permissions().mode() & 0o7777 != 0o600
            || after.len() > BUNDLE_LIMIT
        {
            return Err(invalid("bundle privado invalido"));
        }
        let mut bytes = Zeroizing::new(Vec::new());
        file.take(BUNDLE_LIMIT + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > BUNDLE_LIMIT {
            return Err(invalid("bundle excede limite"));
        }
        Ok(format!("{:x}", Sha256::digest(&*bytes)))
    }

    fn send(socket: &Path, request: &Request<'_>) -> io::Result<Value> {
        let parent = socket
            .parent()
            .ok_or_else(|| invalid("diretorio do socket ausente"))?;
        let dir = fs::symlink_metadata(parent)?;
        let endpoint = fs::symlink_metadata(socket)?;
        if !dir.is_dir()
            || dir.permissions().mode() & 0o077 != 0
            || !endpoint.file_type().is_socket()
            || endpoint.permissions().mode() & 0o777 != 0o600
            || endpoint.uid() != dir.uid()
        {
            return Err(invalid("socket administrativo nao privado"));
        }
        let mut stream = UnixStream::connect(socket)?;
        let deadline = Instant::now() + DEADLINE;
        let mut bytes = Zeroizing::new(serde_json::to_vec(request)?);
        bytes.push(b'\n');
        if bytes.len() > LIMIT {
            return Err(invalid("request administrativo excede limite"));
        }
        stream.set_write_timeout(Some(DEADLINE))?;
        stream.write_all(&bytes)?;
        let mut response = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "resultado desconhecido; consultar receipt",
                ));
            }
            stream.set_read_timeout(Some(remaining))?;
            let count = stream.read(&mut chunk)?;
            if count == 0 || response.len() + count > LIMIT {
                return Err(invalid(
                    "resposta administrativa ausente ou excessiva; consultar receipt",
                ));
            }
            response.extend_from_slice(&chunk[..count]);
            if response.contains(&b'\n') {
                break;
            }
        }
        let newline = response
            .iter()
            .position(|b| *b == b'\n')
            .expect("checked above");
        if response[newline + 1..]
            .iter()
            .any(|b| !b.is_ascii_whitespace())
        {
            return Err(invalid("resposta administrativa contem dados extras"));
        }
        let value: Value = serde_json::from_slice(&response[..newline])?;
        if value["request_id"] != request.id || !value["ok"].is_boolean() {
            return Err(invalid("resposta administrativa sem correlacao"));
        }
        Ok(value)
    }
}

#[cfg(target_os = "linux")]
pub use linux::run;

#[cfg(not(target_os = "linux"))]
pub fn run(_: &[OsString]) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "canal requer Linux/WSL",
    ))
}

#[cfg(target_os = "linux")]
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
