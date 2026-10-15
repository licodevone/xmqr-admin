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
        fs::{self, File, OpenOptions},
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
        #[serde(skip_serializing_if = "Option::is_none")]
        topic: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_id: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        archive_sha256: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        archive_count: Option<u16>,
    }

    impl<'a> Request<'a> {
        fn base(id: &'a str, operation: &'a str, token: &'a str) -> Self {
            Self {
                version: 1,
                id,
                operation,
                token,
                expected_generation: None,
                bundle_sha256: None,
                target_request_id: None,
                cursor: None,
                limit: None,
                topic: None,
                client_id: None,
                archive_sha256: None,
                archive_count: None,
            }
        }
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
        if operation == "audit-rotate" {
            return rotate(args, socket, &token);
        }
        let epoch = if matches!(
            operation,
            "reload" | "retained-delete" | "session-disconnect"
        ) {
            fetch_epoch(socket, &token)?
        } else {
            0
        };
        let request_id = match (operation, args.len()) {
            ("status", 6)
            | ("receipt" | "retained-delete" | "session-disconnect", 8)
            | ("reload", 10)
            | ("sessions" | "retained" | "audit", 6 | 8 | 10) => new_id(epoch),
            ("reload", 12) | ("retained-delete" | "session-disconnect", 10)
                if text(args.len() - 2)? == "--request-id" =>
            {
                let id = text(args.len() - 1)?;
                if !valid_id(id) {
                    return Err(invalid("request ID deve ter 32 digitos hexadecimais"));
                }
                id.to_owned()
            }
            _ => return Err(invalid("argumentos de channel invalidos")),
        };
        let mut request = Request::base(
            &request_id,
            match operation {
                "sessions" => "sessions_list",
                "retained" => "retained_list",
                "audit" => "audit_list",
                "retained-delete" => "retained_delete",
                "session-disconnect" => "session_disconnect",
                other => other,
            },
            token.as_str(),
        );
        let digest;
        match operation {
            "status" => {}
            "receipt" if text(6)? == "--id" && valid_id(text(7)?) => {
                request.target_request_id = Some(text(7)?);
            }
            "sessions" | "retained" | "audit" => {
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
                if operation == "audit"
                    && request.cursor.is_some_and(|value| {
                        !value.parse::<u16>().is_ok_and(|number| number <= 128)
                    })
                {
                    return Err(invalid("cursor de auditoria invalido"));
                }
            }
            "retained-delete" if text(6)? == "--topic" => {
                let topic = text(7)?;
                if topic.is_empty() || topic.len() > 1024 || topic.contains(['\0', '+', '#']) {
                    return Err(invalid("topico retained invalido"));
                }
                request.topic = Some(topic);
                println!("{}", json!({"request_id":request_id,"state":"submitted"}));
                io::stdout().flush()?;
            }
            "session-disconnect" if text(6)? == "--client-id" => {
                let target = text(7)?;
                if target.is_empty() || target.len() > 256 || target.contains('\0') {
                    return Err(invalid("client ID invalido"));
                }
                request.client_id = Some(target);
                println!("{}", json!({"request_id":request_id,"state":"submitted"}));
                io::stdout().flush()?;
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
                if matches!(
                    operation,
                    "reload" | "retained-delete" | "session-disconnect"
                ) {
                    eprintln!(
                        "Resultado desconhecido: consulte channel receipt --id {request_id} antes de repetir."
                    );
                }
                return Err(error);
            }
        };
        println!("{response}");
        if response["ok"] == false {
            if matches!(
                operation,
                "reload" | "retained-delete" | "session-disconnect"
            ) && response["error"] == "result_unknown"
            {
                eprintln!(
                    "Resultado desconhecido: consulte channel receipt --id {request_id} antes de repetir."
                );
            }
            return Err(io::Error::other("broker recusou operacao administrativa"));
        }
        Ok(())
    }

    fn new_id(epoch: u32) -> String {
        let mut bytes = [0_u8; 16];
        OsRng.fill_bytes(&mut bytes);
        let mut id = String::with_capacity(32);
        for byte in bytes {
            write!(&mut id, "{byte:02x}").expect("String formatting cannot fail");
        }
        id.replace_range(..8, &format!("{epoch:08x}"));
        id
    }

    fn fetch_epoch(socket: &Path, token: &crate::token::Token) -> io::Result<u32> {
        let id = new_id(0);
        let response = send(socket, &Request::base(&id, "status", token.as_str()))?;
        if response["ok"] != true {
            return Err(io::Error::other("status administrativo indisponivel"));
        }
        response["admin_epoch"]
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| invalid("broker nao informa epoca administrativa"))
    }

    fn rotate(args: &[OsString], socket: &Path, token: &crate::token::Token) -> io::Result<()> {
        if args.len() != 8 || args[6] != "--archive" {
            return Err(invalid(
                "uso: channel audit-rotate --socket CAMINHO --token-file ARQUIVO --archive NOVO_ARQUIVO",
            ));
        }
        let archive = Path::new(&args[7]);
        let epoch = fetch_epoch(socket, token)?;
        let mut items: Vec<Value> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let id = new_id(0);
            let mut request = Request::base(&id, "audit_list", token.as_str());
            request.cursor = cursor.as_deref();
            request.limit = Some(16);
            let response = send(socket, &request)?;
            if response["ok"] != true {
                return Err(io::Error::other("auditoria indisponivel para exportacao"));
            }
            let page = response["items"]
                .as_array()
                .ok_or_else(|| invalid("pagina de auditoria invalida"))?;
            items.extend(page.iter().cloned());
            if items.len() > 128 {
                return Err(invalid("auditoria excede limite"));
            }
            let next = response["next_cursor"].as_str().map(ToOwned::to_owned);
            if next.is_none() {
                break;
            }
            if next == cursor || page.is_empty() {
                return Err(invalid("cursor de auditoria nao avanca"));
            }
            cursor = next;
        }
        if items.is_empty() {
            return Err(invalid("auditoria vazia"));
        }
        let canonical = serde_json::to_vec(&items)?;
        let digest = format!("{:x}", Sha256::digest(&canonical));
        let contents = serde_json::to_vec_pretty(&json!({
            "version":1,"admin_epoch":epoch,"sha256":digest,"receipts":items,
        }))?;
        write_archive(archive, &contents)?;
        let id = new_id(epoch);
        println!(
            "{}",
            json!({"request_id":id,"state":"submitted",
            "archive_sha256":digest,"archive_count":items.len()})
        );
        io::stdout().flush()?;
        let mut request = Request::base(&id, "audit_rotate", token.as_str());
        request.archive_sha256 = Some(&digest);
        request.archive_count =
            Some(u16::try_from(items.len()).map_err(|_| invalid("auditoria excessiva"))?);
        let response = match send(socket, &request) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "Resultado desconhecido: consulte channel receipt --id {id} antes de repetir."
                );
                return Err(error);
            }
        };
        println!("{response}");
        if response["ok"] != true {
            return Err(io::Error::other(
                "rotacao de auditoria recusada; arquivo exportado preservado",
            ));
        }
        Ok(())
    }

    fn write_archive(path: &Path, contents: &[u8]) -> io::Result<()> {
        let parent = path
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .ok_or_else(|| invalid("diretorio de arquivo explicito necessario"))?;
        let metadata = fs::symlink_metadata(parent)?;
        if !path.is_absolute() || !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(invalid("diretorio de arquivo deve ser privado"));
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        File::open(parent)?.sync_all()
    }

    pub fn verify_archive(path: &Path) -> io::Result<()> {
        if !path.is_absolute() {
            return Err(invalid("arquivo de auditoria exige caminho absoluto"));
        }
        let parent = path
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .ok_or_else(|| invalid("diretorio de arquivo explicito necessario"))?;
        let dir = fs::symlink_metadata(parent)?;
        let before = fs::symlink_metadata(path)?;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)?;
        let after = file.metadata()?;
        if !dir.is_dir()
            || dir.permissions().mode() & 0o077 != 0
            || !before.is_file()
            || !after.is_file()
            || before.dev() != after.dev()
            || before.ino() != after.ino()
            || after.permissions().mode() & 0o7777 != 0o600
            || after.len() > 1024 * 1024
        {
            return Err(invalid("arquivo de auditoria invalido"));
        }
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 1024 * 1024 {
            return Err(invalid("arquivo de auditoria excessivo"));
        }
        let data: Value = serde_json::from_slice(&bytes)?;
        let receipts = data["receipts"]
            .as_array()
            .ok_or_else(|| invalid("recibos invalidos"))?;
        if receipts.is_empty() || receipts.len() > 128 || !data["admin_epoch"].is_u64() {
            return Err(invalid("metadados de auditoria invalidos"));
        }
        let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(receipts)?));
        if data["version"] != 1 || data["sha256"] != digest {
            return Err(invalid("checksum de auditoria invalido"));
        }
        println!(
            "{}",
            json!({"ok":true,"sha256":digest,"count":receipts.len(),
            "admin_epoch":data["admin_epoch"]})
        );
        Ok(())
    }

    pub fn diagnose(args: &[OsString]) -> io::Result<()> {
        if args.len() != 11
            || args[1] != "--state-dir"
            || args[3] != "--bundle"
            || args[5] != "--backup"
            || args[7] != "--socket"
            || args[9] != "--token-file"
        {
            return Err(invalid(
                "uso: diagnose --state-dir DIR --bundle ARQUIVO --backup ARQUIVO --socket CAMINHO --token-file ARQUIVO",
            ));
        }
        let path = |index| -> io::Result<&Path> {
            args.get(index)
                .map(Path::new)
                .ok_or_else(|| invalid("caminho obrigatorio"))
        };
        let state = path(2)?;
        let bundle = path(4)?;
        let backup = path(6)?;
        let socket = path(8)?;
        let token_file = path(10)?;
        let private_state = fs::symlink_metadata(state).is_ok_and(|metadata| {
            metadata.is_dir() && metadata.permissions().mode().trailing_zeros() >= 6
        });
        let free_bytes = if private_state {
            fs2::available_space(state).ok()
        } else {
            None
        };
        let bundle_valid = crate::bundle::validate(bundle).is_ok();
        let backup_valid = mqtt_broker::persistence::backup::verify(backup).is_ok();
        let status = crate::token::read(token_file).ok().and_then(|token| {
            send(socket, &Request::base(&new_id(0), "status", token.as_str())).ok()
        });
        let broker_ready = status
            .as_ref()
            .is_some_and(|value| value["ok"] == true && value["ready"] == true);
        let broker_version = status
            .as_ref()
            .and_then(|value| value["broker_version"].as_str());
        let adequate_space = free_bytes.is_some_and(|bytes| bytes >= 16 * 1024 * 1024);
        let ok = private_state
            && bundle_valid
            && backup_valid
            && broker_ready
            && broker_version.is_some()
            && adequate_space;
        println!(
            "{}",
            json!({
                "ok":ok,"state_directory_private":private_state,
                "bundle_valid":bundle_valid,"backup_valid":backup_valid,
                "broker_ready":broker_ready,"adequate_space":adequate_space,
                "free_bytes":free_bytes,"minimum_free_bytes":16 * 1024 * 1024,
                "broker_version":broker_version,"admin_version":env!("CARGO_PKG_VERSION")
            })
        );
        if ok {
            Ok(())
        } else {
            Err(io::Error::other(
                "diagnostico encontrou falha; veja os campos booleanos",
            ))
        }
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
pub use linux::diagnose;
#[cfg(target_os = "linux")]
pub use linux::{run, verify_archive};

#[cfg(not(target_os = "linux"))]
pub fn run(_: &[OsString]) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "canal requer Linux/WSL",
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn verify_archive(_: &std::path::Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "auditoria requer Linux/WSL",
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn diagnose(_: &[OsString]) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "diagnostico requer Linux/WSL",
    ))
}

#[cfg(target_os = "linux")]
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
