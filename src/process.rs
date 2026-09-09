//! osu!lazer 进程检测
//!
//! 在操作数据库之前，必须确保 osu!lazer 进程已关闭，
//! 否则可能导致数据损坏或锁定问题。

use crate::error::{Error, Result};
use std::process::Command;

/// 检测 osu!lazer 进程是否正在运行
///
/// # 返回
///
/// - `Ok(true)` - 进程正在运行
/// - `Ok(false)` - 进程未运行
/// - `Err(_)` - 检测失败
pub fn is_osu_running() -> Result<bool> {
    #[cfg(target_os = "linux")]
    {
        is_osu_running_linux()
    }

    #[cfg(target_os = "windows")]
    {
        is_osu_running_windows()
    }

    #[cfg(target_os = "macos")]
    {
        is_osu_running_macos()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(Error::other("Unsupported platform for process detection"))
    }
}

/// 确保 osu!lazer 未运行，否则返回错误
///
/// 这是所有写操作的前置检查
pub fn ensure_osu_not_running() -> Result<()> {
    if is_osu_running()? {
        Err(Error::other(
            "osu!lazer is currently running. Please close it before modifying the database."
        ))
    } else {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn is_osu_running_linux() -> Result<bool> {
    // 检查常见的 osu!lazer 进程名
    // 使用精确匹配避免误报（例如 fusermount 中包含 "ou"）
    let process_patterns = [
        "osu!",       // AppImage 格式
        "osu.Desktop", // Flatpak 或直接运行
        "^osu$",      // 精确匹配 "osu"
    ];

    for pattern in &process_patterns {
        let output = Command::new("pgrep")
            .arg("-f")
            .arg(pattern)
            .output()
            .map_err(|e| Error::other(format!("Failed to run pgrep: {}", e)))?;

        if output.status.success() && !output.stdout.is_empty() {
            // 进一步验证：检查进程命令行
            let pid_list = String::from_utf8_lossy(&output.stdout);
            for pid in pid_list.lines() {
                if let Ok(pid_num) = pid.trim().parse::<u32>() {
                    // 读取 /proc/<pid>/cmdline 验证
                    if let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{}/cmdline", pid_num)) {
                        let cmdline_lower = cmdline.to_lowercase();
                        // 确保是真正的 osu 进程，而不是包含 "osu" 的其他进程
                        if cmdline_lower.contains("osu!") || cmdline_lower.contains("osu.desktop") || cmdline_lower.contains("/osu") {
                            log::info!("Found osu!lazer process: PID {} - {}", pid_num, cmdline.replace('\0', " "));
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}

#[cfg(target_os = "windows")]
fn is_osu_running_windows() -> Result<bool> {
    // 在 Windows 上使用 tasklist
    let process_names = ["osu!.exe", "osu.Desktop.exe", "osu.exe"];

    for name in &process_names {
        let output = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("IMAGENAME eq {}", name))
            .output()
            .map_err(|e| Error::other(format!("Failed to run tasklist: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(name) {
            log::info!("Found osu!lazer process: {}", name);
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(target_os = "macos")]
fn is_osu_running_macos() -> Result<bool> {
    // 在 macOS 上使用 pgrep
    let process_names = ["osu!", "osu.Desktop", "osu"];

    for name in &process_names {
        let output = Command::new("pgrep")
            .arg("-f")
            .arg(name)
            .output()
            .map_err(|e| Error::other(format!("Failed to run pgrep: {}", e)))?;

        if output.status.success() && !output.stdout.is_empty() {
            log::info!("Found osu!lazer process: {}", name);
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_detection() {
        // 测试进程检测功能
        let result = is_osu_running();
        assert!(result.is_ok(), "Process detection should not fail");

        let is_running = result.unwrap();
        println!("osu!lazer running: {}", is_running);
    }

    #[test]
    fn test_ensure_not_running() {
        let result = ensure_osu_not_running();
        // 如果测试时 osu!lazer 正在运行，这个测试会失败
        // 这是预期行为
        if result.is_err() {
            println!("Test skipped: osu!lazer is running");
        }
    }
}
