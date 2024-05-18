use std::path::Path;

use crate::git::init_submodules;

pub(crate) fn invoke(root_path: &Path) {
    println!("Running setup command");
    init_submodules(root_path);

    // execute ELOSQueues.ps1 with argument -account <account_name>
    let account_name = "pbac@spinit.local";
    let queues_status = std::process::Command::new("powershell")
        .args([
            "-File",
            root_path
                .join("ELOSQueues.ps1")
                .to_str()
                .expect("ELOSQueues.ps1 path is not valid"),
            "-account",
            account_name,
        ])
        .status()
        .expect("Failed to run ELOSQueues.ps1");
}
