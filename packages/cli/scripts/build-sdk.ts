// Run this script via terminal or command line with bun scripts/build-sdk.ts

import { $ } from "bun";
import { platform, arch } from "os";
import { join, dirname, relative, basename } from "path";
import {
    existsSync,
    mkdirSync,
    readdirSync,
    readFileSync,
    renameSync,
    rmSync,
    statSync,
    unlinkSync,
    writeFileSync,
    cpSync,
} from "fs";
import { parseArgs } from "util";
import process from "process";
import { BUN_VERSION } from "../src/core/shared/bun-version";
import { WGPU_LIB_FILENAMES } from "./wgpu-shared";

console.log("building...", platform(), arch());

const { values: args } = parseArgs({
    args: Bun.argv,
    options: {
        release: {
            type: "boolean",
        },
        ci: {
            type: "boolean",
        },
        npm: {
            type: "boolean",
        },
    },
    allowPositionals: true,
});

// TODO: set via cl arg
const CHANNEL: "debug" | "release" = args.release ? "release" : "debug";
const IS_NPM_BUILD = args.npm || false;
const OS: "win" | "linux" | "macos" = getPlatform();
const ARCH: "arm64" | "x64" = getArch();

const isWindows = platform() === "win32";
const binExt = OS === "win" ? ".exe" : "";
const bunBin = isWindows ? "bun.exe" : "bun";

// Note: We want all binaries in /dist to be extensionless to simplify our cross platform code
// (no .exe on windows)

// PATHS
const PATH = {
    bun: {
        RUNTIME: join(process.cwd(), "vendors", "bun", bunBin),
        DIST: join(process.cwd(), "dist", bunBin),
    },
};

// Minimum expected file sizes for downloaded archives (in bytes)
// These are sanity checks to detect failed downloads (e.g., HTML error pages)
const MIN_DOWNLOAD_SIZES: Record<string, number> = {
    bun: 10 * 1024 * 1024, // Bun zip should be > 10MB
    wgpu: 1 * 1024 * 1024, // Dawn (WGPU) tarball should be > 1MB
    cef: 50 * 1024 * 1024, // CEF tarball should be > 50MB
};

function validateDownload(filePath: string, type: string): void {
    if (!existsSync(filePath)) {
        throw new Error(`Download failed: ${filePath} does not exist`);
    }
    const stats = statSync(filePath);
    const minSize = MIN_DOWNLOAD_SIZES[type];
    if (minSize && stats.size < minSize) {
        // Remove the invalid file so next run will re-download
        unlinkSync(filePath);
        throw new Error(
            `Download failed: ${filePath} is only ${stats.size} bytes (expected > ${minSize} bytes). ` +
            `Please try again in a minute.`,
        );
    }
}

// Pause between GitHub downloads to avoid rate limiting
// Track if we've done a GitHub download this session
let lastGitHubDownload = 0;

async function pauseForGitHub(): Promise<void> {
    const now = Date.now();
    const timeSinceLastDownload = now - lastGitHubDownload;
    const pauseDuration = 60000; // 60 seconds

    if (lastGitHubDownload > 0 && timeSinceLastDownload < pauseDuration) {
        const remainingPause = pauseDuration - timeSinceLastDownload;
        console.log(
            `Pausing ${Math.ceil(remainingPause / 1000)} seconds before next GitHub download...`,
        );
        await new Promise((resolve) => setTimeout(resolve, remainingPause));
    }
    lastGitHubDownload = Date.now();
}

// TODO: setup file watchers
try {
    if (IS_NPM_BUILD) {
        console.log("Building for npm (JS/TS files only)...");
        await buildForNpm();
    } else {
        await setup();
        await build();
        await copyToDist();
    }
} catch (err) {
    console.error(err);
    process.exit(1);
}

// Global variables to store build tool paths
var CMAKE_BIN = "cmake";

async function vendorCmake() {
    if (OS !== "macos") return;

    // On macOS, cmake is distributed as an app bundle
    const vendoredCmakePath = join(
        process.cwd(),
        "vendors",
        "cmake",
        "CMake.app",
        "Contents",
        "bin",
        "cmake",
    );

    // Check if cmake is already available (system or vendored)
    try {
        await $`which cmake`.quiet();
        console.log("✓ cmake found in system PATH");
        CMAKE_BIN = "cmake";
        return;
    } catch {
        // Not in system PATH, check if vendored
        if (existsSync(vendoredCmakePath)) {
            CMAKE_BIN = vendoredCmakePath;
            console.log("✓ Using vendored cmake");
            return;
        }
    }

    console.log("cmake not found, downloading...");

    try {
        const cmakeVersion = "3.30.2";
        const cmakeUrl = `https://github.com/Kitware/CMake/releases/download/v${cmakeVersion}/cmake-${cmakeVersion}-macos-universal.tar.gz`;

        await $`mkdir -p vendors`;
        console.log(`Downloading cmake ${cmakeVersion} for macOS...`);

        // Download and extract in vendors directory
        const tempFile = "vendors/cmake_temp.tar.gz";
        await $`curl -L "${cmakeUrl}" -o "${tempFile}"`;

        // Extract in vendors directory
        await $`cd vendors && tar -xzf cmake_temp.tar.gz`;

        // Always clean up the temp file
        await $`rm -f vendors/cmake_temp.tar.gz`;

        // Rename to simple 'cmake' directory if needed
        const extractedDir = `vendors/cmake-${cmakeVersion}-macos-universal`;
        if (existsSync(extractedDir)) {
            await $`rm -rf vendors/cmake`; // Remove old cmake if exists
            await $`mv "${extractedDir}" vendors/cmake`;
        }

        // Set the cmake binary path
        CMAKE_BIN = vendoredCmakePath;

        // Verify it works
        await $`"${CMAKE_BIN}" --version`;
        console.log("✓ cmake vendored successfully");
    } catch (error) {
        console.error("Failed to vendor cmake:", error);
        throw new Error("Could not vendor cmake. Please install it manually.");
    }
}

