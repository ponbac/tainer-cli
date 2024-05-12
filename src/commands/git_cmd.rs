use std::path::Path;

use console::style;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

pub(crate) fn invoke(cmd: &Vec<String>, root_path: &Path) {
    let git_dirs = WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok().filter(is_git_repo))
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    let (successes, failures): (Vec<_>, Vec<_>) = git_dirs
        .par_iter()
        .map(|path| {
            let status = std::process::Command::new("git")
                .args(cmd)
                .current_dir(path)
                .status()
                .expect("Failed to run git command");

            let module = match path.file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => {
                    let path_str = path.to_string_lossy();

                    // if the path is '.' resolve the current directory
                    if path_str == "." {
                        let current_dir =
                            std::env::current_dir().expect("Failed to get current directory");
                        current_dir
                            .file_name()
                            .unwrap_or(current_dir.as_os_str())
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        path_str.to_string()
                    }
                }
            };
            (status.success(), module.to_string())
        })
        .partition(|(success, _)| *success);

    let joined_cmd = format!("git {}", cmd.join(" "));
    if !successes.is_empty() {
        for (_, module) in successes {
            println!(
                "✅ - executed {} in {}",
                style(&joined_cmd).bold().dim(),
                style(&module).bold()
            );
        }
    }
    if !failures.is_empty() {
        for (_, module) in failures {
            println!(
                "❌ - {} failed in {}",
                style(&joined_cmd).bold().dim(),
                style(&module).bold().red()
            );
        }
    }
}

fn is_git_repo(entry: &DirEntry) -> bool {
    entry.path().join(".git").exists()
}
