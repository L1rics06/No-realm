//! 演示 osu!lazer 进程检测
//!
//! 在操作数据库前检测游戏进程

use no_realm::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== osu!lazer Process Detection ===\n");

    println!("Checking if osu!lazer is running...");

    match process::is_osu_running() {
        Ok(true) => {
            println!("✗ osu!lazer is currently running");
            println!("\nPlease close osu!lazer before modifying the database.");
            println!("This is required to prevent data corruption.");

            // 尝试调用 ensure_osu_not_running 会返回错误
            match process::ensure_osu_not_running() {
                Ok(_) => unreachable!(),
                Err(e) => {
                    println!("\nError when trying to proceed:");
                    println!("  {}", e);
                }
            }
        }
        Ok(false) => {
            println!("✓ osu!lazer is not running");
            println!("\nSafe to modify the database.");

            // 这会成功
            process::ensure_osu_not_running()?;
            println!("✓ Process check passed");
        }
        Err(e) => {
            println!("✗ Failed to check process status: {}", e);
            println!("\nThis might happen if:");
            println!("  - pgrep/tasklist is not available");
            println!("  - Permission denied");
            println!("  - Unsupported platform");
        }
    }

    Ok(())
}