// Global variable to store vcvarsall path
var VCVARSALL_PATH = "";

async function findMsvcTools() {
    if (OS !== "win") return;

    // 先尝试用 vswhere 查找
    let foundPath = "";
    try {
        const vswherePath = join(
            process.env["ProgramFiles(x86)"] || "C:\\Program Files (x86)",
            "Microsoft Visual Studio", "Installer", "vswhere.exe",
        );
        if (existsSync(vswherePath)) {
            const result = await $`powershell -command "& '${vswherePath}' -latest -products * -requir
  Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath"`.quiet();
            if (result.exitCode === 0 && result.stdout.toString().trim()) {
                foundPath = result.stdout.toString().trim();
            }
        }
    } catch { }

    // vswhere 没找到则搜索常用路径
    if (!foundPath) {
        const paths = [
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community",
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools",
            "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\Community",
            "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools",
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional",
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise",
        ];
        for (const vsPath of paths) {
            if (existsSync(join(vsPath, "VC", "Auxiliary", "Build", "vcvarsall.bat"))) {
                foundPath = vsPath;
                break;
            }
        }
    }

    if (foundPath) {
        VCVARSALL_PATH = join(foundPath, "VC", "Auxiliary", "Build", "vcvarsall.bat");
        if (existsSync(VCVARSALL_PATH)) {
            console.log("✓ Found MSVC tools with vcvarsall.bat");
            return;
        }
    }

    // 最终 fallback
    try {
        await $`where cl.exe`.quiet();
        console.log("⚠ vcvarsall.bat 未找到，但 cl.exe 可用");
    } catch {
        console.log("MSVC 编译器未找到");
    }
}

async function runMsvcCommand(command: string) {
    if (VCVARSALL_PATH) {
        // 有 vcvarsall: 用完整环境
        const tempBat = join(process.cwd(), "temp_build_cmd.bat");
        writeFileSync(tempBat, `@echo off\ncall "${VCVARSALL_PATH}" x64 >nul\n${command}`);
        try {
            const result = await $`cmd /c "${tempBat}"`;
            await $`rm "${tempBat}"`.catch(() => { });
            return result;
        } catch (error) {
            await $`rm "${tempBat}"`.catch(() => { });
            throw error;
        }
    }

    // 没有 vcvarsall: 直接 cmd /c（cl.exe 在 PATH 就能用）
    console.log("直接运行 MSVC 命令（未加载 vcvarsall 环境）...");
    const noEnvBat = join(process.cwd(), "temp_build_cmd.bat");
    writeFileSync(noEnvBat, `@echo off\n${command}`);
    try {
        const result = await $`cmd /c "${noEnvBat}"`;
        await $`rm "${noEnvBat}"`.catch(() => { });
        return result;
    } catch (error) {
        await $`rm "${noEnvBat}"`.catch(() => { });
        throw new Error(
            `MSVC 命令执行失败:\n  ${command}\n\n` +
            "请安装 Visual Studio 2022 Build Tools 或\n" +
            `在"x64 Native Tools Command Prompt"中运行 bun scripts/build-sdk.ts。`,
        );
    }
}

function getWindowsCmakeGenerator() {
    // Prefer a toolchain-driven generator over the Visual Studio IDE generator.
    // On CI we may have MSVC Build Tools + vcvarsall without a full VS instance
    // that CMake can discover for `-G "Visual Studio 17 2022"`.
    return VCVARSALL_PATH ? "NMake Makefiles" : "Visual Studio 17 2022";
}

async function installWindowsDeps() {
    const scriptPath = join(process.cwd(), "scripts", "install-windows-deps.ps1");
    if (!existsSync(scriptPath)) {
        console.error(`Installer script not found: ${scriptPath}`);
        throw new Error(
            "Windows installer script missing. Please run the installer manually.",
        );
    }

    console.log(
        "Running Windows dependency installer (may require Administrator privileges)...",
    );
    try {
        // Run the PowerShell helper (it will request elevation if needed)
        await $`powershell -ExecutionPolicy Bypass -NoProfile -File "${scriptPath}"`;
        console.log(
            "Windows dependency installer finished. Re-checking dependencies...",
        );
    } catch (err) {
        console.error("Windows installer failed:", err);
        throw err;
    }
}

