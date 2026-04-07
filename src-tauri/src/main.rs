// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    {
        // 1. 检查操作系统版本（建议 Windows 10 及以上）
        if !check_os_version() {
            let text = "您的操作系统版本过低（低于 Windows 10）。\n\nTauri 框架和 WebView2 在旧版 Windows（如 Win7/Win8）上可能无法正常工作，导致界面白屏或程序无法启动。\n\n是否仍要尝试继续运行？（建议升级系统）";
            let caption = "系统版本过低提示";
            // MB_YESNO | MB_ICONWARNING
            let res = show_message_box(text, caption, 0x00000004 | 0x00000030);
            // IDYES = 6, IDNO = 7
            if res != 6 {
                std::process::exit(1);
            }
        }

        // 2. 检查 WebView2 运行时
        if !check_webview2() {
            let text = "您的系统缺少必要的组件：Microsoft Edge WebView2 运行时。\n这会导致软件窗口无法打开。\n\n点击【确定】将自动前往微软官网下载安装，安装后请重新启动本软件。";
            let caption = "缺少必要的运行环境";
            // MB_OKCANCEL | MB_ICONERROR
            let res = show_message_box(text, caption, 0x00000001 | 0x00000010);
            // IDOK = 1
            if res == 1 {
                let _ = std::process::Command::new("cmd")
                    .args(&["/C", "start", "https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/?form=MA13LH#download"])
                    .spawn();
            }
            std::process::exit(1);
        }
        // 3. 检查 Visual C++ Redistributable (WebView2 等组件可能需要)
        if !check_vc_redist() {
            let text = "您的系统缺少必要的组件：Microsoft Visual C++ Redistributable。\n这可能导致软件及其底层组件无法正常运行。\n\n点击【确定】将自动前往微软官网下载安装最新版，安装后请重新启动本软件。";
            let caption = "缺少必要的运行环境";
            // MB_OKCANCEL | MB_ICONERROR
            let res = show_message_box(text, caption, 0x00000001 | 0x00000010);
            // IDOK = 1
            if res == 1 {
                let download_url = if cfg!(target_arch = "x86_64") {
                    "https://aka.ms/vc14/vc_redist.x64.exe"
                } else if cfg!(target_arch = "aarch64") {
                    "https://aka.ms/vc14/vc_redist.arm64.exe"
                } else {
                    "https://aka.ms/vc14/vc_redist.x86.exe"
                };
                
                let _ = std::process::Command::new("cmd")
                    .args(&["/C", "start", download_url])
                    .spawn();
            }
            std::process::exit(1);
        }
    }

    sillytavern_launcher_lib::run()
}

#[cfg(target_os = "windows")]
fn check_os_version() -> bool {
    use std::mem;
    #[repr(C)]
    struct OSVERSIONINFOW {
        dw_os_version_info_size: u32,
        dw_major_version: u32,
        dw_minor_version: u32,
        dw_build_number: u32,
        dw_platform_id: u32,
        sz_csd_version: [u16; 128],
    }

    #[link(name = "ntdll")]
    extern "system" {
        fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> i32;
    }

    let mut info: OSVERSIONINFOW = unsafe { mem::zeroed() };
    info.dw_os_version_info_size = mem::size_of::<OSVERSIONINFOW>() as u32;

    unsafe {
        if RtlGetVersion(&mut info) == 0 {
            // Windows 10 的 Major Version 是 10
            if info.dw_major_version < 10 {
                return false;
            }
        }
    }
    true
}

#[cfg(target_os = "windows")]
fn check_webview2() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let key_path = r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    let key_path_64 = r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    let check_key = |root: &RegKey, path: &str| -> bool {
        if let Ok(key) = root.open_subkey(path) {
            if let Ok(pv) = key.get_value::<String, _>("pv") {
                return !pv.is_empty() && pv != "0.0.0.0";
            }
        }
        false
    };

    check_key(&hklm, key_path) || check_key(&hkcu, key_path) || check_key(&hklm, key_path_64) || check_key(&hkcu, key_path_64)
}

#[cfg(target_os = "windows")]
fn check_vc_redist() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    
    // VC++ Redistributable 通常注册在这个路径
    let key_path = r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes";
    let key_path_64 = r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes";

    let check_arch = |root: &RegKey, path: &str, arch: &str| -> bool {
        let arch_path = format!("{}\\{}", path, arch);
        if let Ok(key) = root.open_subkey(&arch_path) {
            if let Ok(installed) = key.get_value::<u32, _>("Installed") {
                return installed == 1;
            }
        }
        false
    };

    let target_arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86"
    };

    check_arch(&hklm, key_path, target_arch) || check_arch(&hklm, key_path_64, target_arch)
}

#[cfg(target_os = "windows")]
fn show_message_box(text: &str, caption: &str, utype: u32) -> i32 {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: *mut std::ffi::c_void, lptext: *const u16, lpcaption: *const u16, utype: u32) -> i32;
    }

    let mut text_w: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().collect();
    text_w.push(0);

    let mut caption_w: Vec<u16> = std::ffi::OsStr::new(caption).encode_wide().collect();
    caption_w.push(0);

    unsafe {
        MessageBoxW(null_mut(), text_w.as_ptr(), caption_w.as_ptr(), utype)
    }
}
