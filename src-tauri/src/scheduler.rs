use std::process::Command;

use anyhow::Result;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const TASK_NAME: &str = "TraeCheckinManager_DailyCheckin";

pub fn create_task(exe_path: &str, hour: u32, minute: u32) -> Result<()> {
    let time = format!("{:02}:{:02}", hour, minute);
    let exe_with_args = format!("\"{}\" --auto-checkin", exe_path);

    let mut cmd = Command::new("schtasks");
    cmd.args([
        "/Create",
        "/TN",
        TASK_NAME,
        "/TR",
        &exe_with_args,
        "/SC",
        "DAILY",
        "/ST",
        &time,
        "/F",
    ]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "schtasks 创建失败: {} {}",
            stdout.trim(),
            stderr.trim()
        ));
    }

    Ok(())
}

pub fn remove_task() -> Result<()> {
    let mut cmd = Command::new("schtasks");
    cmd.args(["/Delete", "/TN", TASK_NAME, "/F"]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("找不到") || stderr.contains("cannot find") {
            return Ok(());
        }
        return Err(anyhow::anyhow!(
            "schtasks 删除失败: {} {}",
            stdout.trim(),
            stderr.trim()
        ));
    }

    Ok(())
}

pub fn task_exists() -> Result<bool> {
    let mut cmd = Command::new("schtasks");
    cmd.args(["/Query", "/TN", TASK_NAME, "/FO", "CSV", "/NH"]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains(TASK_NAME))
    } else {
        Ok(false)
    }
}
