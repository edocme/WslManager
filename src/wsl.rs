use crate::app::{ConfigType, DistroInfo};

fn decode_output(bytes: &[u8]) -> String {
    if bytes.len() < 2 {
        return String::from_utf8_lossy(bytes).to_string();
    }

    let (skip, is_utf16) = if bytes[0] == 0xFF && bytes[1] == 0xFE {
        (2, true)
    } else {
        let null_count = bytes.iter().take(20).enumerate().filter(|(i, _)| i % 2 == 1).filter(|(_, b)| **b == 0).count();
        (0, bytes.len() >= 4 && null_count >= 4)
    };

    if is_utf16 {
        let utf16: Vec<u16> = bytes[skip..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&utf16)
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

fn find_wsl_exe() -> String {
    "wsl.exe".to_string()
}

pub async fn execute_wsl(args: &[&str]) -> Result<String, String> {
    let wsl_path = find_wsl_exe();
    let result = tokio::process::Command::new(&wsl_path)
        .args(args)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .await;

    let output = match result {
        Ok(o) => o,
        Err(e) => {
            return Err(format!("Failed to run wsl: {}", e));
        }
    };

    let stdout = decode_output(&output.stdout);
    let stderr = decode_output(&output.stderr);

    if !output.status.success() {
        let err_lower = stderr.to_lowercase();
        if err_lower.contains("error") || err_lower.contains("failed") {
            return Err(format!("WSL error: {}", stderr.trim()));
        }
    }

    Ok(stdout)
}

pub async fn refresh_distros() -> String {
    match execute_wsl(&["-l", "-v"]).await {
        Ok(output) => {
            if output.trim().is_empty() {
                "ERROR: Empty output from wsl -l -v".to_string()
            } else {
                output
            }
        }
        Err(e) => format!("ERROR: {}", e),
    }
}

pub fn parse_distros(output: &str) -> Vec<DistroInfo> {
    let mut distros = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    if lines.is_empty() {
        return distros;
    }

    for line in &lines[1..] {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let is_default = line.starts_with('*');
            let mut name_parts = Vec::new();
            let mut state = String::new();
            let mut version = 2u8;

            for &p in &parts {
                if p == "*" {
                    continue;
                }
                match p {
                    "Running" | "Stopped" | "Installing" => state = p.to_string(),
                    _ => {
                        if let Ok(v) = p.parse::<u8>() {
                            version = v;
                        } else if state.is_empty() {
                            name_parts.push(p);
                        }
                    }
                }
            }

            let name = name_parts.join(" ");
            if !name.is_empty() && !state.is_empty() {
                distros.push(DistroInfo {
                    name,
                    is_default,
                    state,
                    wsl_version: version,
                });
            }
        }
    }

    distros
}

pub async fn start_distro(name: String) -> Result<String, String> {
    let wsl_path = find_wsl_exe();
    tokio::process::Command::new(&wsl_path)
        .args(&["-d", &name, "-e", "echo"])
        .creation_flags(0x08000000)
        .output()
        .await
        .map_err(|e| format!("Failed to start wsl: {}", e))?;
    Ok(format!(
        "[{}] Distro {} started",
        chrono::Local::now().format("%H:%M:%S"),
        name
    ))
}

pub async fn stop_distro(name: String) -> Result<String, String> {
    execute_wsl(&["--terminate", &name]).await?;
    Ok(format!(
        "[{}] Distro {} stopped",
        chrono::Local::now().format("%H:%M:%S"),
        name
    ))
}

pub async fn restart_distro(name: String) -> Result<String, String> {
    execute_wsl(&["--terminate", &name]).await?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let wsl_path = find_wsl_exe();
    tokio::process::Command::new(&wsl_path)
        .args(&["-d", &name, "-e", "echo"])
        .creation_flags(0x08000000)
        .output()
        .await
        .map_err(|e| format!("Failed to start wsl: {}", e))?;
    Ok(format!(
        "[{}] Distro {} restarted",
        chrono::Local::now().format("%H:%M:%S"),
        name
    ))
}

pub async fn delete_distro(name: String) -> Result<String, String> {
    execute_wsl(&["--unregister", &name]).await?;
    Ok(format!(
        "[{}] Distro {} deleted",
        chrono::Local::now().format("%H:%M:%S"),
        name
    ))
}

pub async fn set_default_distro(name: String) -> Result<String, String> {
    execute_wsl(&["-s", &name]).await?;
    Ok(format!(
        "[{}] {} set as default",
        chrono::Local::now().format("%H:%M:%S"),
        name
    ))
}

pub async fn rename_distro(old_name: String, new_name: String) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let export_dir = temp_dir.join(format!("wsl-rename-{}", old_name));
    let export_path = export_dir.join("export.tar");
    let export_path_str = export_path.to_string_lossy().to_string();

    tokio::fs::create_dir_all(&export_dir).await.ok();

    execute_wsl(&["--export", &old_name, &export_path_str]).await?;
    execute_wsl(&["--unregister", &old_name]).await?;

    let install_dir = format!("C:\\WSL\\{}", new_name);
    tokio::fs::create_dir_all(&install_dir).await.ok();
    execute_wsl(&["--import", &new_name, &install_dir, &export_path_str]).await?;

    let _ = tokio::fs::remove_dir_all(&export_dir).await;

    Ok(format!(
        "[{}] Renamed {} -> {}",
        chrono::Local::now().format("%H:%M:%S"),
        old_name,
        new_name
    ))
}

