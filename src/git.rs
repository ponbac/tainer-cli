use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct GitModule {
    pub path: PathBuf,
    pub name: String,
}

pub fn find_git_modules(root_path: &Path) -> Vec<GitModule> {
    WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok().filter(is_git_repo))
        .map(|e| match e.path().file_name() {
            Some(name) => GitModule {
                path: e.path().to_path_buf(),
                name: name.to_string_lossy().to_string(),
            },
            None => {
                let path_str = e.path().to_string_lossy();
                if path_str == "." {
                    // resolve the current directory
                    let current_dir =
                        std::env::current_dir().expect("Failed to get current directory");
                    GitModule {
                        path: current_dir.clone(),
                        name: current_dir
                            .file_name()
                            .unwrap_or(current_dir.as_os_str())
                            .to_string_lossy()
                            .to_string(),
                    }
                } else {
                    GitModule {
                        path: e.path().to_path_buf(),
                        name: path_str.to_string(),
                    }
                }
            }
        })
        .collect::<Vec<_>>()
}

pub fn init_submodules(root_path: &Path) {
    std::process::Command::new("git")
        .args(["submodule", "update", "--init", "--recursive"])
        .current_dir(root_path)
        .status()
        .expect("Failed to run git command");

    let git_modules = find_git_modules(root_path);
    for module in git_modules {
        let fetch_status = std::process::Command::new("git")
            .args(["fetch"])
            .current_dir(&module.path)
            .status()
            .expect("Failed to run git command");
        let pull_status = std::process::Command::new("git")
            .args(["pull", "--ff-only", "origin", "main"])
            .current_dir(&module.path)
            .status()
            .expect("Failed to run git command");

        if fetch_status.success() && pull_status.success() {
            println!("✅ - initialized and updated {}", module.name);
        } else {
            println!("❌ - failed to initialize and update {}", module.name);
        }
    }
}

fn is_git_repo(entry: &DirEntry) -> bool {
    entry.path().join(".git").exists()
}
