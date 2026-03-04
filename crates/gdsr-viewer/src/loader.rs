use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use gdsr::Library;

/// Opens a file dialog on the current thread (required by macOS), then parses
/// the selected GDS file on a background thread. Returns a receiver that will
/// deliver the result, or `None` if the user cancelled the dialog.
pub fn load_file_dialog() -> Option<(PathBuf, mpsc::Receiver<Result<Library, String>>)> {
    let path = rfd::FileDialog::new()
        .add_filter("GDS files", &["gds", "gds2", "gdsii"])
        .pick_file()?;

    let (tx, rx) = mpsc::channel();
    let path_clone = path.clone();

    thread::spawn(move || {
        let result = Library::read_file(&path_clone, None).map_err(|e| e.to_string());
        let _ = tx.send(result);
    });

    Some((path, rx))
}
