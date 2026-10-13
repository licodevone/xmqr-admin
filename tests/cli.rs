// SPDX-License-Identifier: MIT
use std::process::Command;

#[test]
fn version_and_help_are_available_without_private_configuration() {
    for flag in ["--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
            .arg(flag)
            .env("XMQR_ADMIN_TOKEN_FILE", "/nonexistent/private.token")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            format!("xmqr-admin {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
        );
        assert_eq!(output.stderr, [] as [u8; 0]);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("somente leitura"));
}

#[test]
fn invalid_arguments_never_echo_sensitive_values() {
    let output = Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
        .args(["--version", "private-value-must-not-echo"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private-value-must-not-echo"));
}

#[cfg(unix)]
#[test]
fn backup_roundtrip_refuses_active_lock_existing_destination_and_tampering() {
    use mqtt_broker::persistence::Store;
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state source");
    let backup = root.path().join("backup copy");
    let restored = root.path().join("restored state");
    std::fs::create_dir(&state).unwrap();
    let mut store = Store::open(&state).unwrap();
    store.put(b"key", b"accepted").unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_xmqr-admin"))
            .args(args)
            .output()
            .unwrap()
    };
    let backup_args = [
        "state",
        "backup",
        "--state-dir",
        state.to_str().unwrap(),
        "--destination",
        backup.to_str().unwrap(),
    ];
    assert!(!run(&backup_args).status.success());
    assert!(!backup.exists());
    drop(store);
    assert!(run(&backup_args).status.success());
    assert!(
        run(&["state", "verify", "--backup", backup.to_str().unwrap()])
            .status
            .success()
    );
    let restore_args = [
        "state",
        "restore",
        "--backup",
        backup.to_str().unwrap(),
        "--state-dir",
        restored.to_str().unwrap(),
    ];
    assert!(run(&restore_args).status.success());
    assert!(!run(&restore_args).status.success());
    let recovered = Store::open(&restored).unwrap();
    assert_eq!(recovered.get(b"key"), Some(b"accepted".as_slice()));
    let snapshot = backup.join("state.snapshot");
    let mut bytes = std::fs::read(&snapshot).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    std::fs::write(&snapshot, bytes).unwrap();
    assert!(
        !run(&["state", "verify", "--backup", backup.to_str().unwrap()])
            .status
            .success()
    );
}
