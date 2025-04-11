use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

// Watches the "shaders" directory for file modifications and notifies via a channel
pub fn watch_shader_files(sender: Sender<()>) -> notify::Result<RecommendedWatcher> {
    // Create a new file watcher
    let mut watcher = RecommendedWatcher::new(
        move |result: Result<notify::Event, notify::Error>| {
            if let Ok(event) = result {
                // Check if the event is a file modification
                if event.kind.is_modify() {
                    // Add a small delay to avoid rapid notifications
                    thread::sleep(Duration::from_millis(50));
                    let _ = sender.send(()); // Notify the main thread
                }
            }
        },
        Config::default(),
    )?;

    // Watch the "shaders" directory recursively
    watcher.watch(Path::new("shaders"), RecursiveMode::Recursive)?;
    Ok(watcher)
}
