use std::path::{Path, PathBuf};

use crate::quick_pick::QuickPickItem;

const MAX_ENTRIES: usize = 10;

/// Persists a list of recently opened file paths.
#[derive(Default)]
pub struct RecentProjects {
    paths: Vec<PathBuf>,
}

impl RecentProjects {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let paths: Vec<PathBuf> = contents
            .lines()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect();
        Self { paths }
    }

    pub fn save(&self) {
        let Some(path) = config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let contents: String = self
            .paths
            .iter()
            .filter_map(|p| p.to_str())
            .collect::<Vec<_>>()
            .join("\n");
        let _ = std::fs::write(&path, contents);
    }

    pub fn add(&mut self, path: &Path) {
        self.paths.retain(|p| p != path);
        self.paths.insert(0, path.to_path_buf());
        self.paths.truncate(MAX_ENTRIES);
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

/// An item in the recent-projects picker, showing the file name prominently
/// and the abbreviated path below it.
pub struct RecentProjectItem {
    pub name: String,
    pub display_path: String,
    pub path: PathBuf,
}

impl RecentProjectItem {
    pub fn from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let display_path = abbreviate_home(path);

        Self {
            name,
            display_path,
            path: path.to_path_buf(),
        }
    }
}

impl QuickPickItem for RecentProjectItem {
    fn filter_text(&self) -> &str {
        &self.name
    }

    fn ui(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(&self.name);
            ui.label(egui::RichText::new(&self.display_path).small().weak());
        });
    }
}

/// Replaces the home directory prefix with `~` for shorter display.
fn abbreviate_home(path: &Path) -> String {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok();
    if let Some(home) = home {
        if let Some(rest) = path.to_str().and_then(|p| p.strip_prefix(&home)) {
            return format!("~{rest}");
        }
    }
    path.display().to_string()
}

fn config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs_config_macos()
    }
    #[cfg(not(target_os = "macos"))]
    {
        dirs_config_other()
    }
}

#[cfg(target_os = "macos")]
fn dirs_config_macos() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join("Library/Application Support/gdsr-viewer/recent.txt"))
}

#[cfg(not(target_os = "macos"))]
fn dirs_config_other() -> Option<PathBuf> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("APPDATA").map(PathBuf::from))
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok()?;
    Some(config_dir.join("gdsr-viewer/recent.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_deduplicates_and_prepends() {
        let mut rp = RecentProjects::default();
        rp.add(Path::new("/a"));
        rp.add(Path::new("/b"));
        rp.add(Path::new("/a"));
        insta::assert_debug_snapshot!(rp.paths, @r#"
        [
            "/a",
            "/b",
        ]
        "#);
    }

    #[test]
    fn add_truncates_to_max() {
        let mut rp = RecentProjects::default();
        for i in 0..15 {
            rp.add(Path::new(&format!("/file{i}")));
        }
        assert_eq!(rp.paths.len(), MAX_ENTRIES);
        assert_eq!(rp.paths[0], PathBuf::from("/file14"));
    }

    #[test]
    fn config_path_returns_some() {
        assert!(config_path().is_some());
    }

    fn home_dir() -> String {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap()
    }

    #[test]
    fn abbreviate_home_replaces_prefix() {
        let home = home_dir();
        let path_str = format!("{home}/Documents/test.gds");
        let path = Path::new(&path_str);
        assert_eq!(abbreviate_home(path), "~/Documents/test.gds");
    }

    #[test]
    fn abbreviate_home_leaves_non_home_paths() {
        insta::assert_snapshot!(abbreviate_home(Path::new("/tmp/test.gds")), @"/tmp/test.gds");
    }

    #[test]
    fn recent_project_item_from_path() {
        let item = RecentProjectItem::from_path(Path::new("/some/dir/chip.gds"));
        insta::assert_snapshot!(item.name, @"chip.gds");
        insta::assert_snapshot!(item.display_path, @"/some/dir/chip.gds");
    }

    #[test]
    fn recent_project_item_from_home_path() {
        let home = home_dir();
        let path_str = format!("{home}/projects/chip.gds");
        let item = RecentProjectItem::from_path(Path::new(&path_str));
        insta::assert_snapshot!(item.name, @"chip.gds");
        assert_eq!(item.display_path, "~/projects/chip.gds");
    }
}
