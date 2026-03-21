use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use tauri::AppHandle;

/// 检查当前进程是否具有管理员权限
#[tauri::command]
pub fn is_elevated() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 在 Windows 上，尝试运行 'net session'。只有管理员权限才能成功运行此命令。
        let mut cmd = Command::new("net");
        cmd.arg("session");
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        
        match cmd.output() {
            Ok(output) => output.status.success(),
            Err(_) => {
                // 如果 net 命令失败，尝试通过 whoami 检查（作为备选方案）
                let mut check_cmd = Command::new("whoami");
                check_cmd.arg("/groups");
                check_cmd.creation_flags(0x08000000);
                if let Ok(output) = check_cmd.output() {
                    let s = String::from_utf8_lossy(&output.stdout);
                    // S-1-16-12288 是高完整性级别 (High Mandatory Level) 的 SID
                    s.contains("S-1-16-12288")
                } else {
                    false
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // 在类 Unix 系统上，检查 UID 是否为 0 (root)
        unsafe { libc::getuid() == 0 }
    }
}

/// 以管理员权限重新启动应用程序
#[tauri::command]
pub fn elevate_process(_app: AppHandle) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| {
        tracing::error!("获取当前执行文件路径失败: {}", e);
        e.to_string()
    })?;

    #[cfg(target_os = "windows")]
    {
        tracing::info!("确认提权请求，准备重新启动应用...");
        
        // 获取当前进程的所有参数
        // 跳过第一个参数（程序路径本身）
        let args: Vec<String> = std::env::args().skip(1).collect();
        let args_str = if args.is_empty() {
            "".to_string()
        } else {
            // 需要正确处理带空格的参数
            args.iter()
                .map(|a| {
                    if a.contains(' ') {
                        format!("'{}'", a)
                    } else {
                        a.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        };

        // 使用 PowerShell 的 Start-Process -Verb RunAs 来触发 UAC
        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        
        let ps_command = if args_str.is_empty() {
            format!("Start-Process '{}' -Verb RunAs", current_exe.to_string_lossy())
        } else {
            format!(
                "Start-Process '{}' -ArgumentList \"{}\" -Verb RunAs",
                current_exe.to_string_lossy(),
                args_str.replace("\"", "\\\"")
            )
        };
        
        cmd.arg(ps_command);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        
        match cmd.spawn() {
            Ok(_) => {
                tracing::info!("已发起提权后的新进程，当前进程即将退出。");
                std::process::exit(0);
            }
            Err(e) => {
                tracing::error!("启动提权进程失败: {}", e);
                return Err(format!("启动提权进程失败: {}", e));
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("目前提权功能仅支持 Windows 平台。".to_string())
    }
}
