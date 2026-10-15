//! Offline, atomic edits to the broker's private bundle. The core validates each candidate.
use fs2::FileExt;
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};
use toml::Value;

const LIMIT: u64 = 1_048_576;

#[expect(
    clippy::too_many_lines,
    reason = "CLI validation and mutation share one bounded transaction"
)]
pub fn run(args: &[OsString]) -> io::Result<()> {
    let a: Vec<&str> = args
        .iter()
        .map(|x| x.to_str().ok_or_else(|| invalid("argumento nao UTF-8")))
        .collect::<io::Result<_>>()?;
    if a.len() < 5 || a[0] != "bundle" || a[3] != "--file" {
        return Err(invalid("uso: bundle TIPO ACAO --file ARQUIVO [opcoes]"));
    }
    let (kind, action, path) = (a[1], a[2], Path::new(a[4]));
    if !matches!(kind, "role" | "group" | "user") {
        return Err(invalid("tipo de bundle desconhecido"));
    }
    let mut name = None;
    let mut publish = Vec::new();
    let mut subscribe = Vec::new();
    let mut roles = Vec::new();
    let mut groups = Vec::new();
    let (pairs, remainder) = a[5..].as_chunks::<2>();
    for pair in pairs {
        match pair[0] {
            "--name" | "--username" if name.replace(pair[1]).is_none() => {}
            "--publish" => publish.push(pair[1]),
            "--subscribe" => subscribe.push(pair[1]),
            "--role" => roles.push(pair[1]),
            "--group" => groups.push(pair[1]),
            _ => return Err(invalid("opcao desconhecida ou duplicada")),
        }
    }
    if !remainder.is_empty() {
        return Err(invalid("opcao sem valor"));
    }
    let has_acl = !publish.is_empty() || !subscribe.is_empty();
    let has_membership = !roles.is_empty() || !groups.is_empty();
    if has_acl
        && !matches!(
            (kind, action),
            ("role", "create" | "update") | ("user", "update-acl")
        )
        || has_membership
            && !matches!(
                (kind, action),
                ("group", "create" | "update") | ("user", "set-membership")
            )
        || kind == "group" && !groups.is_empty()
    {
        return Err(invalid("opcoes incompatíveis"));
    }
    let name = if action == "list" {
        if name.is_some() {
            return Err(invalid("list nao recebe nome"));
        }
        None
    } else {
        let value = name.ok_or_else(|| invalid("--name/--username obrigatorio"))?;
        if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
            return Err(invalid("nome invalido"));
        }
        Some(value)
    };
    let lock_path = path.with_file_name(format!(
        ".{}.lock",
        path.file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| invalid("nome de arquivo invalido"))?
    ));
    private_parent(path)?;
    let lock = open_lock(&lock_path)?;
    FileExt::try_lock_exclusive(&lock)?;
    let result = (|| {
        let (mut bundle, original) = read(path)?;
        crate::bundle::validate(path)?;
        let field = match kind {
            "role" => "roles",
            "group" => "groups",
            _ => "users",
        };
        let key = if kind == "user" { "username" } else { "name" };
        let table = bundle
            .as_table_mut()
            .ok_or_else(|| invalid("bundle invalido"))?;
        if !table.contains_key(field) {
            table.insert(field.into(), Value::Array(Vec::new()));
        }
        let entries = table
            .get_mut(field)
            .and_then(Value::as_array_mut)
            .ok_or_else(|| invalid("lista de bundle invalida"))?;
        if action == "list" {
            for item in entries {
                let label = item
                    .get(key)
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("nome de bundle invalido"))?;
                if kind == "user" {
                    println!(
                        "{label} disabled={}",
                        item.get("disabled")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                    );
                } else if kind == "role" {
                    println!(
                        "{label} publish={:?} subscribe={:?}",
                        item.get("publish"),
                        item.get("subscribe")
                    );
                } else {
                    println!("{label} roles={:?}", item.get("roles"));
                }
            }
            return Ok(());
        }
        let name = name.ok_or_else(|| invalid("nome obrigatorio"))?;
        let index = entries
            .iter()
            .position(|x| x.get(key).and_then(Value::as_str) == Some(name));
        match (kind, action, index) {
            ("role" | "group", "create", None) => {
                let mut record = toml::map::Map::new();
                record.insert("name".into(), Value::String(name.into()));
                put_lists(&mut record, kind, &publish, &subscribe, &roles);
                entries.push(Value::Table(record));
            }
            ("role" | "group", "update", Some(i)) => {
                let record = entries[i]
                    .as_table_mut()
                    .ok_or_else(|| invalid("registro invalido"))?;
                put_lists(record, kind, &publish, &subscribe, &roles);
            }
            ("role" | "group", "delete", Some(i)) => {
                entries.remove(i);
            }
            ("user", "enable" | "disable", Some(i)) => {
                let record = entries[i]
                    .as_table_mut()
                    .ok_or_else(|| invalid("registro invalido"))?;
                record.insert("disabled".into(), Value::Boolean(action == "disable"));
            }
            ("user", "update-acl", Some(i)) => {
                let record = entries[i]
                    .as_table_mut()
                    .ok_or_else(|| invalid("registro invalido"))?;
                record.insert("publish".into(), strings(&publish));
                record.insert("subscribe".into(), strings(&subscribe));
            }
            ("user", "set-membership", Some(i)) => {
                let record = entries[i]
                    .as_table_mut()
                    .ok_or_else(|| invalid("registro invalido"))?;
                record.insert("roles".into(), strings(&roles));
                record.insert("groups".into(), strings(&groups));
            }
            (_, "create", Some(_)) => return Err(invalid("registro ja existe")),
            (
                _,
                "update" | "delete" | "enable" | "disable" | "update-acl" | "set-membership",
                None,
            ) => {
                return Err(invalid("registro inexistente"));
            }
            _ => return Err(invalid("acao invalida")),
        }
        write_candidate(path, &bundle, &original)?;
        println!("Bundle atualizado offline. Aplique ao broker separadamente.");
        Ok(())
    })();
    let unlocked = FileExt::unlock(&lock);
    if result.is_ok() {
        unlocked?;
    }
    result
}

