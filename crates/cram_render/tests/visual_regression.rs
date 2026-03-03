use image::GenericImageView;

/// Render a card and return a summary string suitable for snapshot testing.
///
/// Captures structural properties of the rendered image (dimensions, size
/// class, content presence) using coarse buckets so the snapshots remain
/// stable across platforms with different system fonts while still detecting
/// meaningful visual regressions (blank renders, layout breakage, missing
/// content).
fn render_summary(source: &str, dark_mode: bool) -> String {
    let bytes = cram_render::render(source, dark_mode).expect("render failed");
    let img = image::load_from_memory(&bytes).expect("decode failed");
    let (width, height) = img.dimensions();

    let rgba = img.to_rgba8();
    let total = (width * height) as usize;

    let non_transparent: usize = rgba.pixels().filter(|p| p[3] > 0).count();

    let content_coverage = if total > 0 {
        match (non_transparent as f64 / total as f64 * 100.0) as u32 {
            0 => "none",
            1..=25 => "sparse",
            26..=75 => "moderate",
            _ => "dense",
        }
    } else {
        "none"
    };

    let size_bucket = match bytes.len() {
        0..=1_000 => "tiny (<1KB)",
        1_001..=5_000 => "small (1-5KB)",
        5_001..=20_000 => "medium (5-20KB)",
        20_001..=100_000 => "large (20-100KB)",
        _ => "very large (>100KB)",
    };

    let width_bucket = match width {
        0..=100 => "narrow (0-100px)",
        101..=300 => "compact (101-300px)",
        301..=600 => "medium (301-600px)",
        601..=1000 => "wide (601-1000px)",
        _ => "very wide (>1000px)",
    };

    let height_bucket = match height {
        0..=50 => "short (0-50px)",
        51..=150 => "compact (51-150px)",
        151..=300 => "medium (151-300px)",
        301..=500 => "tall (301-500px)",
        _ => "very tall (>500px)",
    };

    format!(
        "valid_png: true\n\
         width: {width_bucket}\n\
         height: {height_bucket}\n\
         size: {size_bucket}\n\
         has_content: {has_content}\n\
         content_coverage: {content_coverage}",
        has_content = non_transparent > 0,
    )
}

#[test]
fn plain_text_light() {
    let summary = render_summary("Hello, world!", false);
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: compact (101-300px)
    height: short (0-50px)
    size: small (1-5KB)
    has_content: true
    content_coverage: sparse
    ");
}

#[test]
fn heading_with_body_light() {
    let summary = render_summary("= Chapter One\nThis is a heading with body text.", false);
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: medium (301-600px)
    height: compact (51-150px)
    size: medium (5-20KB)
    has_content: true
    content_coverage: sparse
    ");
}

#[test]
fn math_equation_light() {
    let summary = render_summary(
        "$ integral_0^infinity e^(-x^2) dif x = sqrt(pi) / 2 $",
        false,
    );
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: compact (101-300px)
    height: compact (51-150px)
    size: small (1-5KB)
    has_content: true
    content_coverage: sparse
    ");
}

#[test]
fn code_block_light() {
    let source = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
    let summary = render_summary(source, false);
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: compact (101-300px)
    height: compact (51-150px)
    size: medium (5-20KB)
    has_content: true
    content_coverage: sparse
    ");
}

#[test]
fn formatted_text_light() {
    let source = "*Bold text*, _emphasized text_, and `inline code`.";
    let summary = render_summary(source, false);
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: medium (301-600px)
    height: short (0-50px)
    size: medium (5-20KB)
    has_content: true
    content_coverage: sparse
    ");
}

#[test]
fn plain_text_dark() {
    let summary = render_summary("Hello, world!", true);
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: compact (101-300px)
    height: short (0-50px)
    size: small (1-5KB)
    has_content: true
    content_coverage: sparse
    ");
}

#[test]
fn math_equation_dark() {
    let summary = render_summary(
        "$ integral_0^infinity e^(-x^2) dif x = sqrt(pi) / 2 $",
        true,
    );
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: compact (101-300px)
    height: compact (51-150px)
    size: small (1-5KB)
    has_content: true
    content_coverage: sparse
    ");
}

#[test]
fn bullet_list_light() {
    let source = "Key concepts:\n- Ownership\n- Borrowing\n- Lifetimes";
    let summary = render_summary(source, false);
    insta::assert_snapshot!(summary, @r"
    valid_png: true
    width: compact (101-300px)
    height: compact (51-150px)
    size: medium (5-20KB)
    has_content: true
    content_coverage: sparse
    ");
}
