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
fn decks_dir_prints_path() {
    let ctx = TestContext::new();
    let output = ctx
        .command()
        .args(["decks", "dir"])
        .output()
        .expect("spawn");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = std::path::Path::new(stdout.trim());
    assert!(
        path.ends_with("cram/decks"),
        "expected path ending with cram/decks, got: {stdout}"
    );
}
