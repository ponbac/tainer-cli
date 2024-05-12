use std::path::Path;

pub(crate) fn invoke(root_path: &Path) {
    let app_settings_path = root_path
        .join("Web.Api/Envirotainer.ELOS.Web.Api")
        .join("appsettings.Development.json");

    if !app_settings_path.exists() {
        eprintln!(
            "No appsettings.Development.json found at {:?}",
            app_settings_path
        );
        return;
    }

    let content = std::fs::read_to_string(app_settings_path.clone()).expect("Could not read file");

    let mut inside_azure_ad_block = false;
    let new_content = content
            .lines()
            .map(|line| {
                if line.contains(r#""AzureAd": {"#) {
                    inside_azure_ad_block = true;
                } else if inside_azure_ad_block && line.contains('}') {
                    inside_azure_ad_block = false;
                } else if inside_azure_ad_block {
                    if line.contains(r#""ClientId": ""#) {
                        return r#"    "ClientId": "aeaa2c3a-06e7-455c-bc79-717667bc55d6","#
                            .to_string();
                    } else if line.contains("ClientSecret") {
                        return "".to_string();
                    } else if line.contains(r#""Authority": ""#) {
                        return r#"    "Authority": "https://login.microsoftonline.com/d89ef75c-38db-4904-9d78-b872502ca145/v2.0/""#
                            .to_string();
                    }
                }

                line.to_string()
            })
            .collect::<Vec<String>>()
            .join("\n");

    std::fs::write(app_settings_path.clone(), new_content).expect("Could not write to file");
    println!(
        "Updated appsettings.Development.json at {:?}",
        app_settings_path
    );
}
