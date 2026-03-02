#![allow(dead_code, unreachable_pub)]

use assert_cmd::Command;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Test context for running cram commands.
pub struct TestContext {
    /// Standard filters for this test context.
    filters: Vec<(String, String)>,
    /// The temporary directory for this test.
    pub _root: tempfile::TempDir,
    /// The decks directory within the temporary root.
    decks_dir: PathBuf,
}

impl TestContext {
    /// Create a new test context with a temporary directory.
    pub fn new() -> Self {
        let root = tempfile::TempDir::with_prefix("cram-test")
            .expect("Failed to create test root directory");

        eprintln!("{}", root.path().display());

        let decks_dir = root.path().join("decks");

        let mut filters = Vec::new();

        filters.extend(
            Self::path_patterns(root.path())
                .into_iter()
                .map(|pattern| (pattern, "[TEMP]/".to_string())),
        );

        Self {
            _root: root,
            decks_dir,
            filters,
        }
    }

    /// Generate various escaped regex patterns for the given path.
    pub fn path_patterns(path: impl AsRef<Path>) -> Vec<String> {
        let mut patterns = Vec::new();

        if path.as_ref().exists() {
            patterns.push(Self::path_pattern(
                path.as_ref()
                    .canonicalize()
                    .expect("Failed to create canonical path"),
            ));
        }

        patterns.push(Self::path_pattern(path));

        patterns
    }

    /// Generate an escaped regex pattern for the given path.
    fn path_pattern(path: impl AsRef<Path>) -> String {
        format!(
            r"{}(\\|\/)*",
            regex::escape(&path.as_ref().display().to_string()).replace(r"\\", r"(\\|\/)+")
        )
    }

    /// Standard snapshot filters _plus_ those for this test context.
    pub fn filters(&self) -> Vec<(&str, &str)> {
        self.filters
            .iter()
            .map(|(p, r)| (p.as_str(), r.as_str()))
            .chain(INSTA_FILTERS.iter().copied())
            .collect()
    }

    /// Create a `cram` command with an isolated decks directory.
    pub fn command(&self) -> Command {
        let mut command = Self::new_command();
        command.env("CRAM_DECKS_DIR", &self.decks_dir);
        command
    }

    fn new_command() -> Command {
        Command::new(get_bin())
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the cram binary that cargo built before launching the tests.
pub fn get_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cram"))
}

/// Common filters for snapshot testing.
pub static INSTA_FILTERS: &[(&str, &str)] = &[
    // Normalize Windows line endings
    (r"\r\n", "\n"),
    // Normalize Windows paths
    (r"\\([\w\d]|\.)", "/$1"),
    // Strip ANSI color codes
    (r"[\x1b]\[[0-9;]*m", ""),
    // cram version display
    (
        r"cram(-.*)? \d+\.\d+\.\d+(-(alpha|beta|rc)\.\d+)?",
        r"cram [VERSION]",
    ),
];

/// Helper method to apply filters to a string.
pub fn apply_filters<T: AsRef<str>>(mut snapshot: String, filters: impl AsRef<[(T, T)]>) -> String {
    for (matcher, replacement) in filters.as_ref() {
        let re = Regex::new(matcher.as_ref()).expect("Do you need to regex::escape your filter?");
        if re.is_match(&snapshot) {
            snapshot = re.replace_all(&snapshot, replacement.as_ref()).to_string();
        }
    }
    snapshot
}

/// Execute the command and format its output status, stdout and stderr into a snapshot string.
#[allow(clippy::print_stderr)]
pub fn run_and_format(
    cmd: &mut Command,
    filters: &[(&str, &str)],
) -> (String, std::process::Output) {
    let program = cmd.get_program().to_string_lossy().to_string();

    let output = cmd
        .output()
        .unwrap_or_else(|err| panic!("Failed to spawn {program}: {err}"));

    let snapshot = apply_filters(
        format!(
            "success: {:?}\nexit_code: {}\n----- stdout -----\n{}\n----- stderr -----\n{}",
            output.status.success(),
            output.status.code().unwrap_or(!0),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ),
        filters,
    );

    (snapshot, output)
}

/// Run snapshot testing with the cram command.
#[macro_export]
macro_rules! cram_snapshot {
    ($cmd:expr, @$snapshot:literal) => {{
        cram_snapshot!($crate::common::INSTA_FILTERS.to_vec(), $cmd, @$snapshot)
    }};
    ($filters:expr, $cmd:expr, @$snapshot:literal) => {{
        let (snapshot, output) = $crate::common::run_and_format(
            $cmd,
            &$filters,
        );
        ::insta::assert_snapshot!(snapshot, @$snapshot);
        output
    }};
}

#[allow(unused_imports)]
pub(crate) use cram_snapshot;
