use clap::Parser;

mod app;
mod colors;
mod drawable;
mod loader;
mod panels;
#[cfg(test)]
mod property_tests;
mod spatial;
mod state;
#[cfg(test)]
mod testutil;
mod viewport;

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "FILE")]
    file: Option<std::path::PathBuf>,
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let args = Args::parse();

    let app = match args.file {
        Some(file) => app::ViewerApp::with_path(&file),
        None => app::ViewerApp::default(),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native("GDS Viewer", options, Box::new(|_cc| Ok(Box::new(app))))
}
