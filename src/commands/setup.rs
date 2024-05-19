use std::path::Path;

use crate::{git, win};

pub(crate) fn invoke(root_path: &Path) {
    println!("Running setup command");
    git::init_submodules(root_path);

    // check if MSMQ is installed
    println!("Checking if MSMQ is enabled...");
    let msmq_enabled = win::msmq_enabled();
    if !msmq_enabled {
        println!("MSMQ is not enabled, enabling it now.");
        match win::enable_msmq() {
            Ok(_) => println!("MSMQ has been enabled successfully."),
            Err(e) => {
                eprintln!("Failed to enable MSMQ: {}", e);
                return;
            }
        }
    }

    // execute ELOSQueues.ps1 with argument -account <account_name>
    let account_name = "pbac@spinit.local";
    match win::execute_ps1(
        root_path
            .join("ELOSQueues.ps1")
            .to_str()
            .expect("ELOSQueues.ps1 path is not valid"),
        &["-account", account_name],
    ) {
        Ok(_) => println!("ELOSQueues.ps1 has been executed successfully."),
        Err(e) => eprintln!("Failed to execute ELOSQueues.ps1: {}", e),
    }
}
