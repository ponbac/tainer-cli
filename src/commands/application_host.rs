use std::path::Path;

pub(crate) fn invoke(root_path: &Path) {
    let apphost_path = root_path.join(".vs/Envirotainer.ELOS/config/applicationhost.config");

    if !apphost_path.exists() {
        eprintln!("No applicationhost.config found at {:?}", apphost_path);
        return;
    }

    let file_content =
        std::fs::read_to_string(&apphost_path).expect("Failed to read applicationhost.config");

    let mut inside_auth_section = false;
    let new_content = file_content
        .lines()
        .map(|line| {
            if line.contains(r#"<sectionGroup name="authentication">"#) {
                inside_auth_section = true;
            } else if inside_auth_section && line.contains("</sectionGroup>") {
                inside_auth_section = false;
            } else if inside_auth_section && line.contains(r#"overrideModeDefault="Deny""#) {
                return line.replace(r#""Deny""#, r#""Allow""#);
            } else if line.contains(r#"<windowsAuthentication enabled="false">"#) {
                return line.replace("false", "true");
            }

            line.to_string()
        })
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write(&apphost_path, new_content).expect("Failed to write to applicationhost.config");
    println!("Updated applicationhost.config at {:?}", apphost_path);
}
