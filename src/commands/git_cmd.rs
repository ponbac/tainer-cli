use std::path::PathBuf;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

pub(crate) fn invoke(cmd: &str, root_path: Option<PathBuf>) {
    let git_dirs = WalkDir::new(root_path.unwrap_or(PathBuf::from(".")))
        .into_iter()
        .filter_map(|e| e.ok().filter(is_git_repo))
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    git_dirs.par_iter().for_each(|path| {
        let status = std::process::Command::new("git")
            .args(cmd.split_whitespace())
            .current_dir(path)
            .status()
            .expect("Failed to run git command");

        if status.success() {
            println!("Ran 'git {}' in {}", cmd, path.display());
        } else {
            println!("Failed to run 'git {}' in {}", cmd, path.display());
        }
    });
}

fn is_git_repo(entry: &DirEntry) -> bool {
    entry.path().join(".git").exists()
}
