use std::path::Path;

use dialoguer::Input;

use crate::{
    commands, git,
    win::{self},
};

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
    // let account_name: String = Input::new()
    //     .with_prompt("Enter the name of your account (e.g. nlin@spinit.local)")
    //     .interact()
    //     .expect("Failed to get account name");
    let account_name = win::get_account_name();
    println!("Account name: {}", account_name);
    match win::execute_ps1(
        root_path
            .join("ELOSQueues.ps1")
            .to_str()
            .expect("ELOSQueues.ps1 path is not valid"),
        &["-account", &account_name],
    ) {
        Ok(_) => println!("ELOSQueues.ps1 has been executed successfully."),
        Err(e) => eprintln!("Failed to execute ELOSQueues.ps1: {}", e),
    }

    // Set up connection strings
    let computer_name = win::get_computer_name();
    println!("Computer name: {}", computer_name);
    let main_connection_string: String = Input::new()
        .with_prompt("Main connection string (dbEnvirotainerELOS)")
        .interact()
        .expect("Failed to get main connection string");
    let service_bus_connection_string: String = Input::new()
        .with_prompt("Service bus connection string (EnvirotainerNServiceBus)")
        .interact()
        .expect("Failed to get service bus connection string");

    commands::connection_strings::invoke(
        &computer_name,
        &main_connection_string,
        &service_bus_connection_string,
        root_path,
    );
    // Fix applicationhost
    commands::application_host::invoke(root_path);
    // Fix Web API appsettings
    commands::web_api::invoke(root_path);

    println!("Setup command has finished.");
}