async function checkDependencies() {
    const missingDeps: string[] = [];

    if (OS === "macos") {
        // Try to vendor cmake if not available
        await vendorCmake();

        // Check for make (should be available with Xcode command line tools)
        try {
            await $`which make`.quiet();
        } catch {
            missingDeps.push(
                "make (install Xcode Command Line Tools: xcode-select --install)",
            );
        }
    } else if (OS === "win") {
        // Find MSVC compiler tools
        await findMsvcTools();

        // Check for cmake
        try {
            await $`where cmake`.quiet();
            CMAKE_BIN = "cmake";
        } catch {
            missingDeps.push("cmake");
        }

        // Check for Visual Studio (use vswhere if available)
        let vsFound = false;
        try {
            const vswherePath = join(
                process.env["ProgramFiles(x86)"] || "C:\\Program Files (x86)",
                "Microsoft Visual Studio",
                "Installer",
                "vswhere.exe",
            );
            if (existsSync(vswherePath)) {
                // Use PowerShell wrapper to ensure output is captured correctly on Windows
                const out =
                    await $`powershell -command "& '${vswherePath}' -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath"`.quiet();
                if (out.exitCode === 0 && out.stdout.toString().trim()) vsFound = true;
            } else {
                const out =
                    await $`vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`.quiet();
                if (out.exitCode === 0 && out.stdout.toString().trim()) vsFound = true;
            }
        } catch {
            vsFound = false;
        }

        if (!vsFound) missingDeps.push("visual-studio");

        if (missingDeps.length > 0) {
            // In CI we should not attempt interactive installs
            if (process.env["GITHUB_ACTIONS"]) {
                console.warn(
                    "\n⚠️  Missing required dependencies in CI - continuing (CI should provide these)",
                );
            } else {
                try {
                    await installWindowsDeps();
                } catch {
                    console.error("Auto-install failed or was cancelled.");
                }

                // Re-check cmake
                const newMissing: string[] = [];
                try {
                    await $`where cmake`.quiet();
                    CMAKE_BIN = "cmake";
                } catch {
                    newMissing.push("cmake");
                }

                // Re-check Visual Studio
                try {
                    const vswherePath = join(
                        process.env["ProgramFiles(x86)"] || "C:\\Program Files (x86)",
                        "Microsoft Visual Studio",
                        "Installer",
                        "vswhere.exe",
                    );
                    let out;
                    if (existsSync(vswherePath)) {
                        // Use PowerShell wrapper to ensure output is captured correctly on Windows
                        out =
                            await $`powershell -command "& '${vswherePath}' -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath"`.quiet();
                    } else {
                        out =
                            await $`vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`.quiet();
                    }
                    if (!(out && out.exitCode === 0 && out.stdout.toString().trim())) {
                        newMissing.push("visual-studio");
                    }
                } catch {
                    newMissing.push("visual-studio");
                }

                if (newMissing.length > 0) {
                    missingDeps.length = 0;
                    newMissing.forEach((m) => missingDeps.push(m));
                } else {
                    // clear missingDeps if everything is present now
                    missingDeps.length = 0;
                }
            }
        }
    } else if (OS === "linux") {
        // Check for build essentials
        try {
            await $`which cmake`.quiet();
            CMAKE_BIN = "cmake";
        } catch {
            missingDeps.push("cmake");
        }
        try {
            await $`which make`.quiet();
        } catch {
            missingDeps.push("make");
        }
        try {
            await $`which gcc`.quiet();
        } catch {
            missingDeps.push("build-essential");
        }
    }

    if (missingDeps.length > 0) {
        console.error("\n⚠️  Missing required dependencies:");
        missingDeps.forEach((dep) => console.error(`  • ${dep}`));

        if (OS === "macos") {
            console.error("\nTo install missing dependencies on macOS:");
            console.error("• For make: Install Xcode Command Line Tools");
            console.error("   xcode-select --install");
        } else if (OS === "win") {
            console.error("\nTo install missing dependencies on Windows:");
            console.error("1. Install Visual Studio 2022 with C++ development tools");
            console.error("2. Install cmake from: https://cmake.org/download/");
        } else if (OS === "linux") {
            console.error("\nTo install missing dependencies on Linux:");
            console.error(
                "   sudo apt update && sudo apt install -y build-essential cmake",
            );
        }

        // In CI, just warn but continue; locally throw an error
        if (process.env["GITHUB_ACTIONS"]) {
            console.warn(
                "\n⚠️  Running in CI - continuing despite missing dependencies",
            );
            console.warn(
                "   The CI workflow should have already installed these dependencies",
            );
        } else {
            throw new Error(
                "Missing required dependencies. Please install them and try again.",
            );
        }
    }

    console.log("✓ All required dependencies found");
}

async function setup() {
    await checkDependencies();
    // Run vendors sequentially to avoid network/curl conflicts
    // GitHub downloads have built-in pauses to avoid rate limiting
    await vendorBun(); // GitHub
    await vendorWGPU(); // GitHub
    // CEF 已废弃，由 wry (WebView2/WebKit) 替代
    // await vendorCEF(); // Spotify CDN (not GitHub)
    await vendorWebview2();
    await vendorLinuxDeps();
}

async function build() {
    await createDistFolder();
    await BunInstall();

    // Generate template embeddings before building CLI
    console.log("Generating template embeddings...");
    await generateTemplateEmbeddings();

    // Build preload script (compiles TypeScript to JS for webview injection)
    console.log("Building preload script...");
    await buildPreload();

    await Promise.all([
        buildBuildTools(),
        buildSelfExtractor(),
        buildCore(),
        buildLauncher(),
        buildCli(),
        buildMainJs(),
    ]);
}

async function buildForNpm() {
    console.log("Creating dist folder for npm...");
    await createDistFolder();

    console.log("Building main.js...");
    await buildMainJs();

    // Build preload script (compiles TypeScript to JS for webview injection)
    // Must run before copyApiFiles so the generated file is included
    console.log("Building preload script...");
    await buildPreload();

    console.log("Copying API files...");
    await copyApiFiles();

    console.log(
        "npm build complete! dist/ contains main.js and api/ folder (bun, browser, shared APIs).",
    );
}

async function copyApiFiles() {
    // Copy TypeScript APIs from monorepo packages (core, browser to dist/api/)
    // 使用 fs.cpSync 以确保跨平台一致性（cp -R 的语义在 Win/Unix 上不同）
    const apiSrcDirs = ["src/core/bun", "src/core/browser", "src/core/shared"];
    mkdirSync("dist/api", { recursive: true });
    for (const src of apiSrcDirs) {
        cpSync(src, join("dist/api", basename(src)), {
            recursive: true,
            dereference: true,
            force: true,
        });
    }
}


