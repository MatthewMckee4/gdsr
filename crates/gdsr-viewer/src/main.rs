mod app;
mod colors;
mod loader;
mod panels;
mod spatial;
mod viewport;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GDS Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(app::ViewerApp::default()))),
    )
}
