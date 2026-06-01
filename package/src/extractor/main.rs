//! Electrobun Self-Extractor - Rust Port
//! Converts from Zig to Rust following the cross-reference table

use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// Zig: const ARCHIVE_MARKER = "ELECTROBUN_ARCHIVE_V1"
const ARCHIVE_MARKER: &[u8] = b"ELECTROBUN_ARCHIVE_V1";
const METADATA_MARKER: &[u8] = b"ELECTROBUN_METADATA_V1";

// Base64 constant
const BASE64_STANDARD: base64::engine::general_purpose::GeneralPurpose = 
    base64::engine::general_purpose::GeneralPurpose::new(&base64::engine::general_purpose::STANDARD);

/// App metadata structure
#[derive(Debug, Clone)]
struct AppMetadata {
    identifier: String,
    name: String,
    channel: String,
    hash: Option<String>,
}

/// Progress indicator for extraction
struct ProgressIndicator {
    child_process: Option<std::process::Child>,
    app_name: String,
}

impl ProgressIndicator {
    fn new(metadata: &AppMetadata) -> Self {
        let mut indicator = ProgressIndicator {
            child_process: None,
            app_name: metadata.name.clone(),
        };

        if let Err(_) = indicator.start_progress_dialog(metadata) {
            println!("\nInstalling {}...", metadata.name);
        }

        indicator
    }

    fn start_progress_dialog(&mut self, metadata: &AppMetadata) -> Result<(), Box<dyn std::error::Error>> {
        // Windows doesn't support progress dialogs
        if cfg!(target_os = "windows") {
            return Err("No progress dialog on Windows".into());
        }

        if !cfg!(target_os = "linux") {
            return Err("Only Linux supported".into());
        }

        // Try zenity first
        let extract_text = format!("--text=Extracting {}...", metadata.name);

        let child = Command::new("zenity")
            .args(&[
                "--progress", "--pulsate", "--no-cancel",
                "--title=Electrobun Installer",
                &extract_text,
                "--auto-close",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match child {
            Ok(c) => {
                self.child_process = Some(c);
                Ok(())
            }
            Err(_) => {
                // Try kdialog for KDE
                let kdialog_text = format!("Extracting {}...", metadata.name);
                let kde_child = Command::new("kdialog")
                    .args(&["--progressbar", &kdialog_text, "0", "--title", "Electrobun Installer"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();

                match kde_child {
                    Ok(c) => {
                        self.child_process = Some(c);
                        Ok(())
                    }
                    Err(_) => Err("No progress dialog available".into()),
                }
            }
        }
    }

    fn close(&mut self) {
        if let Some(mut child) = self.child_process.take() {
            drop(child.stdin.take());
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for ProgressIndicator {
    fn drop(&mut self) {
        self.close();
    }
}

/// Get application data directory based on platform
fn get_app_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        // Use %LOCALAPPDATA% on Windows
        if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_appdata));
        }
        if let Ok(appdata) = env::var("APPDATA") {
            return Ok(PathBuf::from(appdata));
        }
        if let Ok(userprofile) = env::var("USERPROFILE") {
            return Ok(PathBuf::from(userprofile).join("AppData").join("Local"));
        }
        Err("Could not determine app data directory".into())
    } else if cfg!(target_os = "linux") {
        // Use XDG_DATA_HOME or ~/.local/share on Linux
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data_home));
        }
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local").join("share"));
        }
        Err("Could not determine app data directory".into())
    } else {
        Err("Unsupported platform".into())
    }
}

/// Read embedded metadata from binary
fn read_embedded_metadata(
    file: &mut File,
    metadata_start: u64,
    archive_start: u64,
) -> Result<AppMetadata, Box<dyn std::error::Error>> {
    let metadata_size = archive_start - metadata_start;
    if metadata_size > 4096 {
        return Err("Metadata too large".into());
    }

    file.seek(std::io::SeekFrom::Start(metadata_start as u64))?;

    let mut metadata_bytes = vec![0u8; metadata_size as usize];
    file.read_exact(&mut metadata_bytes)?;

    // Parse JSON metadata
    let parsed: serde_json::Value = serde_json::from_slice(&metadata_bytes)?;

    Ok(AppMetadata {
        identifier: parsed["identifier"]
            .as_str()
            .map(String::from)
            .unwrap_or_default(),
        name: parsed["name"].as_str().map(String::from).unwrap_or_default(),
        channel: parsed["channel"]
            .as_str()
            .map(String::from)
            .unwrap_or_default(),
        hash: parsed["hash"].as_str().map(String::from),
    })
}

