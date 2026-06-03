#[cfg(target_os = "windows")]
use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
use crate::constants::CLAUDE_STORE_APP_ID;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
const GRACEFUL_STOP_TIMEOUT: Duration = Duration::from_secs(8);

#[cfg(target_os = "windows")]
pub fn restart() -> Result<(), String> {
    stop_claude_processes()?;
    launch_claude()
}

#[cfg(target_os = "windows")]
pub fn is_running() -> bool {
    let Ok(output) = hidden_output_command("tasklist")
        .args(["/FI", "IMAGENAME eq Claude.exe"])
        .output()
    else {
        return false;
    };
    tasklist_contains_claude(&output.stdout) || tasklist_contains_claude(&output.stderr)
}

#[cfg(not(target_os = "windows"))]
pub fn restart() -> Result<(), String> {
    Err("当前只支持在 Windows 上重启 Claude Desktop".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn is_running() -> bool {
    false
}

#[cfg(target_os = "windows")]
pub(crate) fn stop_claude_processes() -> Result<(), String> {
    let graceful = run_taskkill(false)?;
    if taskkill_succeeded_or_not_running(&graceful.stdout, &graceful.stderr, graceful.status.code())
    {
        return Ok(());
    }

    let forced = run_taskkill(true)?;
    if taskkill_succeeded_or_not_running(&forced.stdout, &forced.stderr, forced.status.code()) {
        Ok(())
    } else {
        Err(taskkill_error_message(&forced.stdout, &forced.stderr))
    }
}

#[cfg(target_os = "windows")]
fn run_taskkill(force: bool) -> Result<std::process::Output, String> {
    let mut command = hidden_command("taskkill");
    command.args(["/IM", "Claude.exe", "/T"]);
    if force {
        command.arg("/F");
        return command
            .output()
            .map_err(|e| format!("关闭 Claude Desktop 失败: {e}"));
    }

    run_with_timeout(command, GRACEFUL_STOP_TIMEOUT)
        .map_err(|e| format!("关闭 Claude Desktop 失败: {e}"))
}

#[cfg(target_os = "windows")]
fn run_with_timeout(
    mut command: Command,
    timeout: Duration,
) -> std::io::Result<std::process::Output> {
    let mut child = command.spawn()?;
    let started = Instant::now();
    loop {
        if let Some(_status) = child.try_wait()? {
            return child.wait_with_output();
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            return child.wait_with_output();
        }
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(target_os = "windows")]
fn taskkill_succeeded_or_not_running(
    stdout: &[u8],
    stderr: &[u8],
    status_code: Option<i32>,
) -> bool {
    status_code == Some(0) || status_code == Some(128) || taskkill_means_not_running(stdout, stderr)
}

#[cfg(target_os = "windows")]
fn taskkill_error_message(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    if detail.is_empty() {
        "关闭 Claude Desktop 失败".to_string()
    } else {
        format!("关闭 Claude Desktop 失败: {detail}")
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
fn tasklist_contains_claude(output: &[u8]) -> bool {
    String::from_utf8_lossy(output)
        .to_ascii_lowercase()
        .contains("claude.exe")
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn taskkill_not_running_text_is_success() {
        assert!(taskkill_succeeded_or_not_running(
            b"ERROR: The process \"Claude.exe\" not found.",
            b"",
            Some(128)
        ));
        assert!(taskkill_succeeded_or_not_running(
            "错误: 没有找到进程 Claude.exe".as_bytes(),
            b"",
            None
        ));
    }

    #[test]
    fn tasklist_detects_claude_process_name() {
        assert!(tasklist_contains_claude(
            b"Claude.exe                    1234 Console"
        ));
        assert!(!tasklist_contains_claude(
            b"INFO: No tasks are running which match the specified criteria."
        ));
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn launch_claude() -> Result<(), String> {
    if let Some(shortcut) = claude_desktop_shortcut() {
        if launch_with_explorer(shortcut.as_os_str()).is_ok() {
            return Ok(());
        }
    }

    launch_with_explorer(CLAUDE_STORE_APP_ID.as_ref())
        .map_err(|e| format!("启动 Claude Desktop 失败: {e}"))
}

#[cfg(target_os = "windows")]
fn claude_desktop_shortcut() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(user_profile) = env::var_os("USERPROFILE") {
        candidates.push(
            PathBuf::from(user_profile)
                .join("Desktop")
                .join("Claude.lnk"),
        );
    }
    if let Some(public) = env::var_os("PUBLIC") {
        candidates.push(PathBuf::from(public).join("Desktop").join("Claude.lnk"));
    }

    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(target_os = "windows")]
fn launch_with_explorer(target: &std::ffi::OsStr) -> std::io::Result<()> {
    hidden_command("explorer.exe")
        .arg(target)
        .spawn()
        .map(|_| ())
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

#[cfg(target_os = "windows")]
fn hidden_output_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}