async function copyToDist() {
    // Bun runtime
    await $`cp ${PATH.bun.RUNTIME} ${PATH.bun.DIST}`;
    const sourceDir = CHANNEL === "release" ? "release" : "debug";
    // Rust workspace builds output to root target/
    await $`cp ../../target/${sourceDir}/launcher${binExt} dist/launcher${binExt}`;

    const extractorTarget = OS === "win" ? `x86_64-pc-windows-msvc/${sourceDir}` : sourceDir;
    await $`cp ../../target/${extractorTarget}/extractor${binExt} dist/extractor${binExt}`;
    const coreLibName =
        OS === "win"
            ? "electrobun_core.dll"
            : OS === "macos"
                ? "libelectrobun_core.dylib"
                : "libelectrobun_core.so";
    await $`cp ${join("../..", "target", sourceDir, coreLibName)} ${join("dist", coreLibName)}`;
    // Copy bsdiff, bspatch, rust-zstd from Rust build
    const toolSource = CHANNEL === "release" ? "release" : "debug";
    await $`cp ../../target/${toolSource}/bsdiff${binExt} dist/bsdiff${binExt}`;
    await $`cp ../../target/${toolSource}/bspatch${binExt} dist/bspatch${binExt}`;
    await $`cp ../../target/${toolSource}/rust-zstd${binExt} dist/rust-zstd${binExt}`;

    // Copy Rust-built ASAR CLI and shared library
    if (OS === "win") {
        // Windows: single arch (Rust binary is native)
        await $`cp ../../target/${toolSource}/rust-asar${binExt} dist/rust-asar${binExt}`;
        await $`cp ../../target/${toolSource}/asar.dll dist/asar.dll`;
        await $`mkdir -p dist-win-x64/rust-asar/x64`;
        await $`cp dist/asar.dll dist-win-x64/rust-asar/x64/asar.dll`;
        await $`mkdir -p dist-win-x64/rust-asar/x64`;
    } else {
        // Unix: copy CLI and shared library
        var asarLibName = OS === "macos" ? "libasar.dylib" : "libasar.so";
        await $`cp ../../target/${toolSource}/rust-asar${binExt} dist/rust-asar${binExt}`;
        await $`cp ../../target/${toolSource}/${asarLibName} dist/${asarLibName}`;
    }
    // Verify critical files were copied
    if (OS === "macos") {
        const launcherPath = join("dist", `launcher${binExt}`);
        if (!existsSync(launcherPath)) {
            throw new Error(`launcher${binExt} was not copied to ${launcherPath}`);
        }
        console.log(`launcher${binExt} copied successfully to ${launcherPath}`);
    }
    // Electrobun cli and npm launcher
    // npmbin launcher — 不再需要（package/src/npmbin/ 已废弃）
    // await $`cp src/npmbin/index.js dist/npmbin.js`;
    await $`cp build/electrobun${binExt} dist/electrobun${binExt}`;
    // Also copy to bin/ so the compiled binary is available for fallback
    await $`mkdir -p bin && cp build/electrobun${binExt} bin/electrobun${binExt}`;
    // Electrobun's Typescript bun and browser apis
    await copyApiFiles();
    // Native code and frameworks
    if (OS === "macos") {
        // process_helper for macOS（可选 — 仅当存在时才复制）
        if (existsSync("src/native/build/process_helper")) {
            await $`cp -R src/native/build/process_helper dist/process_helper`;
        }
    } else if (OS === "win") {
        // WebView2Loader for Windows (wry uses WebView2)
        const webview2Arch = "x64";
        await $`cp vendors/webview2/Microsoft.Web.WebView2/build/native/${webview2Arch}/WebView2Loader.dll dist/WebView2Loader.dll`;
    } else if (OS === "linux") {
        // Linux 使用 wry/WebKitGTK，无需额外 native 库
        console.log("Linux: using wry/WebKitGTK (no native wrapper needed)");
    }

    // Create platform-specific dist folder and copy all files
    await createPlatformDistFolder();
}

async function createPlatformDistFolder() {
    // Create platform-specific dist folder (e.g., dist-linux-arm64)
    const platformDistDir = `dist-${OS}-${ARCH}`;
    console.log(`Creating platform-specific dist folder: ${platformDistDir}`);

    // 先用 Bun/Bash mkdir（兼容跨平台）
    await $`mkdir -p ${platformDistDir}`;

    // 用 fs.cpSync 复制所有文件（跨平台统一，无需 PowerShell/rsync）
    const distItems = readdirSync("dist");
    for (const item of distItems) {
        const srcPath = join("dist", item);
        const destPath = join(platformDistDir, item);
        cpSync(srcPath, destPath, {
            recursive: true,
            dereference: true,
            force: true,
        });
    }

    console.log(`Successfully created and populated ${platformDistDir}`);
}

function getPlatform() {
    switch (platform()) {
        case "win32":
            return "win";
        case "darwin":
            return "macos";
        case "linux":
            return "linux";
        default:
            throw new Error("unsupported platform");
    }
}

function getArch() {
    switch (arch()) {
        case "arm64":
            return "arm64";
        case "x64":
            return "x64";
        default:
            throw new Error("unsupported arch");
    }
}

async function createDistFolder() {
    // Use Bun's fs APIs instead of shell rm for reliability on Windows
    const distPath = join(process.cwd(), "dist");
    if (existsSync(distPath)) {
        rmSync(distPath, { recursive: true, force: true });
    }
    await $`mkdir -p dist/api`;
    await $`mkdir -p dist/api/bun`;
    await $`mkdir -p dist/api/browser`;
}

async function BunInstall() {
    // Use vendored Bun for consistency with CI
    await $`${PATH.bun.RUNTIME} install`;
}

