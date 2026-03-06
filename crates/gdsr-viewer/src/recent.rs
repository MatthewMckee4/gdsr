use std::path::{Path, PathBuf};

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
}
