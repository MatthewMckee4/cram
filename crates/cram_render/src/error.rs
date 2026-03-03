use std::fmt;

use thiserror::Error;

/// A single diagnostic extracted from a Typst compilation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    /// 1-based line number in the user's source (preamble lines subtracted).
    /// `None` when the span cannot be resolved to a location.
    pub line: Option<usize>,
    /// 1-based column number. `None` when unavailable.
    pub column: Option<usize>,
    /// The error message produced by the Typst compiler.
    pub message: String,
    /// Optional hints provided by the compiler (e.g. "did you mean ...?").
    pub hints: Vec<String>,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(col)) => write!(f, "line {line}, column {col}: {}", self.message)?,
            (Some(line), None) => write!(f, "line {line}: {}", self.message)?,
            _ => write!(f, "{}", self.message)?,
        }
        for hint in &self.hints {
            write!(f, "\n  hint: {hint}")?;
        }
        if let Some(suggestion) = suggest_fix(&self.message) {
            write!(f, "\n  hint: {suggestion}")?;
        }
        Ok(())
    }
}

/// Suggest a fix for common Typst mistakes based on the error message.
fn suggest_fix(message: &str) -> Option<&'static str> {
    let msg = message.to_lowercase();
    if msg.contains("expected hash")
        || msg.contains("unexpected end of block comment")
        || msg.contains("unclosed delimiter")
    {
        return Some("check for mismatched braces, brackets, or parentheses");
    }
    if msg.contains("unknown variable") {
        return Some("function calls in Typst use `#` prefix, e.g. `#text(\"hello\")`");
    }
    if msg.contains("expected expression") {
        return Some("make sure `#` is followed by a valid Typst expression");
    }
    if msg.contains("unclosed string") {
        return Some("strings must be wrapped in matching double quotes: `\"...\"`");
    }
    None
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("{}", format_compile_errors(.0))]
    Compile(Vec<CompileError>),
    #[error("no pages in document")]
    NoPages,
    #[error("png encode failed")]
    Encode,
}

fn format_compile_errors(errors: &[CompileError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_error_with_location() {
        let err = CompileError {
            line: Some(3),
            column: Some(5),
            message: "unknown variable: foo".to_string(),
            hints: vec![],
        };
        let formatted = err.to_string();
        assert!(formatted.contains("line 3, column 5"));
        assert!(formatted.contains("unknown variable: foo"));
        assert!(formatted.contains("hint: function calls in Typst use `#` prefix"));
    }

    #[test]
    fn compile_error_without_location() {
        let err = CompileError {
            line: None,
            column: None,
            message: "something went wrong".to_string(),
            hints: vec![],
        };
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn compile_error_with_compiler_hints() {
        let err = CompileError {
            line: Some(1),
            column: Some(1),
            message: "type error".to_string(),
            hints: vec!["try using `str()` to convert".to_string()],
        };
        let formatted = err.to_string();
        assert!(formatted.contains("line 1, column 1: type error"));
        assert!(formatted.contains("hint: try using `str()` to convert"));
    }

    #[test]
    fn compile_error_line_only() {
        let err = CompileError {
            line: Some(2),
            column: None,
            message: "bad syntax".to_string(),
            hints: vec![],
        };
        assert!(err.to_string().starts_with("line 2: bad syntax"));
    }

    #[test]
    fn suggest_fix_unclosed_delimiter() {
        assert!(suggest_fix("unclosed delimiter").is_some());
        assert!(suggest_fix("unclosed Delimiter found").is_some());
    }

    #[test]
    fn suggest_fix_unknown_variable() {
        let hint = suggest_fix("unknown variable: foo").expect("should suggest");
        assert!(hint.contains("#"));
    }

    #[test]
    fn suggest_fix_expected_expression() {
        assert!(suggest_fix("expected expression, found end").is_some());
    }

    #[test]
    fn suggest_fix_unclosed_string() {
        assert!(suggest_fix("unclosed string").is_some());
    }

    #[test]
    fn suggest_fix_no_match() {
        assert!(suggest_fix("some random error").is_none());
    }

    #[test]
    fn render_error_display_multiple() {
        let err = RenderError::Compile(vec![
            CompileError {
                line: Some(1),
                column: Some(2),
                message: "first error".to_string(),
                hints: vec![],
            },
            CompileError {
                line: Some(3),
                column: Some(4),
                message: "second error".to_string(),
                hints: vec![],
            },
        ]);
        let formatted = err.to_string();
        assert!(formatted.contains("line 1, column 2: first error"));
        assert!(formatted.contains("line 3, column 4: second error"));
    }

    #[test]
    fn render_error_no_pages() {
        assert_eq!(RenderError::NoPages.to_string(), "no pages in document");
    }

    #[test]
    fn render_error_encode() {
        assert_eq!(RenderError::Encode.to_string(), "png encode failed");
    }
}
