use std::path::PathBuf;

use console::style;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

pub(crate) fn invoke(cmd: &Vec<String>, root_path: Option<PathBuf>) {
    let git_dirs = WalkDir::new(root_path.unwrap_or(PathBuf::from(".")))
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

            let module = path.file_name().unwrap().to_str().unwrap();
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
