use std::path::PathBuf;

const EXE_NAME: &str = "TRAE SOLO CN.exe";

pub fn find_trae_exe() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("TRAE_EXE_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    for path in common_paths() {
        if path.exists() {
            return Some(path);
        }
    }

    find_running_trae_path()
}

fn common_paths() -> Vec<PathBuf> {
    let mut paths = vec![];

    let bases = [
        r"C:\Program Files",
        r"C:\Program Files (x86)",
        r"D:\Program Files",
        r"D:\Software",
        r"E:\Software",
        r"D:\",
        r"E:\",
    ];

    for base in &bases {
        paths.push(PathBuf::from(base).join("TRAE SOLO CN").join(EXE_NAME));
        paths.push(PathBuf::from(base).join("trae").join(EXE_NAME));
    }

    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        paths.push(PathBuf::from(&local_appdata).join("Programs").join("TRAE SOLO CN").join(EXE_NAME));
        paths.push(PathBuf::from(&local_appdata).join("TRAE SOLO CN").join(EXE_NAME));
        paths.push(PathBuf::from(&local_appdata).join("Programs").join("trae").join(EXE_NAME));
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        paths.push(PathBuf::from(&appdata).join("TRAE SOLO CN").join(EXE_NAME));
    }

    paths
}

fn find_running_trae_path() -> Option<PathBuf> {
    let script = "Get-CimInstance Win32_Process | Where-Object { $_.Name -eq 'TRAE SOLO CN.exe' } | Select-Object -ExpandProperty ExecutablePath -First 1";

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout.lines().next()?.trim().to_string();

    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}