pub async fn export_distro_to(name: String, path: String) -> Result<String, String> {
    let dir = std::path::Path::new(&path);
    if let Some(parent) = dir.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    execute_wsl(&["--export", &name, &path]).await?;
    Ok(format!(
        "[{}] Distro {} exported to {}",
        chrono::Local::now().format("%H:%M:%S"),
        name,
        path
    ))
}

pub async fn import_distro_to(name: String, tar_path: String, install_dir: String) -> Result<String, String> {
    if !install_dir.is_empty() {
        tokio::fs::create_dir_all(&install_dir).await.ok();
        execute_wsl(&["--import", &name, &install_dir, &tar_path]).await?;
    } else {
        let default_dir = format!("C:\\WSL\\{}", name);
        tokio::fs::create_dir_all(&default_dir).await.ok();
        execute_wsl(&["--import", &name, &default_dir, &tar_path]).await?;
    }
    Ok(format!(
        "[{}] Distro {} imported from {}",
        chrono::Local::now().format("%H:%M:%S"),
        name,
        tar_path
    ))
}

pub async fn install_distro_from_store(name: String) -> Result<String, String> {
    execute_wsl(&["--install", "-d", &name]).await?;
    Ok(format!(
        "[{}] Installing {} from Microsoft Store...",
        chrono::Local::now().format("%H:%M:%S"),
        name
    ))
}

pub async fn set_wsl_version(name: String, version: u8) -> Result<String, String> {
    execute_wsl(&["--set-version", &name, &version.to_string()]).await?;
    Ok(format!(
        "[{}] {} switched to WSL{}",
        chrono::Local::now().format("%H:%M:%S"),
        name,
        version
    ))
}

pub async fn open_terminal(name: String) -> Result<String, String> {
    let cmd_path = "C:\\Windows\\System32\\cmd.exe";
    let result = tokio::process::Command::new(cmd_path)
        .args(&[
            "/C",
            "start",
            "",
            "cmd.exe",
            "/K",
            "wsl.exe",
            "-d",
            &name,
        ])
        .creation_flags(0x08000000)
        .output()
        .await;

    match result {
        Ok(_) => Ok(format!(
            "[{}] Terminal opened for {}",
            chrono::Local::now().format("%H:%M:%S"),
            name
        )),
        Err(e) => Err(format!("Failed to open terminal: {}", e)),
    }
}

pub async fn open_explorer(name: String) -> Result<String, String> {
    let wsl_path = format!("\\\\wsl$\\{}", name);
    tokio::process::Command::new("explorer.exe")
        .arg(&wsl_path)
        .creation_flags(0x08000000)
        .output()
        .await
        .map_err(|e| format!("Failed to open explorer: {}", e))?;
    Ok(format!(
        "[{}] Explorer opened for {}",
        chrono::Local::now().format("%H:%M:%S"),
        name
    ))
}

