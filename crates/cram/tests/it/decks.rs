use crate::common::TestContext;
use crate::common::cram_snapshot;

#[test]
fn decks_list_empty() {
    let ctx = TestContext::new();
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "list"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    No decks found.

    ----- stderr -----
    ");
}

#[test]
fn decks_check_empty() {
    let ctx = TestContext::new();
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "check"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    No decks found.

    ----- stderr -----
    ");
}

#[test]
fn decks_check_clean_deck() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("decks").join("test.toml");
    std::fs::create_dir_all(deck_path.parent().unwrap()).expect("create dir");
    std::fs::write(
        &deck_path,
        r#"name = "test"
description = ""
created = "2026-03-02"
preamble = ""

[[cards]]
id = "00000000-0000-0000-0000-000000000001"
front = "Q1"
back = "A1"
"#,
    )
    .expect("write deck");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "check"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    test (1 cards)

    Checked 1 card across 1 deck: 0 errors, 0 warnings

    ----- stderr -----
    ");
}

#[test]
fn decks_check_reports_broken_card() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("decks").join("broken.toml");
    std::fs::create_dir_all(deck_path.parent().unwrap()).expect("create dir");
    std::fs::write(
        &deck_path,
        r##"name = "broken"
description = ""
created = "2026-03-02"
preamble = ""

[[cards]]
id = "00000000-0000-0000-0000-000000000001"
front = "#unknown_func()"
back = "ok"
"##,
    )
    .expect("write deck");

    let output = ctx
        .command()
        .args(["decks", "check"])
        .output()
        .expect("run");
    assert!(!output.status.success(), "expected non-zero exit");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let plain_stdout = regex::Regex::new(r"\x1b\[[0-9;]*m")
        .expect("compile ansi regex")
        .replace_all(&stdout, "");
    assert!(
        plain_stdout.contains("broken (1 cards)"),
        "stdout: {plain_stdout}"
    );
    assert!(
        plain_stdout.contains("error card 00000000-0000-0000-0000-000000000001 (front)"),
        "stdout: {plain_stdout}"
    );
    assert!(
        plain_stdout.contains("unknown variable: unknown_func"),
        "stdout: {plain_stdout}"
    );
    assert!(stderr.contains("compile errors found"), "stderr: {stderr}");
}

#[test]
fn decks_dir_prints_path() {
    let ctx = TestContext::new();
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "dir"]), @"
    success: true
    exit_code: 0
    ----- stdout -----
    [TEMP]/decks

    ----- stderr -----
    ");
}
