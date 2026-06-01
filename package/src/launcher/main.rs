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

// ④ Zig: var child_pid = undefined
//    Rust 不允许未初始化，用 Mutex 包裹以便在信号处理函数中修改
//    （Rust 的信号处理函数限制很多，所以用 Mutex + static）
static CHILD_PID: std::sync::Mutex<u32> = std::sync::Mutex::new(0);

// ⑤ Zig: var should_exit = false
//    同理用 Mutex
#[allow(dead_code)]
static SHOULD_EXIT: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

// ⑥ Zig: var sigint_count = 0
//    同理用 Mutex
#[allow(dead_code)]
static SIGINT_COUNT: std::sync::Mutex<u32> = std::sync::Mutex::new(0);


// Zig 用 if (builtin.os.tag == .windows) struct { ... } else struct {}
// Rust 用 #[cfg(target_os = "windows")] 属性

#[cfg(target_os = "windows")]
#[allow(dead_code, non_snake_case)]
mod windows_imports {
    use std::os::windows::raw::HANDLE;

    // Zig: const BOOL = win.BOOL → Rust 直接用 i32
    pub type BOOL = i32;
    // Zig: const DWORD = win.DWORD → Rust 直接用 u32
    pub type DWORD = u32;
    // Zig: const LPWSTR = win.LPWSTR → Rust 用 *mut u16（UTF-16 字符串指针）
    pub type LPWSTR = *mut u16;

    // Zig: extern struct → Rust 用 #[repr(C)] 确保内存布局和 C 一致
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
        pub lpReserved: LPWSTR,       // Zig: ?LPWSTR → Rust: LPWSTR（null 表示无）
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
        pub wShowWindow: u16,         // Zig: win.WORD → Rust: u16
        pub cbReserved2: u16,
        pub lpReserved2: *mut u8,     // Zig: ?*u8 → Rust: *mut u8
        pub hStdInput: HANDLE,
        pub hStdOutput: HANDLE,
        pub hStdError: HANDLE,
    }

    // 声明 Windows API 函数
    // Zig: extern "kernel32" fn ... callconv(win.WINAPI)
    // Rust: extern "system" fn ...（"system" = Windows 上的 WINAPI 调用约定）
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

/// 判断是否是开发版本
/// Zig: fn isDevBuild(allocator, exe_dir) bool
/// Rust: fn is_dev_build(exe_dir: &str) -> bool
///
/// 区别：Zig 需要传入 allocator 来分配内存，Rust 的 String/PathBuf 自动管理
fn is_dev_build(exe_dir: &str) -> bool {
    // Zig: std.fs.path.join(allocator, &.{...}) catch return false
    // Rust: PathBuf::new().push(...)  自动分配，不需要 allocator
    let version_path = PathBuf::from(exe_dir).join("..").join("Resources").join("version.json");

    // Zig: std.fs.openFileAbsolute(path, .{}) catch return false
    // Rust: fs::read_to_string() 一步完成：打开文件 + 读取内容 + 关闭文件
    //      如果失败返回 Err，用 .ok()? 转为 None
    let content = match fs::read_to_string(&version_path) {
        Ok(c) => c,
        Err(_) => return false,  // Zig: catch return false
    };

    // Zig: std.json.parseFromSlice(std.json.Value, ...)
    // Rust: 用 serde_json 解析为 HashMap<String, serde_json::Value>
    let parsed: HashMap<String, serde_json::Value> = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Zig: parsed.value.object.get("channel") |channel_value|
    //      if (channel_value == .string) { ... channel_value.string ... }
    // Rust: parsed.get("channel")  返回 Option<&Value>
    if let Some(channel_value) = parsed.get("channel") {
        if let Some(channel_str) = channel_value.as_str() {
            // Zig: std.mem.eql(u8, channel_value.string, "dev")
            // Rust: channel_str == "dev"
            return channel_str == "dev";
        }
    }

    false
}

/// 主进程类型
// Zig: const MainProcess = enum { bun, zig };
// Rust: enum 几乎一样
#[derive(Debug, Clone, Copy)]
enum MainProcess {
    Bun,
    Zig,
}

/// 检测主进程类型
fn detect_main_process(exe_dir: &str) -> MainProcess {
    let build_path = PathBuf::from(exe_dir).join("..").join("Resources").join("build.json");

    let content = match fs::read_to_string(&build_path) {
        Ok(c) => c,
        Err(_) => return MainProcess::Bun,  // Zig: catch return .bun
    };

    let parsed: HashMap<String, serde_json::Value> = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(_) => return MainProcess::Bun,
    };

    // Zig: if (parsed.value.object.get("mainProcess")) |value| { ... }
    // Rust: if let Some(value) = parsed.get("mainProcess") { ... }
    if let Some(value) = parsed.get("mainProcess") {
        if let Some(s) = value.as_str() {
            if s == "zig" {
                return MainProcess::Zig;
            }
        }
    }

    MainProcess::Bun
}




// Zig: fn alarmHandler(_: c_int) callconv(.C) void
// Rust: extern "C" fn alarm_handler(_: i32)
// 注意：Rust 的信号处理函数必须是 extern "C" 的，且只能调用 async-signal-safe 函数
#[cfg(unix)]
extern "C" fn alarm_handler(_: i32) {
    unsafe {
        // Zig: _ = c.kill(0, c.SIGKILL)
        // Rust: kill(0, SIGKILL)  — 0 表示整个进程组
        kill(0, SIGKILL);
    }
}

