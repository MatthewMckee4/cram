pub fn launch_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Cram")
            .with_inner_size([1200.0, 800.0]),
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
