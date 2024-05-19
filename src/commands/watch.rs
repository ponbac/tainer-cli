use std::{path::Path, thread, time::Duration};

use notify::{Event, EventKind, RecursiveMode, Watcher};

pub fn invoke(root_path: &Path) -> notify::Result<()> {
    // Automatically select the best implementation for your platform.
    let mut watcher = notify::recommended_watcher(|res| match res {
        Ok(event) => {
            if let Event {
                kind: EventKind::Modify(_),
                paths,
                ..
            } = event
            {
                for path in paths {
                    println!("modified: {:?}", path);
                }
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    println!("Watching...");
    watcher.watch(root_path, RecursiveMode::Recursive)?;

    loop {
        thread::sleep(Duration::from_secs(10));
    }

    Ok(())
}
