use std::process::Command;

pub fn msmq_enabled() -> bool {
    if !is_admin_shell() {
        panic!("Please run this command in an admin shell.");
    }

    let powershell_command = r#"
    $dismOutput = (dism /online /get-features /format:table | Select-String -Pattern "MSMQ")

    if ($dismOutput -match "MSMQ.*Enable") {
        Write-Output "MSMQ is enabled."
    } else {
        Write-Output "MSMQ is not enabled."
    }
    "#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", powershell_command])
        .output()
        .expect("Failed to execute PowerShell command")
        .stdout;

    String::from_utf8(output)
        .expect("Failed to convert output to string")
        .contains("MSMQ is enabled.")
}

pub fn enable_msmq() -> Result<(), String> {
    if !is_admin_shell() {
        return Err("Please run this command in an admin shell.".to_string());
    }

    // let enable_msmq_command = r#"
    // Enable-WindowsOptionalFeature -Online -FeatureName MSMQ-Server -All -NoRestart
    // "#;

    // need to test this!
    let enable_msmq_command = r#"
    $features = @(
        'MSMQ-Server',
        'MSMQ-Services',
        'MSMQ-DCOMProxy',
        'MSMQ-ADIntegration',
        'MSMQ-HTTP',
        'MSMQ-Multicast',
        'MSMQ-Triggers',
        'MSMQ-RoutingServer'
    )
    foreach ($feature in $features) {
        Enable-WindowsOptionalFeature -Online -FeatureName $feature -All -NoRestart
    }
    "#;

    let enable_status = Command::new("powershell")
        .args(["-NoProfile", "-Command", enable_msmq_command])
        .status()
        .expect("Failed to execute PowerShell command");

    if enable_status.success() {
        Ok(())
    } else {
        Err("Failed to enable MSMQ.".to_string())
    }
}

pub fn execute_ps1(script_path: &str, args: &[&str]) -> Result<(), String> {
    let script_args: Vec<&str> = vec!["-File", script_path];
    let script_args = script_args
        .iter()
        .chain(args.iter())
        .cloned()
        .collect::<Vec<&str>>();

    let script_status = Command::new("powershell")
        .args(script_args)
        .status()
        .expect("Failed to execute PowerShell command");

    if script_status.success() {
        Ok(())
    } else {
        Err("Failed to execute PowerShell script.".to_string())
    }
}

pub fn get_computer_name() -> String {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "Write-Output $env:COMPUTERNAME"])
        .output()
        .expect("Failed to execute PowerShell command")
        .stdout;

    String::from_utf8(output)
        .expect("Failed to convert output to string")
        .trim()
        .to_string()
}

pub fn get_account_name() -> String {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "Write-Output $env:USERNAME"])
        .output()
        .expect("Failed to execute PowerShell command")
        .stdout;

    String::from_utf8(output)
        .expect("Failed to convert output to string")
        .trim()
        .to_string()
}

fn is_admin_shell() -> bool {
    let powershell_command = r#"
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if ($isAdmin) {
        Write-Output "Admin shell"
    } else {
        Write-Output "Non-admin shell"
    }
    "#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", powershell_command])
        .output()
        .expect("Failed to execute PowerShell command");

    String::from_utf8(output.stdout)
        .expect("Failed to convert output to string")
        .contains("Admin shell")
}
