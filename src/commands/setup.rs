use std::path::Path;

use dialoguer::Input;

use crate::{
    commands, git,
    win::{self},
};

pub(crate) async fn invoke(
    main_connection_string: &Option<String>,
    service_bus_connection_string: &Option<String>,
    root_path: &Path,
) {
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
    let main_connection_string = if let Some(main) = main_connection_string {
        main.clone()
    } else {
        Input::new()
            .with_prompt("Main connection string (dbEnvirotainerELOS)")
            .interact()
            .expect("Failed to get main connection string")
    };
    let service_bus_connection_string = if let Some(service_bus) = service_bus_connection_string {
        service_bus.clone()
    } else {
        Input::new()
            .with_prompt("Service bus connection string (EnvirotainerNServiceBus)")
            .interact()
            .expect("Failed to get service bus connection string")
    };
    commands::connection_strings::invoke(
        &main_connection_string,
        &service_bus_connection_string,
        root_path,
    );
    // Fix applicationhost
    commands::application_host::invoke(root_path);
    // Fix Web API appsettings
    commands::web_api::invoke(root_path);
    // Create a new user in database
    let user_name: String = Input::new()
        .with_prompt("First and last name of the new user (e.g. Pontus Backman)")
        .interact()
        .expect("Failed to get user name");
    let user_email: String = Input::new()
        .with_prompt("Email of the new user (e.g. pontus.backman@spinit.se)")
        .interact()
        .expect("Failed to get user email");
    commands::create_user::invoke(&user_name, &user_email, &main_connection_string).await;

    println!("Setup command has finished.");
}