async function vendorBun() {
    // Check if vendored Bun version matches expected version.
    // When the hardcoded version is bumped (e.g. after a git pull),
    // this detects the mismatch and forces a clean re-vendor.
    const bunDir = join(process.cwd(), "vendors", "bun");
    const bunVersionFile = join(bunDir, ".bun-version");

    if (existsSync(PATH.bun.RUNTIME)) {
        if (existsSync(bunVersionFile)) {
            const vendoredVersion = readFileSync(bunVersionFile, "utf-8").trim();
            if (vendoredVersion !== BUN_VERSION) {
                console.log(
                    `Bun version mismatch: vendored "${vendoredVersion}" vs expected "${BUN_VERSION}"`,
                );
                console.log("Cleaning stale Bun binary and re-vendoring...");
                unlinkSync(PATH.bun.RUNTIME);
            } else {
                return;
            }
        } else {
            // Binary exists but no version stamp (legacy state) — write one and keep going
            mkdirSync(bunDir, { recursive: true });
            writeFileSync(bunVersionFile, BUN_VERSION);
            return;
        }
    }

    await pauseForGitHub();

    let bunUrlSegment: string;
    let bunDirName: string;

    if (OS === "win") {
        // Use baseline x64 for Windows to ensure ARM64 compatibility
        bunUrlSegment = "bun-windows-x64-baseline.zip";
        bunDirName = "bun-windows-x64-baseline";
    } else if (OS === "macos") {
        bunUrlSegment =
            ARCH === "arm64" ? "bun-darwin-aarch64.zip" : "bun-darwin-x64.zip";
        bunDirName = ARCH === "arm64" ? "bun-darwin-aarch64" : "bun-darwin-x64";
    } else if (OS === "linux") {
        bunUrlSegment =
            ARCH === "arm64" ? "bun-linux-aarch64.zip" : "bun-linux-x64.zip";
        bunDirName = ARCH === "arm64" ? "bun-linux-aarch64" : "bun-linux-x64";
    } else {
        throw new Error(`Unsupported platform: ${OS}`);
    }

    const tempZipPath = join("vendors", "bun", "temp.zip");
    const extractDir = join("vendors", "bun");

    // Download zip file
    await $`mkdir -p ${extractDir} && curl -L -o ${tempZipPath} https://github.com/oven-sh/bun/releases/download/bun-v${BUN_VERSION}/${bunUrlSegment}`;

    // Validate download
    validateDownload(tempZipPath, "bun");

    // Extract zip file
    if (isWindows) {
        // Use PowerShell to extract zip on Windows
        await $`powershell -command "Expand-Archive -Path ${tempZipPath} -DestinationPath ${extractDir} -Force"`;
    } else {
        // Use unzip on macOS/Linux
        await $`unzip -o ${tempZipPath} -d ${extractDir}`;
    }

    // Move the bun binary to the correct location
    // The path inside the zip might be different depending on the platform
    if (isWindows) {
        await $`mv ${join("vendors", "bun", bunDirName, "bun.exe")} ${PATH.bun.RUNTIME}`;
    } else {
        await $`mv ${join("vendors", "bun", bunDirName, "bun")} ${PATH.bun.RUNTIME}`;
    }

    // Add execute permissions on non-Windows platforms
    if (!isWindows) {
        await $`chmod +x ${PATH.bun.RUNTIME}`;
    }

    // Clean up
    await $`rm ${tempZipPath}`;
    await $`rm -rf ${join("vendors", "bun", bunDirName)}`;

    // Write version stamp so future builds can detect staleness
    writeFileSync(join("vendors", "bun", ".bun-version"), BUN_VERSION);
}
async function vendorWGPU() {
    const WGPU_VERSION = "0.2.3";
    const wgpuBaseDir = join(process.cwd(), "vendors", "wgpu");
    const wgpuDir = join(wgpuBaseDir, `${OS}-${ARCH}`);
    const wgpuVersionFile = join(wgpuBaseDir, ".wgpu-version");
    const currentVersion = existsSync(wgpuVersionFile)
        ? readFileSync(wgpuVersionFile, "utf8").trim()
        : null;

    // 用共享常量中的文件名构建 vendor 候选路径（运行时文件名见 wgpu-shared.js）
    const osToWgpuKey: Record<string, "darwin" | "win32" | "linux"> = {
        macos: "darwin",
        win: "win32",
        linux: "linux",
    };
    const platformBaseNames = WGPU_LIB_FILENAMES[osToWgpuKey[OS] ?? "linux"];
    const libExt = OS === "win" ? ".dll" : OS === "macos" ? ".dylib" : ".so";
    const libCandidates =
        OS === "win"
            ? [
                // Windows 同时检查 bin/ 和 lib/ 子目录
                ...platformBaseNames.flatMap((name) => [
                    join(wgpuDir, "bin", name),
                    join(wgpuDir, "lib", name),
                ]),
            ]
            : [
                // macOS/Linux 只检查 lib/ 子目录
                ...platformBaseNames.map((name) => join(wgpuDir, "lib", name)),
                // _shared 变体仅构建产物，不在运行时常量中
                join(wgpuDir, "lib", `libwebgpu_dawn_shared${libExt}`),
            ];

    if (libCandidates.some((p) => existsSync(p)) && currentVersion === WGPU_VERSION) {
        return;
    }

    if (libCandidates.some((p) => existsSync(p)) && !currentVersion) {
        writeFileSync(wgpuVersionFile, WGPU_VERSION);
        return;
    }

    if (currentVersion && currentVersion !== WGPU_VERSION && existsSync(wgpuDir)) {
        await $`rm -rf "${wgpuDir}"`;
    }

    await pauseForGitHub();
    console.log("Downloading electrobun-dawn binaries...");

    const platformMap: Record<string, string> = {
        macos: "darwin",
        win: "win32",
        linux: "linux",
    };
    const platformName = platformMap[OS];
    const archName = ARCH;

    const tarballUrl = `https://github.com/blackboardsh/electrobun-dawn/releases/download/v${WGPU_VERSION}/electrobun-dawn-${platformName}-${archName}.tar.gz`;
    const tempTarball = join("vendors", `electrobun-dawn-temp.tar.gz`);
    const tempExtractDir = join("vendors", `electrobun-dawn-extract-${Date.now()}`);

    try {
        await $`mkdir -p "${wgpuBaseDir}"`;
        await $`rm -f "${tempTarball}"`;

        const githubToken =
            process.env["GITHUB_TOKEN"] ??
            process.env["GH_TOKEN"] ??
            process.env["GITHUB_ACCESS_TOKEN"];
        if (githubToken) {
            await $`curl -fL -H "Authorization: Bearer ${githubToken}" -H "Accept: application/octet-stream" "${tarballUrl}" -o "${tempTarball}"`;
        } else {
            await $`curl -fL -H "Accept: application/octet-stream" "${tarballUrl}" -o "${tempTarball}"`;
        }

        validateDownload(tempTarball, "wgpu");

        await $`rm -rf "${tempExtractDir}"`;
        await $`mkdir -p "${tempExtractDir}"`;
        await $`tar -xzf "${tempTarball}" -C "${tempExtractDir}"`;

        const extracted = readdirSync(tempExtractDir);
        if (extracted.length === 1) {
            const single = join(tempExtractDir, extracted[0]!);
            if (existsSync(wgpuDir)) {
                await $`rm -rf "${wgpuDir}"`;
            }
            await $`mv "${single}" "${wgpuDir}"`;
        } else {
            if (existsSync(wgpuDir)) {
                await $`rm -rf "${wgpuDir}"`;
            }
            await $`mkdir -p "${wgpuDir}"`;
            for (const item of extracted) {
                await $`mv "${join(tempExtractDir, item)}" "${wgpuDir}/"`;
            }
        }

        await $`rm -rf "${tempExtractDir}"`;
        await $`rm -f "${tempTarball}"`;

        if (!libCandidates.some((p) => existsSync(p))) {
            throw new Error(`WGPU library not found after extraction: ${wgpuDir}`);
        }

        writeFileSync(wgpuVersionFile, WGPU_VERSION);

        // Regenerate Bun FFI bindings when WGPU version changes
        if (!existsSync(join(process.cwd(), "src", "core", "bun", "webGPU.ts"))) {
            await $`bun scripts/gen-webgpu-ffi.ts`;
        } else if (currentVersion !== WGPU_VERSION) {
            await $`bun scripts/gen-webgpu-ffi.ts`;
        }

        console.log("✓ electrobun-dawn binaries downloaded successfully");
    } catch (error: unknown) {
        console.error(
            "Failed to download electrobun-dawn binaries:",
            error instanceof Error ? error.message : error,
        );
        throw new Error(
            `Failed to download electrobun-dawn binaries. Please try again in a minute.`,
        );
    }
}




