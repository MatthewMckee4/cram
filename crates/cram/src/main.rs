fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into()))
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Cram")
            .with_inner_size([960.0, 680.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Cram",
        options,
        Box::new(|cc| Ok(Box::new(cram_ui::CramApp::new(cc)))),
    ) {
        eprintln!("cram: {e}");
        std::process::exit(1);
    }
}
