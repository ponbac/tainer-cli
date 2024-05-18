use std::path::Path;

use console::style;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::git::find_git_modules;

pub(crate) fn invoke(cmd: &Vec<String>, root_path: &Path) {
    let git_modules = find_git_modules(root_path);
    let (successes, failures): (Vec<_>, Vec<_>) = git_modules
        .par_iter()
        .map(|module| {
            let status = std::process::Command::new("git")
                .args(cmd)
                .current_dir(&module.path)
                .status()
                .expect("Failed to run git command");

            (status.success(), &module.name)
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