async function vendorNuget() {
    if (OS === "win") {
        if (existsSync(join(process.cwd(), "vendors", "nuget", "nuget.exe"))) {
            return;
        }

        // install nuget package manager
        await $`mkdir -p vendors/nuget && curl -L -o vendors/nuget/nuget.exe https://dist.nuget.org/win-x86-commandline/latest/nuget.exe`;
    }
}

async function vendorWebview2() {
    if (OS === "win") {
        if (existsSync(join(process.cwd(), "vendors", "webview2"))) {
            return;
        }

        await vendorNuget();

        // install nuget package manager
        await $`vendors/nuget/nuget.exe install Microsoft.Web.WebView2 -OutputDirectory vendors/webview2`;

        const webview2BasePath = "./vendors/webview2";
        const webview2Dir = readdirSync(webview2BasePath).find((dir: string) =>
            dir.startsWith("Microsoft.Web.WebView2"),
        );

        if (webview2Dir && webview2Dir !== "Microsoft.Web.WebView2") {
            const oldPath = join(webview2BasePath, webview2Dir);
            const newPath = join(webview2BasePath, "Microsoft.Web.WebView2");

            try {
                renameSync(oldPath, newPath);
                console.log(`Renamed ${webview2Dir} to Microsoft.Web.WebView2`);
            } catch (error) {
                console.error("Error renaming folder:", error);
            }
        }
    }
}

