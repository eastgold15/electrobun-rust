use std::env;          // 环境变量、当前可执行文件路径
use std::fs;          // 文件操作
use std::path::PathBuf;// 路径拼接
use std::process;     // 子进程管理

#[cfg(unix)]
extern "C" {
    fn signal(sig: i32, handler: extern "C" fn(i32)) -> *const ();
    fn kill(pid: u32, sig: i32) -> i32;
    fn alarm(seconds: u32) -> u32;
}
#[cfg(unix)]
const SIGINT: i32 = 2;
#[cfg(unix)]
const SIGTERM: i32 = 15;
#[cfg(unix)]
const SIGHUP: i32 = 1;
#[cfg(unix)]
const SIGKILL: i32 = 9;

// Global state for signal handling and child process management
// Rust doesn't allow uninitialized globals, so Mutex wraps each value
static CHILD_PID: std::sync::Mutex<u32> = std::sync::Mutex::new(0);

#[allow(dead_code)]
static SHOULD_EXIT: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

#[allow(dead_code)]
static SIGINT_COUNT: std::sync::Mutex<u32> = std::sync::Mutex::new(0);

#[cfg(target_os = "windows")]
#[allow(dead_code, non_snake_case)]
mod windows_imports {
    use std::os::windows::raw::HANDLE;

    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type LPWSTR = *mut u16;

    #[repr(C)]
    pub struct PROCESS_INFORMATION {
        pub hProcess: HANDLE,
        pub hThread: HANDLE,
        pub dwProcessId: DWORD,
        pub dwThreadId: DWORD,
    }

     // 启动信息结构体，对应 C 的 STARTUPINFOW
    #[repr(C)]
    pub struct STARTUPINFOW {
        pub cb: DWORD,
        pub lpReserved: LPWSTR,
        pub lpDesktop: LPWSTR,
        pub lpTitle: LPWSTR,
        pub dwX: DWORD,
        pub dwY: DWORD,
        pub dwXSize: DWORD,
        pub dwYSize: DWORD,
        pub dwXCountChars: DWORD,
        pub dwYCountChars: DWORD,
        pub dwFillAttribute: DWORD,
        pub dwFlags: DWORD,
        pub wShowWindow: u16,
        pub cbReserved2: u16,
        pub lpReserved2: *mut u8,
        pub hStdInput: HANDLE,
        pub hStdOutput: HANDLE,
        pub hStdError: HANDLE,
    }

    // 声明 Windows API 函数
    #[link(name = "kernel32")]
    extern "system" {
        pub fn CreateProcessW(
            lpApplicationName: LPWSTR,
            lpCommandLine: LPWSTR,
            lpProcessAttributes: *mut std::ffi::c_void,
            lpThreadAttributes: *mut std::ffi::c_void,
            bInheritHandles: BOOL,
            dwCreationFlags: DWORD,
            lpEnvironment: *mut std::ffi::c_void,
            lpCurrentDirectory: LPWSTR,
            lpStartupInfo: *mut STARTUPINFOW,
            lpProcessInformation: *mut PROCESS_INFORMATION,
        ) -> BOOL;

        pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
        pub fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: *mut DWORD) -> BOOL;
        pub fn CloseHandle(hObject: HANDLE) -> BOOL;
        pub fn AttachConsole(dwProcessId: DWORD) -> BOOL;
        pub fn FreeConsole() -> BOOL;
        pub fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
        pub fn SetStdHandle(nStdHandle: DWORD, hHandle: HANDLE) -> BOOL;
    }
    // Windows 常量
    pub const ATTACH_PARENT_PROCESS: DWORD = 0xFFFFFFFF;
    pub const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5;
    pub const STD_ERROR_HANDLE: DWORD = 0xFFFFFFF4;
    pub const CREATE_NO_WINDOW: DWORD = 0x08000000;
    pub const INFINITE: DWORD = 0xFFFFFFFF;
}

use std::collections::HashMap;