fn put_lists(
    record: &mut toml::map::Map<String, Value>,
    kind: &str,
    publish: &[&str],
    subscribe: &[&str],
    roles: &[&str],
) {
    if kind == "role" {
        record.insert("publish".into(), strings(publish));
        record.insert("subscribe".into(), strings(subscribe));
    } else {
        record.insert("roles".into(), strings(roles));
    }
}

fn strings(items: &[&str]) -> Value {
    Value::Array(items.iter().map(|x| Value::String((*x).into())).collect())
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(target_os = "linux")]
fn private(meta: &fs::Metadata) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if !meta.is_file() || meta.permissions().mode() & 0o777 != 0o600 {
        return Err(invalid("arquivo regular exige modo 0600"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn private_parent(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let parent = path
        .parent()
        .filter(|x| !x.as_os_str().is_empty())
        .ok_or_else(|| invalid("diretorio explicito necessario"))?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o022 != 0 {
        return Err(invalid("diretorio nao pode ser gravavel por grupo/outros"));
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn private_parent(_: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "edicao requer Linux/WSL",
    ))
}

#[cfg(target_os = "linux")]
fn open_lock(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    private(&file.metadata()?)?;
    Ok(file)
}

#[cfg(not(target_os = "linux"))]
fn open_lock(_: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "edicao requer Linux/WSL",
    ))
}

#[cfg(target_os = "linux")]
fn read(path: &Path) -> io::Result<(Value, fs::Metadata)> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let before = fs::symlink_metadata(path)?;
    private(&before)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let after = file.metadata()?;
    private(&after)?;
    if before.dev() != after.dev() || before.ino() != after.ino() {
        return Err(invalid("bundle trocado durante leitura"));
    }
    let mut text = String::new();
    file.take(LIMIT + 1).read_to_string(&mut text)?;
    if text.len() as u64 > LIMIT {
        return Err(invalid("bundle excede limite"));
    }
    let bundle = toml::from_str(&text).map_err(|_| invalid("TOML invalido"))?;
    Ok((bundle, after))
}

#[cfg(not(target_os = "linux"))]
fn read(_: &Path) -> io::Result<(Value, fs::Metadata)> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "edicao requer Linux/WSL",
    ))
}