async function vendorLinuxDeps() {
    if (OS === "linux") {
        // We can't check the package manager of every Linux distro,
        // so lets just do Ubuntu/Debian for now since thats what CI uses.

        const requiredPackages = [
            "build-essential",
            "cmake",
            "pkg-config",
            "libgtk-3-dev",
            "libwebkit2gtk-4.1-dev",
            "libayatana-appindicator3-dev",
            "librsvg2-dev",
            "fuse",
            "libfuse2",
        ];

        const distroInfo = await $`grep -E '^(ID|ID_LIKE)=' /etc/os-release`.catch(
            () => null,
        );
        if (
            !distroInfo ||
            !(
                String(distroInfo.stdout).includes("debian") ||
                String(distroInfo.stdout).includes("ubuntu")
            )
        ) {
            console.log(
                "Cannot determine Linux distro or not Debian/Ubuntu based - skipping automatic dependency check",
            );
            console.log(
                `Please ensure required packages are installed: ${requiredPackages.join(", ")}`,
            );
            return;
        }

        console.log("Detected Debian/Ubuntu based Linux. Checking dependencies...");
        const missingPackages: string[] = [];
        for (const pkg of requiredPackages) {
            const result = await $`dpkg -l | grep ${pkg}`.catch(() => null);
            if (!result || String(result.stdout).trim() === "") {
                missingPackages.push(pkg);
            }
        }
        if (missingPackages.length > 0) {
            console.log("");
            console.log(
                "═══════════════════════════════════════════════════════════════",
            );
            console.log("🚨 MISSING REQUIRED LINUX DEPENDENCIES");
            console.log(
                "═══════════════════════════════════════════════════════════════",
            );
            console.log(`Missing packages: ${missingPackages.join(", ")}`);
            console.log("");
            console.log("Please install them using:");
            console.log(
                `   sudo apt update && sudo apt install -y ${missingPackages.join(" ")}`,
            );
            console.log("");

            // Check specifically for libfuse2 since it affects AppImage creation
            if (missingPackages.includes("libfuse2")) {
                console.log("⚠️  libfuse2 is required for AppImage creation");
                console.log(
                    "   Without it, AppImage generation will fail with FUSE errors",
                );
                console.log("");
            }

            // In CI, just warn but continue; locally show message and continue
            if (process.env["GITHUB_ACTIONS"]) {
                console.warn("⚠️  Running in CI - continuing despite missing packages");
                console.warn(
                    "   The CI workflow should have already installed these packages",
                );
            } else {
                console.warn("⚠️  Some features may not work without these packages");
                console.warn("   Continuing with build...");
            }
            console.log(
                "═══════════════════════════════════════════════════════════════",
            );
            console.log("");
        }
        console.log("All required packages are installed");
    }
}

async function buildBuildTools() {
    console.log(`Building build tools (asar/bsdiff/bspatch/zstd) for ${OS} ${ARCH} with Cargo...`);
    const cargoArgs = CHANNEL === "release" ? ["--release"] : [];
    await $`cd ../.. && cargo build --package  electrobun-build-tools ${cargoArgs}`;
    console.log("✓ Build tools (asar/bsdiff/bspatch/zstd) built successfully");
}

async function buildLauncher() {
    console.log(`Building launcher for ${OS} ${ARCH} with Cargo...`);
    const cargoArgs = CHANNEL === "release" ? ["--release"] : [];
    await $`cd ../.. && cargo build --package electrobun-launcher ${cargoArgs}`;
}

async function buildCore() {
    console.log(`Building ElectrobunCore for ${OS} ${ARCH} with Cargo...`);
    const cargoArgs = CHANNEL === "release" ? ["--release"] : [];
    await $`cd ../.. && cargo build --package electrobun-core ${cargoArgs}`;
}

async function buildMainJs() {
    const bunModule = await import("bun");
    const result = await bunModule.build({
        entrypoints: [join("..", "..", "crates", "electrobun-launcher", "src", "main.ts")],
        outdir: join("dist"),
        external: [],
        // minify: true, // todo (yoav): add minify in canary and prod builds
        target: "bun",
    });

    // Verify main.js was created
    const mainJsPath = join("dist", "main.js");
    if (!existsSync(mainJsPath)) {
        throw new Error(
            `main.js was not created at ${mainJsPath}. Build result: ${JSON.stringify(result)}`,
        );
    }
    console.log(`main.js built successfully at ${mainJsPath}`);

    return result;
}

async function buildSelfExtractor() {
    // Build extractor with Cargo (Rust)
    const cargoArgs = CHANNEL === "release" ? ["--release"] : [];
    const targetArg = OS === "win" ? ["--target", "x86_64-pc-windows-msvc"] : [];

    await $`cd ../.. && cargo build --package electrobun-extractor ${cargoArgs} ${targetArg}`;
}

async function buildCli() {
    // Use system Bun for building CLI
    await $`mkdir -p build`;
    await $`bun build src/cli/index.ts --compile --outfile build/electrobun`;
}

async function buildPreload() {
    // The preload scripts (drag regions, internal RPC, encryption, webview tags) are written
    // in TypeScript for maintainability. We pre-compile them here because:
    // 1. At runtime, the app runs from an ASAR bundle where source .ts files don't exist
    // 2. Only the bundled JS is shipped, so Bun.build() can't compile at runtime
    // The compiled outputs are imported by native.ts and injected into webviews.
    //
    // Two variants are compiled:
    // - preloadScript: Full preload for trusted webviews (RPC, encryption, webview tags)
    // - preloadScriptSandboxed: Minimal preload for sandboxed/untrusted webviews (events only)
    const preloadDir = join(process.cwd(), "src", "core", "bun", "preload");
    const outputDir = join(preloadDir, ".generated");
    const outputPath = join(outputDir, "compiled.ts");

    // Ensure output directory exists
    mkdirSync(outputDir, { recursive: true });

    const bunModule = await import("bun");

    // Build full preload (trusted webviews)
    const fullPreloadEntry = join(preloadDir, "index.ts");
    const fullResult = await bunModule.build({
        entrypoints: [fullPreloadEntry],
        target: "browser",
        format: "esm",
        minify: false,
    });

    if (!fullResult.success) {
        console.error("Full preload build failed:", fullResult.logs);
        throw new Error("Failed to build full preload script");
    }

    // Build sandboxed preload (untrusted webviews)
    const sandboxedPreloadEntry = join(preloadDir, "index-sandboxed.ts");
    const sandboxedResult = await bunModule.build({
        entrypoints: [sandboxedPreloadEntry],
        target: "browser",
        format: "esm",
        minify: false,
    });

    if (!sandboxedResult.success) {
        console.error("Sandboxed preload build failed:", sandboxedResult.logs);
        throw new Error("Failed to build sandboxed preload script");
    }

    // Wrap in IIFE to prevent top-level variables from leaking into webview global scope
    // (Bun removed iife format support in 1.3.10, so we build as esm and wrap manually)
    const fullPreloadJs = `(function(){${await fullResult.outputs[0].text()}})();`;
    const sandboxedPreloadJs = `(function(){${await sandboxedResult.outputs[0].text()}})();`;
    const distDir = join(process.cwd(), "dist");

    const outputContent = `// Auto-generated file. Do not edit directly.
// Run "bun scripts/build-sdk.ts" or "bun run build:dev" from the package folder to regenerate.

// Full preload for trusted webviews (RPC, encryption, drag regions, webview tags)
export const preloadScript = ${JSON.stringify(fullPreloadJs)};

// Minimal preload for sandboxed/untrusted webviews (lifecycle events only, no RPC)
export const preloadScriptSandboxed = ${JSON.stringify(sandboxedPreloadJs)};
`;

    writeFileSync(outputPath, outputContent);
    mkdirSync(distDir, { recursive: true });
    writeFileSync(join(distDir, "preload-full.js"), fullPreloadJs);
    writeFileSync(join(distDir, "preload-sandboxed.js"), sandboxedPreloadJs);
    console.log("Preload scripts compiled successfully (full + sandboxed)");
}

