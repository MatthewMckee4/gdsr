use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy::winit::WinitSettings;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use clap::Parser;

mod app;
mod bevy_renderer;
mod bevy_scene;
mod colors;
mod drawable;
mod grid;
mod hierarchy;
mod loader;
mod panels;
#[cfg(test)]
mod property_tests;
mod quick_pick;
mod recent;
mod ruler;
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

fn winit_settings() -> WinitSettings {
    WinitSettings::desktop_app()
}

fn main() {
    let args = Args::parse();

    let viewer = match args.file {
        Some(file) => app::ViewerApp::with_path(&file),
        None => app::ViewerApp::default(),
    };

    App::new()
        .insert_non_send(viewer)
        .insert_resource(ClearColor(Color::srgb_u8(30, 30, 30)))
        .insert_resource(winit_settings())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GDS Viewer".to_string(),
                resolution: WindowResolution::new(1280, 800),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, bevy_renderer::setup)
        .add_systems(
            EguiPrimaryContextPass,
            (app::update_system, bevy_renderer::sync_scene).chain(),
        )
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::winit::UpdateMode;

    #[test]
    fn viewer_uses_reactive_window_updates() {
        let settings = winit_settings();

        assert!(matches!(settings.focused_mode, UpdateMode::Reactive { .. }));
        assert!(matches!(
            settings.unfocused_mode,
            UpdateMode::Reactive { .. }
        ));
    }
}
