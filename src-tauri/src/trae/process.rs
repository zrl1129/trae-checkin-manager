use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::Result;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn launch_trae(exe_path: &Path, user_data_dir: &str, debug_port: u16) -> Result<u32> {
    let mut cmd = Command::new(exe_path);
    cmd.arg(format!("--remote-debugging-port={}", debug_port));
    cmd.arg(format!("--user-data-dir={}", user_data_dir));

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn()?;
    Ok(child.id())
}

pub fn kill_trae_pid(pid: u32) -> Result<()> {
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/PID", &pid.to_string()]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let _ = cmd.output();
    Ok(())
}

pub fn kill_trae_by_name() -> Result<()> {
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/IM", "TRAE SOLO CN.exe"]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let _ = cmd.output();
    Ok(())
}

pub fn is_process_running(pid: u32) -> bool {
    let mut cmd = Command::new("tasklist");
    cmd.args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("TRAE SOLO CN.exe")
        })
        .unwrap_or(false)
}

pub async fn is_debug_port_open(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/json/version", port);
    reqwest::Client::new()
        .get(&url)
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

pub async fn wait_for_debug_port(port: u16, timeout_ms: u64) -> Result<()> {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if is_debug_port_open(port).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Err(anyhow::anyhow!(
        "等待调试端口超时 (port={})",
        port
    ))
}