async function generateTemplateEmbeddings() {
    const TEMPLATES_DIR = join(process.cwd(), "..", "..", "templates");
    const OUTPUT_FILE = join(process.cwd(), "src", "templates", "embedded.ts");

    const electrobunPackageJson = JSON.parse(
        readFileSync(join(process.cwd(), "package.json"), "utf-8"),
    );
    const electrobunVersion = electrobunPackageJson.version;

    if (!existsSync(TEMPLATES_DIR)) {
        console.log("No templates directory found, skipping template generation");
        return;
    }

    const templates: Record<
        string,
        { name: string; files: Record<string, string> }
    > = {};

    // Read all template directories
    const templateNames = readdirSync(TEMPLATES_DIR, { withFileTypes: true })
        .filter((dirent) => dirent.isDirectory())
        .map((dirent) => dirent.name);

    if (templateNames.length === 0) {
        console.log("No templates found in templates/ directory");
        return;
    }

    for (const templateName of templateNames) {
        const templateDir = join(TEMPLATES_DIR, templateName);
        const files: Record<string, string> = {};

        // Recursively read all files in the template directory
        function readDirectory(dir: string, basePath: string = "") {
            const entries = readdirSync(dir, { withFileTypes: true });

            for (const entry of entries) {
                const fullPath = join(dir, entry.name);
                const relativePath = join(basePath, entry.name).replace(/\\/g, "/");

                // Skip common directories and files that shouldn't be in templates
                if (
                    entry.name === "node_modules" ||
                    entry.name === ".git" ||
                    entry.name === "build" ||
                    entry.name === "dist" ||
                    entry.name === ".next" ||
                    entry.name === ".DS_Store" ||
                    (entry.name.startsWith(".") && entry.name !== ".gitignore") ||
                    entry.name === "package-lock.json" ||
                    entry.name === "bun.lock" ||
                    entry.name === "bun.lockb" ||
                    entry.name === "yarn.lock"
                ) {
                    continue;
                }

                if (entry.isDirectory()) {
                    readDirectory(fullPath, relativePath);
                } else {
                    try {
                        // 先读为 Buffer，通过内容判断二进制/文本（比扩展名白名单更健壮）
                        const raw = readFileSync(fullPath);
                        const isBinary = raw.includes(0); // 含 null 字节则为二进制
                        if (isBinary) {
                            files[relativePath] = "base64:" + raw.toString("base64");
                        } else {
                            files[relativePath] = raw.toString("utf-8");
                        }
                    } catch (error) {
                        console.warn(`Warning: Could not read ${fullPath}:`, error);
                    }
                }
            }
        }

        readDirectory(templateDir);

        // Pin the electrobun dependency version in template package.json
        // 支持 @pori15/electrobun-rust 和旧的 electrobun 两种包名
        if (files["package.json"]) {
            const pkgJson = JSON.parse(files["package.json"]);
            const depNames = ["@pori15/electrobun-rust", "electrobun"];
            for (const depName of depNames) {
                const electrobunDep = pkgJson.dependencies?.[depName];
                if (
                    typeof electrobunDep === "string" &&
                    (electrobunDep === "latest" || electrobunDep.startsWith("file:"))
                ) {
                    pkgJson.dependencies[depName] = electrobunVersion;
                }
            }
            files["package.json"] = JSON.stringify(pkgJson, null, "\t") + "\n";
        }

        templates[templateName] = {
            name: templateName,
            files,
        };
    }

    // Generate TypeScript file using JSON.stringify for proper escaping
    const output = `// Auto-generated file. Do not edit directly.
// Generated from templates/ directory

export interface Template {
  name: string;
  files: Record<string, string>;
}

export const templates: Record<string, Template> = ${JSON.stringify(templates, null, 2)};

export function getTemplateNames(): string[] {
  return Object.keys(templates);
}

export function getTemplate(name: string): Template | undefined {
  return templates[name];
}
`;

    // Ensure the output directory exists
    const outputDir = dirname(OUTPUT_FILE);
    if (!existsSync(outputDir)) {
        mkdirSync(outputDir, { recursive: true });
    }

    // Write the output file
    writeFileSync(OUTPUT_FILE, output);

    const totalFiles = Object.values(templates).reduce(
        (acc, t) => acc + Object.keys(t.files).length,
        0,
    );
    console.log(
        `Generated ${totalFiles} template files for ${templateNames.length} templates: ${templateNames.join(", ")}`,
    );
}