/// Find second occurrence of marker in buffer
fn find_second_occurrence(buffer: &[u8], marker: &[u8]) -> Option<usize> {
    let first_pos = memchr::memmem::find(buffer, marker)?;
    let search_start = first_pos + marker.len();
    let second_offset = memchr::memmem::find(&buffer[search_start..], marker)?;
    Some(search_start + second_offset)
}

/// Escape special characters for desktop file
fn escape_desktop_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// Fix executable permissions on extracted binaries
fn fix_executable_permissions(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let executables = [
        "bin/launcher",
        "bin/bun",
        "bin/bspatch",
        "bin/bsdiff",
        "bin/zig-zstd",
    ];

    for exe in executables {
        let exe_path = app_dir.join(exe);
        if exe_path.exists() {
            #[cfg(unix)]
            {
                // Skip chmod for macOS app bundles to preserve code signatures
                if cfg!(target_os = "macos") && app_dir.to_string_lossy().contains(".app") {
                    continue;
                }
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mut perms) = fs::metadata(&exe_path)?.permissions().mode() {
                    perms |= 0o755;
                    fs::set_permissions(&exe_path, fs::Permissions::from_mode(perms))?;
                }
            }
        }
    }

    // Find and fix .sh scripts
    if !cfg!(target_os = "windows") {
        fix_shell_scripts(app_dir)?;
    }

    Ok(())
}

#[cfg(unix)]
fn fix_shell_scripts(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fix_shell_scripts(&path)?;
        } else if let Some(ext) = path.extension() {
            if ext == "sh" {
                let mut perms = fs::metadata(&path)?.permissions();
                let mode = perms.mode() | 0o755;
                perms.set_mode(mode);
                fs::set_permissions(&path, perms)?;
            }
        }
    }
    Ok(())
}

/// Fix CEF symlinks
fn fix_cef_symlinks(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let bin_dir = app_dir.join("bin");
    let cef_dir = bin_dir.join("cef");

    if !cef_dir.exists() {
        println!("CEF directory not found, skipping symlink creation");
        return Ok(());
    }

    let cef_libs = [
        "libcef.so",
        "libEGL.so",
        "libGLESv2.so",
        "libvk_swiftshader.so",
        "libvulkan.so.1",
    ];

    println!("Creating CEF symlinks...");

    for lib in cef_libs {
        let symlink_path = bin_dir.join(lib);
        let target_path = format!("cef/{}", lib);

        // Remove existing symlink/file if it exists
        let _ = fs::remove_file(&symlink_path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if let Err(e) = symlink(&target_path, &symlink_path) {
                println!("Warning: Could not create symlink for {}: {}", lib, e);
            } else {
                println!("Created symlink: {} -> {}", lib, target_path);
            }
        }
    }

    Ok(())
}

