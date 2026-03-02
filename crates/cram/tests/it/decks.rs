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

    #[cfg(target_os = "linux")]
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "dir"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    [TEMP]/data/cram/decks

    ----- stderr -----
    ");

    #[cfg(target_os = "macos")]
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "dir"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    [TEMP]Library/Application Support/cram/decks

    ----- stderr -----
    ");

    #[cfg(target_os = "windows")]
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "dir"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    [TEMP]/AppData/Roaming/cram/decks

    ----- stderr -----
    ");
}
