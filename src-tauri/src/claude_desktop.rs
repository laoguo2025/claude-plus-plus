#[cfg(target_os = "windows")]
use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
const CLAUDE_STORE_APP: &str = r"shell:AppsFolder\Claude_pzs8sxrjxfjjc!Claude";

#[cfg(target_os = "windows")]
pub fn restart() -> Result<(), String> {
    stop_claude_processes()?;
    launch_claude()
}

#[cfg(not(target_os = "windows"))]
pub fn restart() -> Result<(), String> {
    Err("当前只支持在 Windows 上重启 Claude Desktop".to_string())
}

#[cfg(target_os = "windows")]
pub(crate) fn stop_claude_processes() -> Result<(), String> {
    let output = hidden_command("taskkill")
        .args(["/IM", "Claude.exe", "/T", "/F"])
        .output()
        .map_err(|e| format!("关闭 Claude Desktop 失败: {e}"))?;

    if output.status.success() || taskkill_means_not_running(&output.stdout, &output.stderr) {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        Err(if detail.is_empty() {
            "关闭 Claude Desktop 失败".to_string()
        } else {
            format!("关闭 Claude Desktop 失败: {detail}")
        })
    }
}

#[cfg(target_os = "windows")]
fn taskkill_means_not_running(stdout: &[u8], stderr: &[u8]) -> bool {
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    )
    .to_ascii_lowercase();

    text.contains("not found") || text.contains("没有找到") || text.contains("未找到")
}

#[cfg(target_os = "windows")]
pub(crate) fn launch_claude() -> Result<(), String> {
    if let Some(shortcut) = claude_desktop_shortcut() {
        if launch_with_explorer(shortcut.as_os_str()).is_ok() {
            return Ok(());
        }
    }

    launch_with_explorer(CLAUDE_STORE_APP.as_ref())
        .map_err(|e| format!("启动 Claude Desktop 失败: {e}"))
}

#[cfg(target_os = "windows")]
fn claude_desktop_shortcut() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(user_profile) = env::var_os("USERPROFILE") {
        candidates.push(PathBuf::from(user_profile).join("Desktop").join("Claude.lnk"));
    }
    if let Some(public) = env::var_os("PUBLIC") {
        candidates.push(PathBuf::from(public).join("Desktop").join("Claude.lnk"));
    }

    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(target_os = "windows")]
fn launch_with_explorer(target: &std::ffi::OsStr) -> std::io::Result<()> {
    hidden_command("explorer.exe").arg(target).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn hidden_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command
}
