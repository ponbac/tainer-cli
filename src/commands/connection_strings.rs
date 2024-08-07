use std::path::Path;

use walkdir::{DirEntry, WalkDir};

use crate::win;

static CONFIG_FILES: [&str; 3] = ["app.config", "web.config", "appsettings.json"];

pub(crate) fn invoke(main: &str, service_bus: &str, root_path: &Path) {
    println!("Walking from {}", root_path.display());
    println!("Setting connection strings to {} and {}", main, service_bus);

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok().filter(is_config_file))
    {
        let path = entry.path();

        let short_path = path
            .to_string_lossy()
            .split_once("ELOS\\")
            .unwrap()
            .1
            .to_string();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => {
                println!("Creating dev appsettings for {}", short_path);
                create_dev_appsettings(path, main, service_bus)
            }
            Some("config") => {
                println!("Appending connection strings for {}", short_path);
                let computer_name = win::get_computer_name();
                append_connection_string(path, &computer_name, main, service_bus)
            }
            _ => println!("Unsupported file type!"),
        }
    }
}

fn create_dev_appsettings(path: &Path, main: &str, service_bus: &str) {
    // check if appsettings.Development.json exists, else read 'path' (which is the appsettings.json)
    let dev_settings_path = path.with_file_name("appsettings.Development.json");
    let content = std::fs::read_to_string(&dev_settings_path).unwrap_or_else(|_| {
        std::fs::read_to_string(path).expect("Could not read appsettings.json")
    });

    // add the new connection strings
    let new_content = content
        .lines()
        .map(|line| match line {
            l if l.contains(r#""ELOS": "#) => {
                format!(r#"    "ELOS": "{main}","#)
            }
            l if l.contains(r#""NServiceBus": "#) => {
                format!(r#"    "NServiceBus": "{service_bus}""#)
            }
            _ => line.to_string(),
        })
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write(dev_settings_path, new_content).expect("Could not write to file");
}

fn append_connection_string(path: &Path, computer_name: &str, main: &str, service_bus: &str) {
    let content = std::fs::read_to_string(path).expect("Could not read file");

    let mut main_replaced = false;
    let mut service_bus_replaced = false;
    let new_content = content
        .lines()
        .map(|line| {
            if line.contains(&format!(r#"name="{}""#, computer_name)) {
                main_replaced = true;
                main_connection_string(computer_name, main)
            } else if line.contains(&format!(r#"name="{}_NSERVICEBUS""#, computer_name)) {
                service_bus_replaced = true;
                service_bus_connection_string(computer_name, service_bus)
            } else if line.contains("</connectionStrings>") {
                let mut new_line = String::new();
                if !main_replaced {
                    new_line.push_str(&main_connection_string(computer_name, main));
                    new_line.push('\n');
                }
                if !service_bus_replaced {
                    new_line.push_str(&service_bus_connection_string(computer_name, service_bus));
                    new_line.push('\n');
                }
                new_line.push_str("  </connectionStrings>");
                new_line
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write(path, new_content).expect("Could not write to file");
}

fn is_config_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| CONFIG_FILES.contains(&s.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn main_connection_string(computer_name: &str, main: &str) -> String {
    format!(
        r#"  <add name="{computer_name}" providerName="System.Data.SqlClient" connectionString="{main}" />"#,
        computer_name = computer_name,
        main = main
    )
}

fn service_bus_connection_string(computer_name: &str, service_bus: &str) -> String {
    format!(
        r#"  <add name="{computer_name}_NSERVICEBUS" providerName="System.Data.SqlClient" connectionString="{service_bus}" />"#,
        computer_name = computer_name,
        service_bus = service_bus
    )
}
