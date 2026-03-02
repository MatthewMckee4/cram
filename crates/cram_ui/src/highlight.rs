use egui::{Color32, FontId, TextFormat, text::LayoutJob};

struct Colors {
    text: Color32,
    comment: Color32,
    string: Color32,
    keyword: Color32,
    function: Color32,
    math: Color32,
    heading: Color32,
}

impl Colors {
    fn for_mode(dark: bool) -> Self {
        if dark {
            Self {
                text: Color32::from_rgb(204, 204, 204),
                comment: Color32::from_rgb(106, 115, 125),
                string: Color32::from_rgb(152, 195, 121),
                keyword: Color32::from_rgb(198, 120, 221),
                function: Color32::from_rgb(97, 175, 239),
                math: Color32::from_rgb(229, 192, 123),
                heading: Color32::from_rgb(224, 108, 117),
            }
        } else {
            Self {
                text: Color32::from_rgb(36, 41, 46),
                comment: Color32::from_rgb(106, 115, 125),
                string: Color32::from_rgb(22, 128, 0),
                keyword: Color32::from_rgb(167, 29, 93),
                function: Color32::from_rgb(0, 92, 197),
                math: Color32::from_rgb(177, 72, 0),
                heading: Color32::from_rgb(128, 0, 0),
            }
        }
    }
}

fn is_typst_keyword(word: &[u8]) -> bool {
    matches!(
        word,
        b"#let"
            | b"#set"
            | b"#show"
            | b"#import"
            | b"#include"
            | b"#if"
            | b"#else"
            | b"#for"
            | b"#while"
            | b"#return"
            | b"#none"
            | b"#auto"
            | b"#true"
            | b"#false"
            | b"#context"
            | b"#break"
            | b"#continue"
    )
}

/// Build a `LayoutJob` with Typst syntax highlighting applied.
pub fn typst_layout_job(text: &str, dark_mode: bool) -> LayoutJob {
    let c = Colors::for_mode(dark_mode);
    let font = FontId::monospace(14.0);

    let text_fmt = TextFormat::simple(font.clone(), c.text);
    let comment_fmt = TextFormat::simple(font.clone(), c.comment);
    let string_fmt = TextFormat::simple(font.clone(), c.string);
    let keyword_fmt = TextFormat::simple(font.clone(), c.keyword);
    let function_fmt = TextFormat::simple(font.clone(), c.function);
    let math_fmt = TextFormat::simple(font.clone(), c.math);
    let heading_fmt = TextFormat::simple(font.clone(), c.heading);

    let mut job = LayoutJob::default();
    let b = text.as_bytes();
    let len = b.len();
    let mut i = 0;
    let mut plain = 0;

    while i < len {
        // Line comment: //
        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'/' {
            if plain < i {
                job.append(&text[plain..i], 0.0, text_fmt.clone());
            }
            let start = i;
            while i < len && b[i] != b'\n' {
                i += 1;
            }
            job.append(&text[start..i], 0.0, comment_fmt.clone());
            plain = i;
            continue;
        }

        // String: "..."
        if b[i] == b'"' {
            if plain < i {
                job.append(&text[plain..i], 0.0, text_fmt.clone());
            }
            let start = i;
            i += 1;
            while i < len {
                if b[i] == b'\\' && i + 1 < len {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            job.append(&text[start..i], 0.0, string_fmt.clone());
            plain = i;
            continue;
        }

        // Math: $...$
        if b[i] == b'$' {
            if plain < i {
                job.append(&text[plain..i], 0.0, text_fmt.clone());
            }
            let start = i;
            i += 1;
            while i < len && b[i] != b'$' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
            job.append(&text[start..i], 0.0, math_fmt.clone());
            plain = i;
            continue;
        }

        // Hash keyword/function: #word
        if b[i] == b'#' && i + 1 < len && b[i + 1].is_ascii_alphabetic() {
            if plain < i {
                job.append(&text[plain..i], 0.0, text_fmt.clone());
            }
            let start = i;
            i += 1;
            while i < len && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'.') {
                i += 1;
            }
            let fmt = if is_typst_keyword(&b[start..i]) {
                &keyword_fmt
            } else {
                &function_fmt
            };
            job.append(&text[start..i], 0.0, fmt.clone());
            plain = i;
            continue;
        }

        // Heading: = at start of line
        if b[i] == b'=' && (i == 0 || b[i.saturating_sub(1)] == b'\n') {
            if plain < i {
                job.append(&text[plain..i], 0.0, text_fmt.clone());
            }
            let start = i;
            while i < len && b[i] != b'\n' {
                i += 1;
            }
            job.append(&text[start..i], 0.0, heading_fmt.clone());
            plain = i;
            continue;
        }

        // Advance past current character (handles multi-byte UTF-8)
        i += 1;
        while i < len && (b[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }

    if plain < len {
        job.append(&text[plain..], 0.0, text_fmt);
    }

    job
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_comment() {
        let job = typst_layout_job("// hello", true);
        assert_eq!(job.sections.len(), 1);
    }

    #[test]
    fn highlights_keyword_and_text() {
        let job = typst_layout_job("#let x = 1", true);
        // "#let" is one section, " x = 1" is another
        assert_eq!(job.sections.len(), 2);
    }

    #[test]
    fn highlights_math() {
        let job = typst_layout_job("$x^2$", true);
        assert_eq!(job.sections.len(), 1);
    }

    #[test]
    fn plain_text_single_section() {
        let job = typst_layout_job("hello world", true);
        assert_eq!(job.sections.len(), 1);
    }

    #[test]
    fn empty_input() {
        let job = typst_layout_job("", true);
        assert!(job.sections.is_empty());
    }
}
