use assert_cmd::Command;

#[expect(deprecated)]
fn cram() -> Command {
    Command::cargo_bin("cram").expect("binary exists")
}

#[test]
fn decks_list_empty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = cram()
        .args(["decks", "list"])
        .env("HOME", dir.path())
        .env("XDG_DATA_HOME", dir.path().join("data"))
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    insta::assert_snapshot!(stdout, @"No decks found.");
}

#[test]
fn decks_dir_prints_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = cram()
        .args(["decks", "dir"])
        .env("HOME", dir.path())
        .env("XDG_DATA_HOME", dir.path().join("data"))
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.trim().ends_with("cram/decks"),
        "expected path ending with cram/decks, got: {stdout}"
    );
}
