use crate::common::TestContext;
use crate::common::cram_snapshot;

#[test]
fn export_deck_to_toml() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("decks").join("test.toml");
    std::fs::create_dir_all(deck_path.parent().unwrap()).expect("create dir");
    std::fs::write(
        &deck_path,
        r#"name = "test"
description = "a test deck"
created = "2026-03-02"
preamble = ""

[[cards]]
id = "00000000-0000-0000-0000-000000000001"
front = "Q1"
back = "A1"
"#,
    )
    .expect("write deck");

    let export_path = ctx._root.path().join("exported.toml");
    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "export", "test", export_path.to_str().unwrap()]),
        @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Exported "test" to [TEMP]/exported.toml

    ----- stderr -----
    "#
    );
    assert!(export_path.exists());
}

#[test]
fn export_missing_deck_errors() {
    let ctx = TestContext::new();
    let export_path = ctx._root.path().join("exported.toml");
    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "export", "nonexistent", export_path.to_str().unwrap()]),
        @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: deck not found: nonexistent
    "
    );
}

#[test]
fn export_rejects_non_toml() {
    let ctx = TestContext::new();
    let export_path = ctx._root.path().join("bad.json");
    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "export", "test", export_path.to_str().unwrap()]),
        @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: export file must have a .toml extension
    "
    );
}

#[test]
fn import_toml_deck() {
    let ctx = TestContext::new();
    let import_path = ctx._root.path().join("external.toml");
    std::fs::write(
        &import_path,
        r#"name = "imported"
description = "from file"
created = "2026-03-02"
preamble = ""

[[cards]]
id = "00000000-0000-0000-0000-000000000001"
front = "Q1"
back = "A1"
"#,
    )
    .expect("write deck");

    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "import", import_path.to_str().unwrap()]),
        @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Imported "imported" (1 cards)

    ----- stderr -----
    "#
    );

    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "list"]),
        @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported (1 cards)

    ----- stderr -----
    "
    );
}

#[test]
fn import_csv_deck() {
    let ctx = TestContext::new();
    let import_path = ctx._root.path().join("vocab.csv");
    std::fs::write(&import_path, "hello,world\nfoo,bar\n").expect("write csv");

    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "import", import_path.to_str().unwrap()]),
        @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Imported "vocab" (2 cards)

    ----- stderr -----
    "#
    );

    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "list"]),
        @r"
    success: true
    exit_code: 0
    ----- stdout -----
    vocab (2 cards)

    ----- stderr -----
    "
    );
}

#[test]
fn import_missing_file_errors() {
    let ctx = TestContext::new();
    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "import", "/nonexistent/deck.toml"]),
        @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: file not found: /nonexistent/deck.toml
    "
    );
}

#[test]
fn import_unsupported_format_errors() {
    let ctx = TestContext::new();
    let bad_path = ctx._root.path().join("deck.json");
    std::fs::write(&bad_path, "{}").expect("write json");

    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "import", bad_path.to_str().unwrap()]),
        @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: unsupported file format: .json (expected .toml or .csv)
    "
    );
}

#[test]
fn export_import_roundtrip() {
    let ctx = TestContext::new();
    let deck_path = ctx._root.path().join("decks").join("roundtrip.toml");
    std::fs::create_dir_all(deck_path.parent().unwrap()).expect("create dir");
    std::fs::write(
        &deck_path,
        r##"name = "roundtrip"
description = "roundtrip test"
created = "2026-03-02"
preamble = "#set text(size: 14pt)"

[[cards]]
id = "00000000-0000-0000-0000-000000000001"
front = "What is Rust?"
back = "A systems language"

[[cards]]
id = "00000000-0000-0000-0000-000000000002"
front = "What is Cram?"
back = "A flashcard app"
"##,
    )
    .expect("write deck");

    let export_path = ctx._root.path().join("exported.toml");
    ctx.command()
        .args([
            "decks",
            "export",
            "roundtrip",
            export_path.to_str().unwrap(),
        ])
        .output()
        .expect("export");

    std::fs::remove_file(&deck_path).expect("remove original");

    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "list"]),
        @r"
    success: true
    exit_code: 0
    ----- stdout -----
    No decks found.

    ----- stderr -----
    "
    );

    ctx.command()
        .args(["decks", "import", export_path.to_str().unwrap()])
        .output()
        .expect("import");

    cram_snapshot!(
        ctx.filters(),
        ctx.command().args(["decks", "list"]),
        @r"
    success: true
    exit_code: 0
    ----- stdout -----
    roundtrip (2 cards)

    ----- stderr -----
    "
    );
}