pub async fn open_vscode(name: String) -> Result<String, String> {
    let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
    let code_paths = [
        format!(
            "{}\\AppData\\Local\\Programs\\Microsoft VS Code\\bin\\code.cmd",
            userprofile
        ),
        "code".to_string(),
    ];
    let mut last_err = String::new();
    for code_path in &code_paths {
        let result = tokio::process::Command::new(code_path)
            .args(&[
                "--remote",
                &format!("wsl+{}", name),
                "/",
            ])
            .creation_flags(0x08000000)
            .output()
            .await;
        match result {
            Ok(_) => return Ok(format!(
                "[{}] VS Code opened for {}",
                chrono::Local::now().format("%H:%M:%S"),
                name
            )),
            Err(e) => last_err = e.to_string(),
        }
    }
    Err(format!("Failed to open VS Code: {}. Is it installed?", last_err))
}

fn find_windows_home() -> String {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        return userprofile;
    }
    if let Ok(homedrive) = std::env::var("HOMEDRIVE") {
        if let Ok(homepath) = std::env::var("HOMEPATH") {
            return format!("{}{}", homedrive, homepath);
        }
    }
    String::new()
}

pub async fn load_config(config_type: ConfigType) -> String {
    match config_type {
        ConfigType::WslConfig => {
            let home = find_windows_home();
            let config_path = format!("{}\\.wslconfig", home);
            match tokio::fs::read_to_string(&config_path).await {
                Ok(content) => content,
                Err(_) => {
                    "[wsl2]\n# memory=4GB\n# processors=4\n# swap=2GB\n# nestedVirtualization=true\n# firewall=true\n# autoProxy=true\n"
                        .to_string()
                }
            }
        }
        ConfigType::WslConf => unreachable!("Use load_wsl_conf instead"),
    }
}

pub async fn load_wsl_conf(distro_name: String) -> String {
    match execute_wsl(&[
        "-d",
        &distro_name,
        "--",
        "cat",
        "/etc/wsl.conf",
    ])
    .await
    {
        Ok(content) => content,
        Err(_) => {
            "[boot]\nsystemd=true\n\n[automount]\nenabled=true\noptions=\"metadata,umask=22,fmask=11\"\n\n[network]\ngenerateHosts=true\ngenerateResolvConf=true\n\n[interop]\nenabled=true\nappendWindowsPath=true\n"
                .to_string()
        }
    }
}

pub async fn save_config(
    config_type: ConfigType,
    distro_name: Option<String>,
    content: String,
) -> Result<String, String> {
    match config_type {
        ConfigType::WslConfig => {
            let home = find_windows_home();
            let config_path = format!("{}\\.wslconfig", home);
            tokio::fs::write(&config_path, &content)
                .await
                .map_err(|e| format!("Failed to save .wslconfig: {}", e))?;
            Ok(format!(
                "[{}] .wslconfig saved to {}",
                chrono::Local::now().format("%H:%M:%S"),
                config_path
            ))
        }
        ConfigType::WslConf => {
            let name = distro_name.ok_or("No distro selected")?;
            let temp_dir = std::env::temp_dir();
            let tmp_win = temp_dir.join("wsl-tmp-wsl.conf");
            let tmp_linux = "/tmp/wsl-tmp-wsl.conf";

            if let Err(e) = tokio::fs::write(&tmp_win, &content).await {
                return Err(format!("Failed to write temp file: {}", e));
            }

            execute_wsl(&[
                "-d",
                &name,
                "--",
                "sudo",
                "cp",
                tmp_linux,
                "/etc/wsl.conf",
            ])
            .await?;

            let _ = tokio::fs::remove_file(&tmp_win).await;

            Ok(format!(
                "[{}] wsl.conf saved for {}",
                chrono::Local::now().format("%H:%M:%S"),
                name
            ))
        }
    }
}
