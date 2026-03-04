use std::{env, path::PathBuf};

mod app;
mod colors;
mod loader;
mod panels;
mod viewport;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let args: Option<String> = env::args().skip(1).next();

    let app = match args {
        Some(arg) => {
            let path = PathBuf::from(&arg);
            app::ViewerApp::with_path(path)
        }
        None => app::ViewerApp::default(),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native("GDS Viewer", options, Box::new(|_cc| Ok(Box::new(app))))
}