#[cfg(unix)]
extern "C" fn signal_handler(sig: i32) {
    if sig == SIGINT {
        // Zig: sigint_count += 1（直接修改全局变量）
        // Rust: 必须通过 Mutex 修改
        let mut count = SIGINT_COUNT.lock().unwrap();
        *count += 1;

        if *count == 1 {
            // 第一次 Ctrl+C
            unsafe { alarm(10); }  // Zig: _ = c.alarm(10)
            return;
        } else {
            // 第二次 Ctrl+C
            unsafe { alarm(0); }   // Zig: _ = c.alarm(0)
            unsafe { kill(0, SIGKILL); }  // Zig: _ = c.kill(0, c.SIGKILL)
            return;
        }
    }

    // Zig: _ = c.kill(@intCast(child_pid), sig)
    // Rust: 转发给子进程
    let pid = *CHILD_PID.lock().unwrap();
    unsafe { kill(pid, sig); }

    if sig == SIGTERM {
        // Zig: should_exit = true
        *SHOULD_EXIT.lock().unwrap() = true;
    }
}



// 第 5 部分：主函数（核心逻辑）
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
        signal(SIGKILL, alarm_handler);  // Zig 用 SIGALRM，这里简化
    }

    // ═══════════════════════════════════════════════════════
    // 检测主进程类型
    // ═══════════════════════════════════════════════════════

    let main_process = detect_main_process(exe_dir_str);
    // ═══════════════════════════════════════════════════════
    // 根据平台和主进程类型，确定启动命令
    // ═══════════════════════════════════════════════════════
    let (program, args): (String, Vec<String>) = match main_process {
      MainProcess::Bun => {
        let bun_name = format!("bun{}", std::env::consts::EXE_SUFFIX);
        let bun_path = PathBuf::from(exe_dir).join(&bun_name);
        let resources_path = PathBuf::from(exe_dir)
            .join("..").join("Resources").join("main.js");
        (bun_path.to_str().unwrap().to_string(), vec![resources_path.to_str().unwrap().to_string()])
    }
      MainProcess::Zig => {
        let main_name = format!("main{}", std::env::consts::EXE_SUFFIX);
        let main_path = PathBuf::from(exe_dir).join(main_name);
        (main_path.to_str().unwrap().to_string(), vec![])
    }
};

    // ═══════════════════════════════════════════════════════
    // Zig 原文第 212-269 行：创建子进程 + 平台环境设置
    // ═══════════════════════════════════════════════════════

    let mut cmd = process::Command::new(&program);
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    cmd.args(&args_refs);
    cmd.current_dir(exe_dir);

    // Linux 特殊处理：CEF 库路径
    #[cfg(target_os = "linux")]
    {
        use std::path::Path;

        // Zig: 检查 libcef.so 和 libvk_swiftshader.so 是否存在
        let cef_exists = Path::new(exe_dir).join("libcef.so").exists();
        let swiftshader_exists = Path::new(exe_dir).join("libvk_swiftshader.so").exists();

        // Zig: 设置 LD_LIBRARY_PATH
        if let Ok(old_val) = env::var("LD_LIBRARY_PATH") {
            cmd.env("LD_LIBRARY_PATH", format!("{}:{}", exe_dir_str, old_val));
        } else {
            cmd.env("LD_LIBRARY_PATH", exe_dir_str);
        }

        // Zig: 如果 CEF 库存在，设置 LD_PRELOAD
        if cef_exists || swiftshader_exists {
            let mut preload_libs = Vec::new();
            if cef_exists { preload_libs.push("./libcef.so"); }
            if swiftshader_exists { preload_libs.push("./libvk_swiftshader.so"); }
            cmd.env("LD_PRELOAD", preload_libs.join(":"));
            println!("Setting LD_PRELOAD: {}", preload_libs.join(":"));
        }

        cmd.env("ICU_DATA", exe_dir_str);
    }

    // Windows 特殊处理
    #[cfg(target_os = "windows")]
    {
        cmd.env("ICU_DATA", exe_dir_str);
    }

    println!("Spawning: {} {}", program, args.first().map(|s| s.as_str()).unwrap_or(""));

    // ═══════════════════════════════════════════════════════
    // Zig 原文第 274-289 行：判断是否为开发版本
    // ═══════════════════════════════════════════════════════
    //
    // Zig:
    //   const force_console = if (std.process.getEnvVarOwned(...)) |val| blk: {
    //       defer arena_alloc.free(val);
    //       break :blk std.mem.eql(u8, val, "1");
    //   } else |_| false;
    //   const is_dev_build = force_console or isDevBuild(arena_alloc, exe_dir);

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
    // Zig 原文第 289-384 行：启动子进程，分两条路径
    // ═══════════════════════════════════════════════════════


    #[cfg(target_os = "windows")]
    if !is_dev_build {
        // ──── Windows 正式版：CreateProcessW，隐藏控制台 ────
        use windows_imports::*;

        // Zig: 把命令行拼成 "program" "arg" 格式（CreateProcessW 要求）
        let cmd_line = format!("\"{}\" \"{}\"", program, args[0]);

        // Zig: std.unicode.utf8ToUtf16LeWithNull → 转 UTF-16
        // Rust: 用 widestring crate 或手动转换
        let mut cmd_line_w: Vec<u16> = cmd_line.encode_utf16().chain(std::iter::once(0)).collect();
        let mut cwd_w: Vec<u16> = exe_dir_str.encode_utf16().chain(std::iter::once(0)).collect();

        // Zig: var si: win.STARTUPINFOW = std.mem.zeroes(win.STARTUPINFOW);
        //       si.cb = @sizeOf(win.STARTUPINFOW);
        // Rust: 零初始化结构体
        let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
        si.cb = std::mem::size_of::<STARTUPINFOW>() as DWORD;

        let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

        // Zig: win.CreateProcessW(null, @constCast(cmd_line_w.ptr), ...)
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

        // Zig: _ = win.WaitForSingleObject(pi.hProcess, win.INFINITE);
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

    // Rust: 保存 PID 到全局 Mutex
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