use std::path::Path;

use walkdir::{DirEntry, WalkDir};

static CONFIG_FILES: [&str; 3] = ["app.config", "web.config", "appsettings.json"];

pub(crate) fn invoke(computer_name: &str, main: &str, service_bus: &str, root_path: &Path) {
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
                append_connection_string(path, computer_name, main, service_bus)
            }
            _ => println!("Unsupported file type!"),
        }
    }
}

fn create_dev_appsettings(path: &Path, main: &str, service_bus: &str) {
    let content = std::fs::read_to_string(path).expect("Could not read file");

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

    let dev_path = path.with_file_name("appsettings.Development.json");
    std::fs::write(dev_path, new_content).expect("Could not write to file");
}

fn append_connection_string(path: &Path, computer_name: &str, main: &str, service_bus: &str) {
    let content = std::fs::read_to_string(path).expect("Could not read file");
    let new_content = content.replace(
        "</connectionStrings>",
        &format!(
            r#"  <add name="{computer_name}" providerName="System.Data.SqlClient" connectionString="{main}" />
    <add name="{computer_name}_NSERVICEBUS" providerName="System.Data.SqlClient" connectionString="{service_bus}" />
  </connectionStrings>"#
        ),
    );

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