/// Check if the build is a development version
fn is_dev_build(exe_dir: &str) -> bool {
    // PathBuf handles memory allocation automatically
    let version_path = PathBuf::from(exe_dir).join("..").join("Resources").join("version.json");
    // fs::read_to_string opens, reads, and closes in one call
    //      如果失败返回 Err，用 .ok()? 转为 None
    let content = match fs::read_to_string(&version_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    // Parsing with serde_json
    let parsed: HashMap<String, serde_json::Value> = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(_) => return false,
    };

    //      if (channel_value == .string) { ... channel_value.string ... }
    // parsed.get("channel") returns Option<&Value>
    if let Some(channel_value) = parsed.get("channel") {
        if let Some(channel_str) = channel_value.as_str() {
            // channel_str == "dev"
            return channel_str == "dev";
        }
    }

    false
}

fn detect_main_process() -> &'static str {
    "bun"
}
// 注意：Rust 的信号处理函数必须是 extern "C" 的，且只能调用 async-signal-safe 函数
#[cfg(unix)]
extern "C" fn alarm_handler(_: i32) {
    unsafe {
        // kill(0, SIGKILL) sends to entire process group
        kill(0, SIGKILL);
    }
}

#[cfg(unix)]
extern "C" fn signal_handler(sig: i32) {
    if sig == SIGINT {
        // Must use Mutex to modify shared state
        let mut count = SIGINT_COUNT.lock().unwrap();
        *count += 1;

        if *count == 1 {
            // 第一次 Ctrl+C
            unsafe { alarm(10); }
            return;
        } else {
            // 第二次 Ctrl+C
            unsafe { alarm(0); }
            unsafe { kill(0, SIGKILL); }
            return;
        }
    }
    // Forward signal to child process
    let pid = *CHILD_PID.lock().unwrap();
    unsafe { kill(pid, sig); }

    if sig == SIGTERM {
        *SHOULD_EXIT.lock().unwrap() = true;
    }
}