/// Remove quarantine attributes on macOS
#[cfg(target_os = "macos")]
fn remove_quarantine(app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Removing quarantine attributes from: {:?}", app_dir);

    let output = Command::new("xattr")
        .args(&["-r", "-d", "com.apple.quarantine"])
        .arg(app_dir)
        .output()?;

    if output.status.success() {
        println!("Successfully removed quarantine attributes");
    } else {
        println!("Warning: xattr returned non-zero exit code");
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn remove_quarantine(_app_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// Create Linux desktop shortcut
fn create_desktop_shortcut(
    app_dir: &Path,
    metadata: &AppMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let desktop_dir = PathBuf::from(&home).join("Desktop");

    if !desktop_dir.exists() {
        println!("Note: Desktop directory not found; skipping Desktop shortcut creation");
        return Ok(());
    }

    let launcher_path = app_dir.join("bin").join("launcher");
    if !launcher_path.exists() {
        println!("Warning: launcher binary not found at {:?}", launcher_path);
        return Ok(());
    }

    // Look for .desktop file in extracted app directory
    let mut found_desktop_file = false;
    for entry in fs::read_dir(app_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "desktop" {
                    found_desktop_file = true;
                    // Copy and modify desktop file
                    let desktop_content = fs::read_to_string(&path)?;

                    // Find icon in app directory
                    let mut icon_path: Option<PathBuf> = None;

                    // Try root directory first
                    for icon_entry in fs::read_dir(app_dir)? {
                        if let Ok(icon_entry) = icon_entry {
                            if let Some(ext) = icon_entry.path().extension() {
                                if ext == "png" {
                                    icon_path = Some(icon_entry.path());
                                    break;
                                }
                            }
                        }
                    }

                    // Try Resources subdirectory
                    if icon_path.is_none() {
                        let resources_path = app_dir.join("Resources");
                        if resources_path.exists() {
                            for icon_entry in fs::read_dir(&resources_path)? {
                                if let Ok(icon_entry) = icon_entry {
                                    if let Some(ext) = icon_entry.path().extension() {
                                        if ext == "png" {
                                            icon_path = Some(icon_entry.path());
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Create modified desktop file
                    let mut new_content = String::new();
                    for line in desktop_content.lines() {
                        if line.starts_with("Exec=") {
                            new_content.push_str(&format!("Exec=\"{}\"\n", launcher_path.display()));
                        } else if line.starts_with("Icon=") {
                            if let Some(ref icon) = icon_path {
                                new_content.push_str(&format!("Icon={}\n", icon.display()));
                            } else {
                                new_content.push_str(line);
                                new_content.push('\n');
                            }
                        } else {
                            new_content.push_str(line);
                            new_content.push('\n');
                        }
                    }

                    // Write to desktop
                    let desktop_filename = format!("{}.desktop", metadata.name);
                    let desktop_path = desktop_dir.join(&desktop_filename);

                    if let Ok(mut file) = File::create(&desktop_path) {
                        let _ = file.write_all(new_content.as_bytes());

                        // Make executable
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = fs::metadata(&desktop_path)?.permissions();
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(&desktop_path, perms);

                        println!("Created desktop shortcut: {:?}", desktop_path);
                    }

                    // Write to XDG applications directory
                    if let Ok(xdg_data_home) = get_app_data_dir() {
                        let applications_dir = xdg_data_home.join("applications");
                        let _ = fs::create_dir_all(&applications_dir);

                        let applications_path = applications_dir.join(&desktop_filename);
                        if let Ok(mut file) = File::create(&applications_path) {
                            let _ = file.write_all(new_content.as_bytes());

                            use std::os::unix::fs::PermissionsExt;
                            let mut perms = fs::metadata(&applications_path)?.permissions();
                            perms.set_mode(0o644);
                            let _ = fs::set_permissions(&applications_path, perms);
                        }
                    }
                    break;
                }
            }
        }
    }

    if !found_desktop_file {
        println!("Warning: No desktop file found in extracted app directory");
    }

    Ok(())
}

/// Create Windows shortcut using PowerShell
#[cfg(target_os = "windows")]
fn create_windows_shortcut(
    app_dir: &Path,
    metadata: &AppMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let userprofile = env::var("USERPROFILE")?;
    let desktop_dir = PathBuf::from(&userprofile).join("Desktop");
    let start_menu_dir = PathBuf::from(&userprofile)
        .join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs");

    let target_path = app_dir.join("bin").join("launcher.exe");
    if !target_path.exists() {
        println!("Warning: Could not find launcher.exe at {:?}", target_path);
        return Ok(());
    }

    let working_dir = app_dir.join("bin");
    let icon_to_use = &target_path;

    // Create shortcuts on Desktop
    create_windows_shortcut_file(
        &desktop_dir,
        &metadata.name,
        &target_path,
        &working_dir,
        icon_to_use,
    )?;

    // Create shortcuts in Start Menu
    let _ = fs::create_dir_all(&start_menu_dir);
    create_windows_shortcut_file(
        &start_menu_dir,
        &metadata.name,
        &target_path,
        &working_dir,
        icon_to_use,
    )?;

    println!("Created Windows shortcuts for: {}", metadata.name);

    // Create uninstall registry file
    create_windows_uninstall_reg(app_dir, metadata)?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn create_windows_shortcut_file(
    shortcut_dir: &Path,
    app_name: &str,
    target_path: &Path,
    working_dir: &Path,
    icon_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let lnk_name = format!("{}.lnk", app_name);
    let lnk_path = shortcut_dir.join(&lnk_name);

    let ps_content = format!(
        r#"$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("{}")
$Shortcut.TargetPath = "{}"
$Shortcut.WorkingDirectory = "{}"
$Shortcut.IconLocation = "{}"
$Shortcut.WindowStyle = 1
$Shortcut.Save()
"#,
        lnk_path.display(),
        target_path.display(),
        working_dir.display(),
        icon_path.display()
    );

    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &ps_content,
        ])
        .output()?;

    if output.status.success() {
        println!("Created Windows shortcut: {:?}", lnk_path);
    } else {
        println!("Warning: Could not create shortcut");
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn create_windows_uninstall_reg(
    app_dir: &Path,
    metadata: &AppMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let reg_name = format!("{}_uninstall.reg", metadata.name);
    let reg_path = app_dir.join(&reg_name);

    let app_display_name = format!("{} ({})", metadata.name, metadata.channel);

    let reg_content = format!(
        r#"Windows Registry Editor Version 5.00

[HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\{}]
@="{}"
"DisplayName"="{}"
"DisplayVersion"="1.0"
"Publisher"="Electrobun"
"InstallLocation"="{}"
"UninstallString"="cmd.exe /c rmdir /s /q \ "{} ""
"NoModify"=dword:00000001
"NoRepair"=dword:00000001
"#,
        metadata.identifier,
        app_display_name,
        app_display_name,
        app_dir.display(),
        app_dir.display()
    );

    if let Ok(mut file) = File::create(&reg_path) {
        let _ = file.write_all(reg_content.as_bytes());
        println!("Created uninstall registry file: {:?}", reg_path);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn create_windows_shortcut(
    _app_dir: &Path,
    _metadata: &AppMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// Replace self with launcher on macOS
#[cfg(target_os = "macos")]
fn replace_self_with_launcher(
    exe_path: &Path,
    app_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let launcher_name = "launcher";
    let launcher_path = app_dir.join("bin").join(launcher_name);

    if !launcher_path.exists() {
        println!("Warning: Could not find launcher at {:?}", launcher_path);
        return Ok(());
    }

    fs::copy(&launcher_path, exe_path)?;
    println!("Replaced self with launcher shortcut from: {:?}", launcher_path);

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn replace_self_with_launcher(
    _exe_path: &Path,
    _app_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// Extract tar archive
fn extract_tar(tar_data: &[u8], extract_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("DEBUG: Starting tar extraction to: {:?}", extract_dir);
    println!("DEBUG: Tar data size: {} bytes", tar_data.len());

    // Clean up existing directory
    let _ = fs::remove_dir_all(extract_dir);

    // Create extraction directory
    fs::create_dir_all(extract_dir)?;

    // Use our tar extraction
    tar_pipe_to_filesystem(extract_dir, tar_data)?;

    Ok(())
}

/// Tar pipe to filesystem
fn tar_pipe_to_filesystem(dir: &Path, tar_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    const BLOCK_SIZE: usize = 512;
    let mut pos: usize = 0;

    loop {
        if pos + BLOCK_SIZE > tar_data.len() {
            break;
        }

        let header = &tar_data[pos..pos + BLOCK_SIZE];

        // Check for zero block (end of archive)
        if header.iter().all(|&b| b == 0) {
            break;
        }

        // Parse tar header
        let file_name = parse_tar_name(header)?;
        let file_size = parse_tar_size(header)?;
        let file_type = parse_tar_type(header)?;

        pos += BLOCK_SIZE;

        match file_type {
            '5' => {
                // Directory
                if !file_name.is_empty() {
                    let path = dir.join(&file_name);
                    let _ = fs::create_dir_all(&path);
                }
            }
            '0' => {
                // Regular file
                if file_size == 0 && file_name.is_empty() {
                    break;
                }

                // Create parent directories
                if let Some(parent) = Path::new(&file_name).parent() {
                    if !parent.as_os_str().is_empty() {
                        let path = dir.join(parent);
                        let _ = fs::create_dir_all(&path);
                    }
                }

                let file_path = dir.join(&file_name);
                let block_size_u64 = BLOCK_SIZE as u64;
                let rounded_size = ((file_size + block_size_u64 - 1) / block_size_u64) * block_size_u64;

                if pos + file_size as usize <= tar_data.len() {
                    let mut file = File::create(&file_path)?;
                    file.write_all(&tar_data[pos..pos + file_size as usize])?;
                }

                pos += rounded_size as usize;
            }
            '2' => {
                // Symbolic link
                let link_target_size = file_size.min(1024) as usize;
                if pos + link_target_size <= tar_data.len() {
                    let link_target = String::from_utf8_lossy(&tar_data[pos..pos + link_target_size]);
                    let link_path = dir.join(&file_name);

                    // Create parent directory
                    if let Some(parent) = Path::new(&file_name).parent() {
                        if !parent.as_os_str().is_empty() {
                            let _ = fs::create_dir_all(&dir.join(parent));
                        }
                    }

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::symlink;
                        let _ = symlink(link_target.trim(), &link_path);
                    }
                }

                let rounded_size = ((file_size + block_size_u64 - 1) / block_size_u64) * block_size_u64;
                pos += rounded_size as usize;
            }
            _ => {
                // Skip other types
                let rounded_size = ((file_size + block_size_u64 - 1) / block_size_u64) * block_size_u64;
                pos += rounded_size as usize;
            }
        }
    }

    Ok(())
}

fn parse_tar_name(header: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    // Simple tar name parsing (first 100 bytes)
    let name_end = header[..100].iter().position(|&b| b == 0).unwrap_or(100);
    let name = String::from_utf8_lossy(&header[..name_end]).to_string();

    // Check for ustar prefix (bytes 345-500)
    if header[257..257 + 6].iter().eq(b"ustar\x00") {
        let prefix_end = header[345..500].iter().position(|&b| b == 0).unwrap_or(155);
        let prefix = String::from_utf8_lossy(&header[345..345 + prefix_end]).to_string();
        if !prefix.is_empty() {
            return Ok(format!("{}/{}", prefix, name));
        }
    }

    Ok(name)
}

fn parse_tar_size(header: &[u8]) -> Result<u64, Box<dyn std::error::Error>> {
    // Bytes 124-136 contain the size in octal
    let size_str = String::from_utf8_lossy(&header[124..136]);
    let trimmed = size_str.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    Ok(u64::from_str_radix(trimmed, 8)?)
}

fn parse_tar_type(header: &[u8]) -> Result<char, Box<dyn std::error::Error>> {
    // Byte 156 contains the type flag
    let type_byte = header[156];
    if type_byte == 0 {
        Ok('0') // Regular file
    } else {
        Ok(type_byte as char)
    }
}

/// Copy directory recursively
fn copy_directory(src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nDEBUG copyDirectory: src='{:?}' dest='{:?}'", src, dest);

    if !src.is_dir() {
        return Err(format!("Source is not a directory: {:?}", src).into());
    }

    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Decompress and extract
fn extract_and_install(
    compressed_data: &[u8],
    metadata: &AppMetadata,
    self_extraction_dir: &Path,
    app_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize progress indicator
    let _progress = ProgressIndicator::new(metadata);

    println!("Decompressing",);

    // Decompress using zstd
    let window_buffer = vec![0u8; 128 * 1024 * 1024]; // 128MB Buffer

    let mut decompressor = zstd::Decoder::new(compressed_data)?;

    let mut decompressed_data = Vec::new();
    let mut buffer = [0u8; 4096];
    let mut bytes_processed: usize = 0;
    let dot_interval = 10 * 1024 * 1024; // Print dot every 10MB

    loop {
        match decompressor.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                decompressed_data.extend_from_slice(&buffer[..n]);
                bytes_processed += n;
                if bytes_processed >= dot_interval {
                    print!(".");
                    bytes_processed = 0;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    println!(" Done!\n");

    // Extract tar archive
    println!("Extracting files",);
    extract_tar(&decompressed_data, self_extraction_dir)?;
    println!(" Done!\n");

    // Build extracted app path
    let sanitized_name = metadata.name.replace(" ", "");
    let dots_removed = sanitized_name.replace(".", "-");

    let app_bundle_name = if metadata.channel == "stable" {
        dots_removed.clone()
    } else {
        format!("{}-{}", dots_removed, metadata.channel)
    };

    let extracted_app_path = self_extraction_dir.join(&app_bundle_name);

    // Remove existing app directory
    let _ = fs::remove_dir_all(app_dir);

    // Move or copy extracted app to app directory
    #[cfg(target_os = "windows")]
    {
        copy_directory(&extracted_app_path, app_dir)?;
        let _ = fs::remove_dir_all(&extracted_app_path);
    }

    #[cfg(not(target_os = "windows"))]
    {
        fs::rename(&extracted_app_path, app_dir)?;
    }

    // Fix executable permissions
    fix_executable_permissions(app_dir)?;

    // Remove quarantine on macOS
    remove_quarantine(app_dir)?;

    // Fix CEF symlinks
    fix_cef_symlinks(app_dir)?;

    // Replace self with launcher on macOS
    let exe_path = env::current_exe()?;
    replace_self_with_launcher(&exe_path, app_dir)?;

    // Create desktop shortcuts
    #[cfg(target_os = "linux")]
    create_desktop_shortcut(app_dir, metadata)?;

    #[cfg(target_os = "windows")]
    create_windows_shortcut(app_dir, metadata)?;

    // Save tar file for Updater API
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        println!("\n✓ Saving tar file for Updater API...");
        let tar_filename = metadata
            .hash
            .as_ref()
            .map(|h| format!("{}.tar", h))
            .unwrap_or_else(|| "current.tar".to_string());

        let tar_path = self_extraction_dir.join(&tar_filename);

        let _ = fs::create_dir_all(self_extraction_dir);
        let mut tar_file = File::create(&tar_path)?;
        tar_file.write_all(&decompressed_data)?;

        println!("✓ Saved tar file ({} bytes)", decompressed_data.len());
    }

    println!("Installation completed successfully!");
    Ok(())
}

/// Extract from self (embedded archive)
fn extract_from_self() -> Result<bool, Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Could not get executable directory")?;

    #[cfg(target_os = "windows")]
    {
        // On Windows, check for adjacent archive file first
        let exe_stem = exe_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Try .installer subdirectory first
        let installer_archive_path = exe_dir
            .join(".installer")
            .join(format!("{}.tar.zst", exe_stem));

        // Fallback to adjacent location
        let archive_path = exe_dir.join(format!("{}.tar.zst", exe_stem));

        let final_archive_path = if installer_archive_path.exists() {
            installer_archive_path
        } else {
            archive_path
        };

        // Check for metadata file
        let installer_metadata_path = exe_dir.join(".installer").join(format!("{}.metadata.json", exe_stem));
        let metadata_path = exe_dir.join(format!("{}.metadata.json", exe_stem));
        let final_metadata_path = if installer_metadata_path.exists() {
            installer_metadata_path
        } else {
            metadata_path
        };

        if final_metadata_path.exists() && final_archive_path.exists() {
            println!("Found adjacent archive file: {:?}", final_archive_path);

            // Read metadata
            let metadata_contents = fs::read_to_string(&final_metadata_path)?;
            let parsed: serde_json::Value = serde_json::from_str(&metadata_contents)?;

            let metadata = AppMetadata {
                identifier: parsed["identifier"].as_str().unwrap_or_default().to_string(),
                name: parsed["name"].as_str().unwrap_or_default().to_string(),
                channel: parsed["channel"].as_str().unwrap_or_default().to_string(),
                hash: parsed["hash"].as_str().map(String::from),
            };

            println!("Using metadata: identifier={}, name={}, channel={}",
                     metadata.identifier, metadata.name, metadata.channel);

            // Build app data directory
            let app_data_dir = get_app_data_dir()?;
            let app_base_dir = app_data_dir.join(&metadata.identifier).join(&metadata.channel);
            let self_extraction_dir = app_base_dir.join("self-extraction");
            let app_dir = app_base_dir.join("app");

            println!("Extracting to: {:?}", self_extraction_dir);
            println!("App will be installed to: {:?}", app_dir);

            // Read compressed data
            let file_size = fs::metadata(&final_archive_path)?.len();
            let compressed_data = if file_size < 1024 * 1024 * 1024 {
                // Read directly if less than 1GB
                fs::read(&final_archive_path)?
            } else {
                // For large files, use streaming
                let mut file = File::open(&final_archive_path)?;
                let mut compressed_data = Vec::new();
                let mut buffer = [0u8; 8192];
                loop {
                    match std::io::Read::read(&mut file, &mut buffer) {
                        Ok(0) => break,
                        Ok(n) => compressed_data.extend_from_slice(&buffer[..n]),
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e.into()),
                    }
                }
                compressed_data
            };

            extract_and_install(&compressed_data, &metadata, &self_extraction_dir, &app_dir)?;
            return Ok(true);
        }
    }

    // Fall back to embedded archive approach
    let mut self_file = File::open(&exe_path)?;
    let file_size = self_file.metadata()?.len();

    // Read entire file to find markers
    let mut search_buffer = vec![0u8; file_size as usize];
    self_file.read_exact(&mut search_buffer)?;

    // Find second occurrence of metadata marker
    let first_metadata_pos = memchr::memmem::find(&search_buffer, METADATA_MARKER)
        .ok_or("No metadata marker found")?;

    let search_start = first_metadata_pos + METADATA_MARKER.len();
    let second_metadata_offset = memchr::memmem::find(&search_buffer[search_start..], METADATA_MARKER)
        .ok_or("No second metadata marker found")?;

    let metadata_marker_pos = search_start + second_metadata_offset;
    let metadata_start = metadata_marker_pos + METADATA_MARKER.len();

    // Find archive marker
    let remaining_buffer = &search_buffer[metadata_start..];
    let archive_marker_offset = memchr::memmem::find(remaining_buffer, ARCHIVE_MARKER)
        .ok_or("Archive marker not found")?;

    let archive_offset = metadata_start + archive_marker_offset;

    // Read metadata
    let metadata = read_embedded_metadata(
        &mut self_file,
        metadata_start as u64,
        archive_offset as u64,
    )?;

    // Build app data directory
    let app_data_dir = get_app_data_dir()?;
    let app_base_dir = app_data_dir.join(&metadata.identifier).join(&metadata.channel);
    let self_extraction_dir = app_base_dir.join("self-extraction");
    let app_dir = app_base_dir.join("app");

    println!("Self-extracting archive found at offset {}", archive_offset);
    println!("Extracting to: {:?}", self_extraction_dir);

    // Read compressed data
    let archive_size = file_size as usize - (archive_offset + ARCHIVE_MARKER.len());
    let compressed_data = search_buffer[archive_offset + ARCHIVE_MARKER.len()..].to_vec();

    extract_and_install(&compressed_data, &metadata, &self_extraction_dir, &app_dir)?;
    Ok(true)
}

fn main() {
    println!("Electrobun self-extractor v1.3 starting...");

    let start_time = std::time::Instant::now();

    // Platform-specific extraction
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        match extract_from_self() {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("ERROR: Not a valid self-extracting installer");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS extraction logic
        println!("macOS extraction not fully implemented in this version");
    }

    println!("Total time: {:?}", start_time.elapsed());
}
