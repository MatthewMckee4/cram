use criterion::{Criterion, criterion_group, criterion_main};

fn bench_heading(c: &mut Criterion) {
    c.bench_function("render_heading", |b| {
        b.iter(|| {
            cram_render::render("= Hello World").expect("render");
        });
    });
}

fn bench_body_text(c: &mut Criterion) {
    c.bench_function("render_body_text", |b| {
        b.iter(|| {
            cram_render::render("This is *bold* and _italic_ text with some content.")
                .expect("render");
        });
    });
}

fn bench_math(c: &mut Criterion) {
    c.bench_function("render_math", |b| {
        b.iter(|| {
            cram_render::render("$ integral_0^infinity e^(-x^2) d x = sqrt(pi) / 2 $")
                .expect("render");
        });
    });
}

fn bench_complex_card(c: &mut Criterion) {
    let source = "\
#set text(size: 14pt)
= Euler's Identity

The most beautiful equation in mathematics:

$ e^(i pi) + 1 = 0 $

This connects five fundamental constants:
- $e$ (Euler's number)
- $i$ (imaginary unit)
- $pi$ (pi)
- $1$ (multiplicative identity)
- $0$ (additive identity)
";
    c.bench_function("render_complex_card", |b| {
        b.iter(|| {
            cram_render::render(source).expect("render");
        });
    });
}

criterion_group!(
    benches,
    bench_heading,
    bench_body_text,
    bench_math,
    bench_complex_card
);
criterion_main!(benches);
