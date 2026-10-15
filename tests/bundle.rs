#![cfg(target_os = "linux")]
use argon2::password_hash::{PasswordHasher, SaltString};
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

fn fixture() -> String {
    let salt = SaltString::encode_b64(b"public-test-salt").unwrap();
    let hash = mqtt_broker::auth::argon2_context()
        .hash_password(b"private-sentinel", &salt)
        .unwrap()
        .to_string();
    format!(
        "version=1\n[[users]]\nusername='fixture'\npassword_hash='{hash}'\ncert_sha256='sha256:{}'\n",
        "00".repeat(32)
    )
}

#[test]
fn secure_bundle_validates_without_writing_or_leaking() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("bundle.toml");
    let original = fixture();
    fs::write(&path, &original).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
        .args(["bundle", "validate", "--file"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "Bundle seguro valido. Nenhum arquivo foi alterado."
    );
    assert_eq!(output.stderr, Vec::<u8>::new());
}

#[test]
fn invalid_references_public_files_and_symlinks_fail_without_contents() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("bundle.toml");
    let link = root.path().join("link.toml");
    let original = fixture();
    for (text, mode) in [
        (
            format!("{original}roles=['missing-private-sentinel']\n"),
            0o600,
        ),
        (original.clone(), 0o644),
        ("invalid-private-sentinel".into(), 0o600),
    ] {
        fs::write(&path, &text).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
            .args(["bundle", "validate", "--file"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(output.stdout, Vec::<u8>::new());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-sentinel"));
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
    }
    fs::write(&path, original).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    std::os::unix::fs::symlink(&path, &link).unwrap();
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
            .args(["bundle", "validate", "--file"])
            .arg(link)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn editor_cli_applies_private_candidate_and_never_prints_hash() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("bundle.toml");
    let original = fixture();
    fs::write(&path, &original).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let edit = |arguments: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
            .args(arguments)
            .arg("--file")
            .arg(&path)
            .output()
            .unwrap()
    };
    let output = edit(&["bundle", "role", "create"]);
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    let output = Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
        .args(["bundle", "role", "create", "--file"])
        .arg(&path)
        .args(["--name", "reader", "--subscribe", "sensor/#"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("argon2"));
    let result = Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
        .args(["bundle", "validate", "--file"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(fs::read_to_string(&path).unwrap().contains("reader"));
}