// Main entry point
fn main() {
    // ═══════════════════════════════════════════════════════
    // 获取可执行文件所在目录
    // ═══════════════════════════════════════════════════════
    let exe_path = env::current_exe().expect("无法获取可执行文件路径");
    let exe_dir = exe_path.parent().expect("无法获取目录");
    let exe_dir_str = exe_dir.to_str().expect("路径不是有效的 UTF-8");
    println!("Launcher starting on {}...", std::env::consts::OS);
    println!("Current directory: {}", exe_dir_str);

    // ═══════════════════════════════════════════════════════
    // 设置信号处理器（仅非 Windows）
    // ═══════════════════════════════════════════════════════

    #[cfg(unix)]
    unsafe {
        signal(SIGINT, signal_handler);
        signal(SIGTERM, signal_handler);
        signal(SIGHUP, signal_handler);
        signal(SIGKILL, alarm_handler);  // SIGKILL as simplified SIGALRM
    }

    // ═══════════════════════════════════════════════════════
    // 检测主进程类型
    // ═══════════════════════════════════════════════════════

    let main_process = detect_main_process();
    // ═══════════════════════════════════════════════════════
    // 根据平台和主进程类型，确定启动命令
    // ═══════════════════════════════════════════════════════
    let (program, args): (String, Vec<String>) = match main_process {
      "bun" => {
        let bun_name = format!("bun{}", std::env::consts::EXE_SUFFIX);
        let bun_path = PathBuf::from(exe_dir).join(&bun_name);
        let resources_path = PathBuf::from(exe_dir)
            .join("..").join("Resources").join("main.js");
        (bun_path.to_str().unwrap().to_string(), vec![resources_path.to_str().unwrap().to_string()])
    },
    _ => {
        eprintln!("未知的主进程类型: {}", main_process);
        std::process::exit(1);
    }
};

    // ═══════════════════════════════════════════════════════
    // ═══════════════════════════════════════════════════════

    let mut cmd = process::Command::new(&program);
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    cmd.args(&args_refs);
    cmd.current_dir(exe_dir);

    // Linux 特殊处理：CEF 库路径
	#[cfg(target_os = "linux")]
	{
		cmd.env("LD_PRELOAD", preload_libs.join(":"));
		println!("Setting LD_PRELOAD: {}", preload_libs.join(":"));
	}

	cmd.env("ICU_DATA", exe_dir_str);

    // Windows 特殊处理
    #[cfg(target_os = "windows")]
    {
        cmd.env("ICU_DATA", exe_dir_str);
    }

    println!("Spawning: {} {}", program, args.first().map(|s| s.as_str()).unwrap_or(""));

    // ═══════════════════════════════════════════════════════
    // ═══════════════════════════════════════════════════════
    //

    let force_console = env::var("ELECTROBUN_CONSOLE")
        .map(|v| v == "1")
        .unwrap_or(false);

    let is_dev_build = force_console || is_dev_build(exe_dir_str);

    if force_console {
        println!("Console mode forced via ELECTROBUN_CONSOLE=1");
    } else if is_dev_build {
        println!("Dev build detected - console output enabled");
    }

    // ═══════════════════════════════════════════════════════
    // ═══════════════════════════════════════════════════════

    #[cfg(target_os = "windows")]
    if !is_dev_build {
        // ──── Windows 正式版：CreateProcessW，隐藏控制台 ────
        use windows_imports::*;

        let cmd_line = format!("\"{}\" \"{}\"", program, args[0]);
        // UTF-16 conversion
        let mut cmd_line_w: Vec<u16> = cmd_line.encode_utf16().chain(std::iter::once(0)).collect();
        let mut cwd_w: Vec<u16> = exe_dir_str.encode_utf16().chain(std::iter::once(0)).collect();

        // zero-initialized struct
        let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
        si.cb = std::mem::size_of::<STARTUPINFOW>() as DWORD;

        let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

        let success = unsafe {
            CreateProcessW(
                std::ptr::null_mut(),       // lpApplicationName
                cmd_line_w.as_mut_ptr(),     // lpCommandLine（可变！）
                std::ptr::null_mut(),       // lpProcessAttributes
                std::ptr::null_mut(),       // lpThreadAttributes
                0,                          // bInheritHandles
                CREATE_NO_WINDOW,           // dwCreationFlags
                std::ptr::null_mut(),       // lpEnvironment
                cwd_w.as_mut_ptr(),             // lpCurrentDirectory
                &mut si,                    // lpStartupInfo
                &mut pi,                    // lpProcessInformation
            )
        };

        if success == 0 {
            eprintln!("Failed to create process");
            std::process::exit(1);
        }

        println!("Child process spawned with PID {}", pi.dwProcessId);

        unsafe { WaitForSingleObject(pi.hProcess, INFINITE); }

        let mut exit_code: DWORD = 0;
        unsafe { GetExitCodeProcess(pi.hProcess, &mut exit_code); }
        unsafe { CloseHandle(pi.hProcess); }
        unsafe { CloseHandle(pi.hThread); }

        println!("Child process exited with code: {}", exit_code);
        if exit_code != 0 {
            std::process::exit(exit_code as i32);
        }

        return;  // 结束，不执行下面的代码
    }

    // ──── 开发版或非 Windows：标准 spawn ────
    use std::process::Stdio;

    cmd.stdout(Stdio::inherit())
       .stderr(Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to spawn child process: {}", e);
            return;
        }
    };
    // Save PID to global Mutex
    *CHILD_PID.lock().unwrap() = child.id();
    println!("Child process spawned with PID {}", child.id());

    let result = match child.wait() {
        Ok(status) => status,
        Err(e) => {
            eprintln!("Failed to wait for child process: {}", e);
            return;
        }
    };

    match result.code() {
        Some(0) => {},  // 正常退出，什么都不做
        Some(code) => {
            println!("Child process exited with code: {}", code);
            std::process::exit(code);
        }
        None => {
            // 被信号杀死（Unix）
            #[cfg(unix)]
            println!("Child process terminated by signal");
            std::process::exit(1);
        }
    }
}