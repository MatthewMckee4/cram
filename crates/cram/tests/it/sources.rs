use crate::common::TestContext;
use crate::common::cram_snapshot;

#[test]
fn sources_list_empty() {
    let ctx = TestContext::new();
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "sources"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    No linked sources.

    ----- stderr -----
    ");
}

#[test]
fn link_and_list_source() {
    let ctx = TestContext::new();
    let linked_dir = ctx._root.path().join("linked");
    std::fs::create_dir_all(&linked_dir).expect("create linked dir");

    // Write a deck file into the linked dir
    std::fs::write(
        linked_dir.join("test-deck.toml"),
        r#"name = "test-deck"
description = ""
created = "2026-03-02"
cards = []
"#,
    )
    .expect("write deck");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "link", linked_dir.to_str().unwrap()]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Linked: [TEMP]/linked

    ----- stderr -----
    ");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "sources"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    [TEMP]/linked (folder, not a git repo)

    ----- stderr -----
    ");
}

#[test]
fn link_shows_decks_in_list() {
    let ctx = TestContext::new();
    let linked_dir = ctx._root.path().join("linked");
    std::fs::create_dir_all(&linked_dir).expect("create linked dir");

    std::fs::write(
        linked_dir.join("remote-deck.toml"),
        r#"name = "remote-deck"
description = "from linked source"
created = "2026-03-02"
cards = []
"#,
    )
    .expect("write deck");

    ctx.command()
        .args(["decks", "link", linked_dir.to_str().unwrap()])
        .output()
        .expect("link");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "list"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    remote-deck (0 cards) [[TEMP]/linked/remote-deck.toml]

    ----- stderr -----
    ");
}

#[test]
fn unlink_source() {
    let ctx = TestContext::new();
    let linked_dir = ctx._root.path().join("linked");
    std::fs::create_dir_all(&linked_dir).expect("create linked dir");

    ctx.command()
        .args(["decks", "link", linked_dir.to_str().unwrap()])
        .output()
        .expect("link");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "unlink", linked_dir.to_str().unwrap()]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Unlinked: [TEMP]/linked

    ----- stderr -----
    ");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "sources"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    No linked sources.

    ----- stderr -----
    ");
}

#[test]
fn sync_with_no_sources() {
    let ctx = TestContext::new();
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "sync"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    No linked sources to sync.

    ----- stderr -----
    ");
}

#[test]
fn sync_non_git_source() {
    let ctx = TestContext::new();
    let linked_dir = ctx._root.path().join("linked");
    std::fs::create_dir_all(&linked_dir).expect("create linked dir");

    ctx.command()
        .args(["decks", "link", linked_dir.to_str().unwrap()])
        .output()
        .expect("link");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "sync"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    [TEMP]/linked: Not a git repo, skipping.

    ----- stderr -----
    ");
}

#[test]
fn new_deck_creates_file() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("my-deck.toml");
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "new", deck_path.to_str().unwrap()]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Created deck: [TEMP]/my-deck.toml

    ----- stderr -----
    ");
    assert!(deck_path.exists());
}

#[test]
fn new_deck_rejects_non_toml_extension() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("bad.json");
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "new", deck_path.to_str().unwrap()]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: deck file must have a .toml extension
    ");
}

#[test]
fn new_deck_rejects_existing_file() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("exists.toml");
    std::fs::write(&deck_path, "").expect("create file");
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "new", deck_path.to_str().unwrap()]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: file already exists: [TEMP]/exists.toml
    ");
}

#[test]
fn new_deck_creates_parent_dirs() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("sub/dir/deep.toml");
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "new", deck_path.to_str().unwrap()]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Created deck: [TEMP]/sub/dir/deep.toml

    ----- stderr -----
    ");
    assert!(deck_path.exists());
}

#[test]
fn link_file_source() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("my-deck.toml");
    std::fs::write(
        &deck_path,
        r#"name = "my-deck"
description = "a file source"
created = "2026-03-02"
cards = []
"#,
    )
    .expect("write deck");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "link", deck_path.to_str().unwrap()]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Linked: [TEMP]/my-deck.toml

    ----- stderr -----
    ");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "sources"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    [TEMP]/my-deck.toml (file, not a git repo)

    ----- stderr -----
    ");

    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "list"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    my-deck (0 cards) [[TEMP]/my-deck.toml]

    ----- stderr -----
    ");
}

#[test]
fn link_nonexistent_directory() {
    let ctx = TestContext::new();
    cram_snapshot!(ctx.filters(), ctx.command().args(["decks", "link", "/nonexistent/path"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: path does not exist: /nonexistent/path
    ");
}