#[cfg(target_os = "linux")]
fn write_candidate(path: &Path, bundle: &Value, original: &fs::Metadata) -> io::Result<()> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    let parent = path
        .parent()
        .filter(|x| !x.as_os_str().is_empty())
        .ok_or_else(|| invalid("diretorio explicito necessario"))?;
    let filename = path
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| invalid("nome de arquivo invalido"))?;
    let parent_meta = fs::symlink_metadata(parent)?;
    if !parent_meta.is_dir() || parent_meta.permissions().mode() & 0o022 != 0 {
        return Err(invalid(
            "diretorio do bundle nao pode ser gravavel por grupo/outros",
        ));
    }
    let text = toml::to_string_pretty(bundle).map_err(|_| invalid("falha ao codificar bundle"))?;
    if text.len() as u64 > LIMIT {
        return Err(invalid("bundle excede limite"));
    }
    let mut temporary = None;
    for attempt in 0..32_u8 {
        let candidate = parent.join(format!(
            ".{filename}.tmp.{}.{}",
            std::process::id(),
            attempt
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&candidate)
        {
            Ok(mut file) => {
                if let Err(error) = file
                    .write_all(text.as_bytes())
                    .and_then(|()| file.sync_all())
                {
                    let _ = fs::remove_file(&candidate);
                    return Err(error);
                }
                temporary = Some(candidate);
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    let temporary = temporary.ok_or_else(|| io::Error::other("sem arquivo temporario"))?;
    let result = (|| {
        crate::bundle::validate(&temporary)?;
        let current = fs::symlink_metadata(path)?;
        private(&current)?;
        if original.dev() != current.dev() || original.ino() != current.ino() {
            return Err(invalid("bundle alterado concorrentemente"));
        }
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(target_os = "linux"))]
fn write_candidate(_: &Path, _: &Value, _: &fs::Metadata) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "edicao requer Linux/WSL",
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bundle.toml");
        let salt = SaltString::encode_b64(b"sixteen-byte-salt").unwrap();
        let hash = mqtt_broker::auth::argon2_context()
            .hash_password(b"public-test-only", &salt)
            .unwrap()
            .to_string();
        fs::write(
            &path,
            format!("version = 1\n[[users]]\nusername = 'device'\npassword_hash = '{hash}'\ncert_sha256 = 'sha256:{}'\n", "00".repeat(32)),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        (dir, path)
    }

    fn call(parts: &[&str]) -> io::Result<()> {
        run(&parts.iter().map(OsString::from).collect::<Vec<_>>())
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "end-to-end lifecycle covers references and identity changes"
    )]
    fn role_group_and_identity_lifecycle() {
        let (_dir, path) = fixture();
        let p = path.to_str().unwrap();
        call(&[
            "bundle",
            "role",
            "create",
            "--file",
            p,
            "--name",
            "reader",
            "--subscribe",
            "sensor/#",
        ])
        .unwrap();
        call(&[
            "bundle",
            "role",
            "update",
            "--file",
            p,
            "--name",
            "reader",
            "--subscribe",
            "status/#",
        ])
        .unwrap();
        call(&[
            "bundle", "group", "create", "--file", p, "--name", "devices", "--role", "reader",
        ])
        .unwrap();
        call(&[
            "bundle",
            "user",
            "set-membership",
            "--file",
            p,
            "--username",
            "device",
            "--group",
            "devices",
        ])
        .unwrap();
        call(&[
            "bundle",
            "user",
            "update-acl",
            "--file",
            p,
            "--username",
            "device",
            "--publish",
            "sensor/value",
            "--subscribe",
            "sensor/#",
        ])
        .unwrap();
        let before = fs::read(&path).unwrap();
        assert!(call(&["bundle", "role", "delete", "--file", p, "--name", "reader"]).is_err());
        assert!(
            call(&[
                "bundle", "group", "delete", "--file", p, "--name", "devices"
            ])
            .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        call(&[
            "bundle",
            "user",
            "disable",
            "--file",
            p,
            "--username",
            "device",
        ])
        .unwrap();
        assert!(
            read(&path).unwrap().0["users"][0]["disabled"]
                .as_bool()
                .unwrap()
        );
        call(&[
            "bundle",
            "user",
            "enable",
            "--file",
            p,
            "--username",
            "device",
        ])
        .unwrap();
        call(&[
            "bundle",
            "user",
            "set-membership",
            "--file",
            p,
            "--username",
            "device",
        ])
        .unwrap();
        call(&[
            "bundle", "group", "delete", "--file", p, "--name", "devices",
        ])
        .unwrap();
        call(&["bundle", "role", "delete", "--file", p, "--name", "reader"]).unwrap();
        crate::bundle::validate(&path).unwrap();
    }

    #[test]
    fn invalid_acl_public_file_and_symlink_preserve_bundle() {
        let (_dir, path) = fixture();
        let p = path.to_str().unwrap();
        let before = fs::read(&path).unwrap();
        assert!(
            call(&[
                "bundle",
                "role",
                "create",
                "--file",
                p,
                "--name",
                "bad",
                "--publish",
                "sensor/#"
            ])
            .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(call(&["bundle", "role", "list", "--file", p]).is_err());
        let link = path.with_file_name("link.toml");
        symlink(&path, &link).unwrap();
        assert!(call(&["bundle", "role", "list", "--file", link.to_str().unwrap()]).is_err());
    }

    #[test]
    fn competing_editor_is_rejected_without_changes() {
        let (_dir, path) = fixture();
        let lock_path = path.with_file_name(".bundle.toml.lock");
        let lock = open_lock(&lock_path).unwrap();
        FileExt::lock_exclusive(&lock).unwrap();
        let before = fs::read(&path).unwrap();
        let p = path.to_str().unwrap();
        assert!(call(&["bundle", "role", "create", "--file", p, "--name", "read"]).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        FileExt::unlock(&lock).unwrap();
    }
}
