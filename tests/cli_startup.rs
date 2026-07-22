use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn tutor_does_not_create_the_normal_database() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let data_home =
        std::env::temp_dir().join(format!("aporic-tutor-test-{}-{nonce}", std::process::id()));

    let mut child = Command::new(env!("CARGO_BIN_EXE_aporic"))
        .arg("tutor")
        .env("XDG_DATA_HOME", &data_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"quit\n").unwrap();

    assert!(child.wait().unwrap().success());
    let database_created = data_home.join("aporic/aporic.db").exists();
    if data_home.exists() {
        std::fs::remove_dir_all(&data_home).unwrap();
    }
    assert!(!database_created);
}

#[test]
fn help_does_not_advertise_legacy_ids() {
    let output = Command::new(env!("CARGO_BIN_EXE_aporic"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("UUID or unique UUID prefix"));
    assert!(!stdout.contains("legacy task ID"));
}
