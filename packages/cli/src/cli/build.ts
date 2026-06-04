import {
    existsSync,
    mkdirSync,
    rmSync,
    cpSync,
    readdirSync,
    readFileSync,
    writeFileSync,
    renameSync,
    unlinkSync,
    statSync,
} from "fs";
import { basename, dirname, join } from "path";
import { execSync, execFileSync } from "child_process";
import { ARCH, OS } from "../core/shared/platform";
import { ELECTROBUN_DEP_PATH, getPlatformPaths } from "./platform-paths";
import { safeCopyFile, createTar } from "./utils/file";
import { escapePathForTerminal } from "./utils/path";
import {
    generateUsageDescriptions,
    generateURLTypes,
    generateDocumentTypes,
} from "./utils/plist";
import type { DefaultConfig } from "./config";
import { ensureCoreDependencies } from "./downloads/core";
import { ensureBunBinary } from "./downloads/bun";
import { ensureWGPUDependencies, getEffectiveWGPUDir } from "./downloads/wgpu";
import {
    getAppFileName,
    getBundleFileName,
    getDmgVolumeName,
    getMacOSBundleDisplayName,
    getPlatformPrefix,
    getTarballFileName,
    getWindowsSetupFileName
} from "../core/shared/naming";

// ── 调试模式 ──────────────────────────────────────────────────
const VERBOSE = process.argv.includes("--verbose");
function debugLog(msg: string, ...args: any[]) {
  if (VERBOSE) console.log(`[DEBUG] ${msg}`, ...args);
}

export async function runBuild(
    config: DefaultConfig,
    buildEnvironment: "dev" | "canary" | "stable",
) {
    const currentTarget = { os: OS, arch: ARCH };
    const targetOS = currentTarget.os;
    const targetARCH = currentTarget.arch;
    const targetBinExt = targetOS === "win" ? ".exe" : "";
    const appFileName = getAppFileName(config.app.name, buildEnvironment);
    const macOSBundleDisplayName = getMacOSBundleDisplayName(
        config.app.name, buildEnvironment,
    );
    const platformPrefix = getPlatformPrefix(buildEnvironment, currentTarget.os, currentTarget.arch);
    const projectRoot = process.cwd();
    const buildFolder = join(projectRoot, config.build.buildFolder, platformPrefix);
    const artifactFolder = join(projectRoot, config.build.artifactFolder);
    debugLog("目标平台:", targetOS, "架构:", targetARCH);
    debugLog("构建环境:", buildEnvironment);
    debugLog("项目目录:", projectRoot);
    debugLog("构建输出:", buildFolder);

    debugLog("检查核心依赖...");
    await ensureCoreDependencies(currentTarget.os, currentTarget.arch);
    const targetPaths = getPlatformPaths(currentTarget.os, currentTarget.arch);
    debugLog("核心依赖路径:", {
      LAUNCHER: targetPaths.LAUNCHER_RELEASE,
      BUN: targetPaths.BUN_BINARY,
      CORE: targetPaths[targetOS === "win" ? "CORE_WIN" : targetOS === "macos" ? "CORE_MACOS" : "CORE_LINUX"],
    });

    const runHook = (hookName: string, extraEnv: Record<string, string> = {}) => {
        const hookScript = (config.scripts as Record<string, string>)[hookName];
        if (!hookScript) return;
        console.log(`Running ${hookName} script:`, hookScript);
        const hostPaths = getPlatformPaths(OS, ARCH);
        const result = Bun.spawnSync([hostPaths.BUN_BINARY, hookScript], {
            stdio: ["ignore", "inherit", "inherit"],
            cwd: projectRoot,
            env: {
                ...process.env,
                ELECTROBUN_BUILD_ENV: buildEnvironment,
                ELECTROBUN_OS: targetOS,
                ELECTROBUN_ARCH: targetARCH,
                ELECTROBUN_BUILD_DIR: buildFolder,
                ELECTROBUN_APP_NAME: appFileName,
                ELECTROBUN_APP_VERSION: config.app.version,
                ELECTROBUN_APP_IDENTIFIER: config.app.identifier,
                ELECTROBUN_ARTIFACT_DIR: artifactFolder,
                ...extraEnv,
            },
        });
        if (result.exitCode !== 0) {
            console.error(`${hookName} script failed with exit code:`, result.exitCode);
            if (result.stderr) {
                console.error("stderr:", new TextDecoder().decode(result.stderr as Uint8Array));
            }
            console.error("Tried to run with bun at:", hostPaths.BUN_BINARY);
            console.error("Script path:", hookScript);
            console.error("Working directory:", projectRoot);
            throw new Error("Build failed: hook script failed");
        }
    };

    const buildIcons = (appBundleFolderResourcesPath: string, appBundleFolderPath: string) => {
        if (targetOS === "macos" && config.build.mac?.icons) {
            const iconSourceFolder = join(projectRoot, config.build.mac.icons);
            const iconDestPath = join(appBundleFolderResourcesPath, "AppIcon.icns");
            if (existsSync(iconSourceFolder)) {
                if (OS === "macos") {
                    if (config.build.mac.icons.endsWith(".icon")) {
                        const actoolCheck = Bun.spawnSync(["xcrun", "--find", "actool"], { stdio: ["ignore", "pipe", "pipe"] });
                        if (actoolCheck.exitCode !== 0) {
                            throw new Error("Building .icon files requires Xcode...");
                        }
                        const iconStem = basename(config.build.mac.icons, ".icon");
                        const partialPlistPath = join(buildFolder, ".actool-partial-info.plist");
                        console.log("Compiling .icon file with actool (requires Xcode)...");
                        const result = Bun.spawnSync(["xcrun", "actool", "--compile", appBundleFolderResourcesPath, "--app-icon", iconStem, "--platform", "macosx", "--minimum-deployment-target", "11.0", "--output-partial-info-plist", partialPlistPath, iconSourceFolder], {
                            cwd: projectRoot, stdio: ["ignore", "inherit", "inherit"],
                            env: { ...process.env, ELECTROBUN_BUILD_ENV: buildEnvironment },
                        });
                        if (result.exitCode !== 0) throw new Error(`actool failed`);
                        const actoolIcns = join(appBundleFolderResourcesPath, `${iconStem}.icns`);
                        if (existsSync(actoolIcns) && actoolIcns !== iconDestPath) renameSync(actoolIcns, iconDestPath);
                    } else {
                        const result = Bun.spawnSync(["iconutil", "-c", "icns", "-o", iconDestPath, iconSourceFolder], {
                            cwd: appBundleFolderResourcesPath, stdio: ["ignore", "inherit", "inherit"],
                        });
                        if (result.exitCode !== 0) throw new Error(`iconutil failed`);
                    }
                } else {
                    console.log(`WARNING: Cannot build macOS icons on ${OS}`);
                }
            }
        } else if (targetOS === "linux" && config.build.linux?.icon) {
            const iconSourcePath = join(projectRoot, config.build.linux.icon);
            if (existsSync(iconSourcePath)) {
                mkdirSync(appBundleFolderResourcesPath, { recursive: true });
                cpSync(iconSourcePath, join(appBundleFolderResourcesPath, "appIcon.png"), { dereference: true });
                const extractorIconDir = join(appBundleFolderResourcesPath, "app");
                mkdirSync(extractorIconDir, { recursive: true });
                cpSync(iconSourcePath, join(extractorIconDir, "icon.png"), { dereference: true });
            }
            const desktopContent = `[Desktop Entry]\nVersion=1.0\nType=Application\nName=${config.app.name}\nComment=${config.app.description || `${config.app.name} application`}\nExec=launcher\nIcon=appIcon.png\nTerminal=false\nStartupWMClass=${config.app.name}\nCategories=Utility;Application;\n`;
            writeFileSync(join(appBundleFolderPath, `${config.app.name}.desktop`), desktopContent);
        } else if (targetOS === "win" && config.build.win?.icon) {
            const iconPath = join(projectRoot, config.build.win.icon);
            if (existsSync(iconPath)) cpSync(iconPath, join(appBundleFolderResourcesPath, "app.ico"), { dereference: true });
        }

        if (targetOS === "macos" && config.app.fileAssociations) {
            for (const assoc of config.app.fileAssociations) {
                if (assoc.icon) {
                    const iconSourcePath = join(projectRoot, assoc.icon);
                    if (existsSync(iconSourcePath)) {
                        cpSync(iconSourcePath, join(appBundleFolderResourcesPath, basename(iconSourcePath)), { dereference: true });
                    }
                }
            }
        }
    };

    runHook("preBuild");

    if (existsSync(buildFolder)) {
      debugLog("清理已有构建目录:", buildFolder);
      rmSync(buildFolder, { recursive: true, force: true });
    }
    mkdirSync(buildFolder, { recursive: true });
    debugLog("构建目录已创建:", buildFolder);

    const bunConfig = config.build.bun;
    const bunSource = join(projectRoot, bunConfig.entrypoint);
    if (!existsSync(bunSource)) {
        throw new Error(`Failed to bundle ${bunSource} because it doesn't exist.`);
    }
    let appBundleFolderPath: string;
    let appBundleFolderContentsPath: string;
    let appBundleMacOSPath: string;
    let appBundleFolderResourcesPath: string;
    let appBundleFolderFrameworksPath: string;
    let appBundleAppCodePath: string;
    const bundleName = targetOS === "macos" ? macOSBundleDisplayName : appFileName;

    const bundle = createAppBundle(bundleName, buildFolder, targetOS);
    appBundleFolderPath = bundle.appBundleFolderPath;
    appBundleFolderContentsPath = bundle.appBundleFolderContentsPath;
    appBundleMacOSPath = bundle.appBundleMacOSPath;
    appBundleFolderResourcesPath = bundle.appBundleFolderResourcesPath;
    appBundleFolderFrameworksPath = bundle.appBundleFolderFrameworksPath;
    appBundleAppCodePath = join(appBundleFolderResourcesPath, "app");
    mkdirSync(appBundleAppCodePath, { recursive: true });
        const usageDescriptions = generateUsageDescriptions(config.build.mac.entitlements || {});
        const urlTypes = generateURLTypes(config.app.urlSchemes, config.app.identifier);
        const documentTypes = generateDocumentTypes(config.app.fileAssociations, projectRoot, config.app.identifier);
        const iconName = config.build.mac?.icons?.endsWith(".icon") ? basename(config.build.mac.icons, ".icon") : null;

        const InfoPlistContents = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>launcher</string>
    <key>CFBundleIdentifier</key>
    <string>${config.app.identifier}</string>
    <key>CFBundleName</key>
    <string>${bundleName}</string>
    <key>CFBundleVersion</key>
    <string>${config.app.version}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>${iconName ? `\n    <key>CFBundleIconName</key>\n    <string>${iconName}</string>` : ""}${usageDescriptions ? "\n" + usageDescriptions : ""}${urlTypes ? "\n" + urlTypes : ""}${documentTypes ? "\n" + documentTypes : ""}
</dict>
</plist>`;

        await Bun.write(join(appBundleFolderContentsPath, "Info.plist"), InfoPlistContents);

        const launcherExecName = appFileName;
        safeCopyFile(targetPaths.LAUNCHER_RELEASE, join(appBundleMacOSPath, launcherExecName) + targetBinExt);

        // Embed icon into launcher (Windows)
        await embedWindowsIcon(targetOS, config, projectRoot, buildFolder, join(appBundleMacOSPath, launcherExecName) + ".exe", "launcher");

        safeCopyFile(targetPaths.PRELOAD_FULL_JS, join(appBundleFolderResourcesPath, "preload-full.js"));
        safeCopyFile(targetPaths.PRELOAD_SANDBOXED_JS, join(appBundleFolderResourcesPath, "preload-sandboxed.js"));
        safeCopyFile(targetPaths.MAIN_JS, join(appBundleFolderResourcesPath, "main.js"));

        const bunBinarySourcePath = await ensureBunBinary(currentTarget.os, currentTarget.arch, config.build.bunVersion, config.build.bunnyBun);
        const bunBinaryDestInBundlePath = join(appBundleMacOSPath, "bun") + targetBinExt;
        const destFolder2 = dirname(bunBinaryDestInBundlePath);
        if (!existsSync(destFolder2)) mkdirSync(destFolder2, { recursive: true });
        safeCopyFile(bunBinarySourcePath, bunBinaryDestInBundlePath);

        const bunDir = dirname(bunBinarySourcePath);
        const icuDataFileName = readdirSync(bunDir).find((f) => /^icudt\d+l\.dat$/.test(f));
        const icuDataSource = icuDataFileName ? join(bunDir, icuDataFileName) : "";
        if (icuDataFileName && existsSync(icuDataSource) && targetOS !== "macos") {
            const icuDataDest = join(appBundleMacOSPath, icuDataFileName);
            const locales = config.build?.locales;
            if (locales && locales !== "*" && Array.isArray(locales) && locales.length > 0) {
                try {
                    await trimICUData(icuDataSource, icuDataDest, locales);
                } catch (error) {
                    console.warn(`Warning: Failed to trim ICU data: ${error}`);
                    safeCopyFile(icuDataSource, icuDataDest);
                }
            } else {
                safeCopyFile(icuDataSource, icuDataDest);
            }
        }

        // Embed icon into bun binary (Windows)
        await embedWindowsIcon(targetOS, config, projectRoot, buildFolder, bunBinaryDestInBundlePath, "bun");

        // Copy core library
        const coreLibName = targetOS === "win" ? "electrobun_core.dll" : targetOS === "macos" ? "libelectrobun_core.dylib" : "libelectrobun_core.so";
        safeCopyFile(join(dirname(targetPaths.LAUNCHER_RELEASE), coreLibName), join(appBundleMacOSPath, coreLibName));

        // Copy WGPU library
        const wgpuDir = config.build.wgpuVersion
            ? await ensureWGPUDependencies(currentTarget.os, currentTarget.arch, config.build.wgpuVersion)
            : undefined;
        if (wgpuDir && existsSync(wgpuDir)) {
            for (const file of readdirSync(wgpuDir)) {
                safeCopyFile(join(wgpuDir, file), join(appBundleMacOSPath, file));
            }
        }

        // ── 编译用户 Bun 入口 (src/bun/index.ts → app/bun/index.js) ──
        const bunDestFolder = join(appBundleAppCodePath, "bun");
        const { entrypoint: _bunEntrypoint, ...bunBuildOptions } = bunConfig;
        const bunBuildResult = await Bun.build({
            ...bunBuildOptions,
            entrypoints: [bunSource],
            outdir: bunDestFolder,
            target: "bun",
        });

        if (!bunBuildResult.success) {
            console.error("Failed to build", bunSource);
            printBuildLogs(bunBuildResult.logs);
            throw new Error("Build failed: bun build failed");
        }

        // ── 编译视图代码 ──
        if (config.build.views) {
            for (const [viewName, viewConfig] of Object.entries(config.build.views)) {
                const viewEntry = (viewConfig as any).entrypoint;
                if (viewEntry) {
                    const viewResult = await Bun.build({
                        entrypoints: [join(projectRoot, viewEntry)],
                        outdir: join(appBundleAppCodePath, "views", viewName),
                        target: "browser",
                    });
                    if (!viewResult.success) {
                        console.error(`Failed to build view "${viewName}":`, viewResult.logs);
                    }
                }
            }
        }

        // ── 复制静态资源 (HTML/CSS/图片等) ──
        if (config.build.copy) {
            for (const relSource in config.build.copy) {
                const source = join(projectRoot, relSource);
                if (!existsSync(source)) {
                    console.error(`Failed to copy ${source} because it doesn't exist.`);
                    continue;
                }
                const destination = join(appBundleAppCodePath, config.build.copy[relSource]!);
                const destFolder = dirname(destination);
                if (!existsSync(destFolder)) mkdirSync(destFolder, { recursive: true });
                cpSync(source, destination, { recursive: true, dereference: true });
            }
        }

        buildIcons(appBundleFolderResourcesPath, appBundleFolderPath);

        // Set permissions on macOS
        if (targetOS === "macos") {
            execSync(`chmod +x ${escapePathForTerminal(join(appBundleMacOSPath, launcherExecName))}`);
            execSync(`chmod +x ${escapePathForTerminal(join(appBundleMacOSPath, "bun"))}`);
        }

        // Codesign
        if (config.build.mac.codesign) {
            const entitlementsFile = buildEntitlementsFile(buildFolder, config);
            codesignAppBundle(appBundleFolderPath, entitlementsFile, config, targetOS);
        }

        // Notarize
        if (config.build.mac.notarize && config.build.mac.codesign) {
            await notarizeAndStaple(appBundleFolderPath, targetOS);
        }

        // Create DMG
        if (targetOS === "macos" && config.build.mac.createDmg) {
            await createDMG(appBundleFolderPath, appFileName, macOSBundleDisplayName, buildFolder, artifactFolder, config, buildEnvironment, currentTarget);
        }

    debugLog("安装程序打包阶段: OS=", targetOS, " ENV=", buildEnvironment);
    debugLog("appBundleFolderPath 是否存在:", existsSync(appBundleFolderPath));
    // 仅稳定版构建才打包安装程序
    if (buildEnvironment !== "dev") {
        if (targetOS === "linux") {
            await createLinuxInstallerArchive(buildFolder, "", appFileName, config, buildEnvironment, "", targetPaths);
        } else if (targetOS === "win") {
            await createWindowsSelfExtractingExe(buildFolder, appFileName, config, buildEnvironment);
            await wrapWindowsInstallerInZip(buildFolder, appFileName, artifactFolder);
        }
    }

    runHook("postBuild");
}

// ── Helper: embed Windows icon into PE binary ──────────────────
async function embedWindowsIcon(
    targetOS: string,
    config: DefaultConfig,
    projectRoot: string,
    buildFolder: string,
    targetBinaryPath: string,
    label: string,
) {
    if (targetOS !== "win" || !config.build.win?.icon) return;
    const iconSourcePath = config.build.win.icon.startsWith("/") || config.build.win.icon.match(/^[a-zA-Z]:/)
        ? config.build.win.icon : join(projectRoot, config.build.win.icon);
    if (!existsSync(iconSourcePath)) return;
    try {
        let iconPath = iconSourcePath;
        if (iconSourcePath.toLowerCase().endsWith(".png")) {
            const pngToIco = (await import("png-to-ico")).default;
            const tempIcoPath = join(buildFolder, `temp-${label}-icon.ico`);
            const icoBuffer = await pngToIco(iconSourcePath);
            writeFileSync(tempIcoPath, new Uint8Array(icoBuffer));
            iconPath = tempIcoPath;
        }
        const rceditPkgPath = require.resolve("rcedit/package.json");
        const rceditDir = dirname(rceditPkgPath);
        const rceditX64 = join(rceditDir, "bin", "rcedit-x64.exe");
        const rceditExe = existsSync(rceditX64) ? rceditX64 : join(rceditDir, "bin", "rcedit.exe");
        execFileSync(rceditExe, [targetBinaryPath, "--set-icon", iconPath]);
        if (iconPath !== iconSourcePath && existsSync(iconPath)) unlinkSync(iconPath);
    } catch (error) {
        console.warn(`Warning: Failed to embed icon into ${label}: ${error}`);
    }
}

// ── Helper: trim ICU data ─────────────────────────────────────
async function trimICUData(source: string, dest: string, locales: string[]): Promise<void> {
    safeCopyFile(source, dest);
    let icupkgPath = "icupkg";
    try {
        execSync(`${icupkgPath} --help`, { stdio: "ignore" });
    } catch {
        throw new Error("icupkg not found in PATH. Install ICU tools to enable locale trimming.");
    }
    const listOutput = execSync(`${icupkgPath} -l "${dest}"`, { encoding: "utf-8" });
    const allItems = listOutput.split("\n").filter((line) => line.trim());
    const localeDirs = ["brkitr/", "coll/", "curr/", "lang/", "locales/", "rbnf/", "region/", "unit/", "zone/"];
    const toRemove = allItems.filter((item) => {
        const isLocaleItem = localeDirs.some((dir) => item.startsWith(dir));
        if (!isLocaleItem) return false;
        const name = (item.split("/").pop() || "").replace(/\.res$/, "");
        return !locales.some((l) => name === l || name === "root" || name.startsWith(`${l}_`) || name.startsWith(`${l}-`));
    });
    if (toRemove.length > 0) {
        const { tmpdir } = await import("os");
        const removeListPath = join(tmpdir(), "icu-remove.txt");
        writeFileSync(removeListPath, toRemove.join("\n"));
        try { execSync(`${icupkgPath} -r "@${removeListPath}" "${dest}"`, { stdio: "inherit" }); }
        finally { try { unlinkSync(removeListPath); } catch { } }
    }
}

// ── Helper: create DMG ────────────────────────────────────────
async function createDMG(
    appBundleFolderPath: string, appFileName: string, macOSBundleDisplayName: string,
    buildFolder: string, artifactFolder: string, config: any, buildEnvironment: string,
    currentTarget: { os: string; arch: string },
) {
    const dmgVolumeName = getDmgVolumeName(config.app.name, buildEnvironment);
    const dmgName = `${appFileName}.dmg`;
    const stagingDir = join(buildFolder, "dmg-staging");
    if (existsSync(stagingDir)) rmSync(stagingDir, { recursive: true, force: true });
    mkdirSync(stagingDir, { recursive: true });

    const appInStaging = join(stagingDir, `${macOSBundleDisplayName}.app`);
    cpSync(appBundleFolderPath, appInStaging, { recursive: true, dereference: true });

    execSync(`ln -sf /Applications "${join(stagingDir, "Applications")}"`);

    const dmgPath = join(buildFolder, dmgName);
    const createDmgArgs = [
        "create-dmg",
        "--volname", dmgVolumeName,
        "--window-pos", "200", "120",
        "--window-size", "600", "450",
        "--icon-size", "100",
        "--icon", `${macOSBundleDisplayName}.app`, "150", "200",
        "--hide-extension", `${macOSBundleDisplayName}.app`,
        "--app-drop-link", "450", "200",
        dmgPath,
        stagingDir,
    ];
    const result = Bun.spawnSync(createDmgArgs, { stdio: ["pipe", "pipe", "pipe"] });
    if (result.exitCode !== 0) throw new Error(`create-dmg failed with exit code ${result.exitCode}`);

    if (config.build.mac.codesign) {
        const entitlementsFile = buildEntitlementsFile(buildFolder, config);
        codesignAppBundle(dmgPath, entitlementsFile, config, currentTarget.os);
    }
    if (config.build.mac.notarize && config.build.mac.codesign) {
        await notarizeAndStaple(dmgPath, currentTarget.os);
    }

    mkdirSync(artifactFolder, { recursive: true });
    const artifactDmgPath = join(artifactFolder, dmgName);
    if (existsSync(artifactDmgPath)) unlinkSync(artifactDmgPath);
    renameSync(dmgPath, artifactDmgPath);
    rmSync(stagingDir, { recursive: true, force: true });
    console.log(`✓ DMG created: ${artifactDmgPath}`);
}

// ── Helper: build entitlements file ───────────────────────────
function buildEntitlementsFile(buildFolder: string, config: any): string {
    const entitlements: Record<string, boolean | string | string[]> = {};
    for (const [key, value] of Object.entries(config.build.mac.entitlements || {})) {
        getEntitlementValue(value as boolean | string | string[]);
    }
    const entitlementsPath = join(buildFolder, "entitlements.plist");
    const plistContent = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
${Object.entries(entitlements)
            .map(([key, value]) => {
                if (typeof value === "boolean") return `    <key>${key}</key>\n    <${value}/>`;
                if (Array.isArray(value)) return `    <key>${key}</key>\n    <array>\n${value.map(v => `      <string>${v}</string>`).join("\n")}\n    </array>`;
                return `    <key>${key}</key>\n    <string>${value}</string>`;
            })
            .join("\n")}
</dict>
</plist>`;
    writeFileSync(entitlementsPath, plistContent);
    return entitlementsPath;
}

function getEntitlementValue(value: boolean | string | string[]) {
    if (typeof value === "string" && (value.toLowerCase() === "true" || value.toLowerCase() === "false")) {
        return value.toLowerCase() === "true";
    }
    return value;
}

// ── Codesign ───────────────────────────────────────────────────
function codesignAppBundle(path: string, entitlementsFilePath: string | undefined, config: any, targetOS: string) {
    if (targetOS !== "macos" || !config.build.mac.codesign) return;
    const developerId = process.env["ELECTROBUN_DEVELOPER_ID"];
    if (!developerId) { console.error("Env var ELECTROBUN_DEVELOPER_ID is required"); process.exit(1); }

    const isBundle = path.endsWith(".app");
    if (isBundle) {
        const helperNames = ["bun Helper", "bun Helper (Alerts)", "bun Helper (GPU)", "bun Helper (Plugin)", "bun Helper (Renderer)"];
        for (const helper of helperNames) {
            const helperPath = join(path, "Contents", "Frameworks", `${helper}.app`);
            if (existsSync(helperPath)) {
                execSync(`codesign --force --sign "${developerId}" --timestamp --options runtime "${helperPath}"`, { stdio: "inherit" });
            }
        }
        execSync(`codesign --force --sign "${developerId}" --timestamp --options runtime "${path}"`, { stdio: "inherit" });
    } else {
        execSync(`codesign --force --sign "${developerId}" --timestamp --options runtime "${path}"`, { stdio: "inherit" });
    }
    console.log(`✓ Signed: ${path}`);
}

// ── Notarize ───────────────────────────────────────────────────
async function notarizeAndStaple(path: string, targetOS: string) {
    if (targetOS !== "macos") return;
    const appleId = process.env["APPLE_ID"];
    const appSpecificPassword = process.env["APPLE_APP_SPECIFIC_PASSWORD"];
    const teamId = process.env["APPLE_TEAM_ID"];
    if (!appleId || !appSpecificPassword || !teamId) {
        console.error("Missing Apple credentials for notarization"); return;
    }
    console.log(`Notarizing: ${path}`);
    execSync(`xcrun notarytool submit "${path}" --apple-id "${appleId}" --password "${appSpecificPassword}" --team-id "${teamId}" --wait`, { stdio: "inherit" });
    execSync(`xcrun stapler staple "${path}"`, { stdio: "inherit" });
    console.log(`✓ Notarized: ${path}`);
}

// ── Create app bundle structure ────────────────────────────────
function createAppBundle(bundleName: string, buildFolder: string, targetOS: string) {
    const appBundleFolderPath = join(buildFolder, `${bundleName}${targetOS === "macos" ? ".app" : ""}`);
    const isMac = targetOS === "macos";
    const appBundleFolderContentsPath = isMac ? join(appBundleFolderPath, "Contents") : appBundleFolderPath;
    // macOS: launcher 在 Contents/MacOS/，Resources 在 Contents/Resources/
    // Windows/Linux: 也用类似结构，launcher 在 bin/，Resources 在 Resources/
    const appBundleMacOSPath = isMac ? join(appBundleFolderContentsPath, "MacOS") : join(appBundleFolderPath, "bin");
    const appBundleFolderResourcesPath = isMac ? join(appBundleFolderContentsPath, "Resources") : join(appBundleFolderPath, "Resources");
    const appBundleFolderFrameworksPath = isMac ? join(appBundleFolderContentsPath, "Frameworks") : join(appBundleFolderPath, "lib");

    mkdirSync(appBundleMacOSPath, { recursive: true });
    mkdirSync(appBundleFolderResourcesPath, { recursive: true });
    mkdirSync(appBundleFolderFrameworksPath, { recursive: true });

    return { appBundleFolderPath, appBundleFolderContentsPath, appBundleMacOSPath, appBundleFolderResourcesPath, appBundleFolderFrameworksPath };
}

// ── Linux installer ────────────────────────────────────────────
async function createLinuxInstallerArchive(
    buildFolder: string, compressedTarPath: string, appFileName: string,
    config: any, buildEnvironment: string, hash: string,
    targetPaths: ReturnType<typeof getPlatformPaths>,
): Promise<string> {
    console.log("Creating Linux installer archive...");
    const installerName = `${appFileName}-Setup`;
    const stagingDir = join(buildFolder, `${installerName}-staging`);
    if (existsSync(stagingDir)) rmSync(stagingDir, { recursive: true, force: true });
    mkdirSync(stagingDir, { recursive: true });

    try {
        const extractorBinary = readFileSync(targetPaths.EXTRACTOR);
        const metadata = { identifier: config.app.identifier, name: config.app.name, channel: buildEnvironment, hash };
        const metadataBuffer = Buffer.from(JSON.stringify(metadata), "utf8");
        const metadataMarker = Buffer.from("ELECTROBUN_METADATA_V1", "utf8");
        const archiveMarker = Buffer.from("ELECTROBUN_ARCHIVE_V1", "utf8");
        const combinedBuffer = Buffer.concat([
            new Uint8Array(extractorBinary), new Uint8Array(metadataMarker),
            new Uint8Array(metadataBuffer), new Uint8Array(archiveMarker),
        ]);

        const installerPath = join(stagingDir, "installer");
        writeFileSync(installerPath, new Uint8Array(combinedBuffer), { mode: 0o755 });
        execSync(`chmod +x ${escapePathForTerminal(installerPath)}`);

        const archiveName = `${installerName}.tar.gz`;
        const archivePath = join(buildFolder, archiveName);
        createTarGz(archivePath, stagingDir, ["."]);

        if (!existsSync(archivePath)) throw new Error(`Installer archive not created: ${archivePath}`);
        const stats = statSync(archivePath);
        console.log(`✓ Linux installer archive created: ${archivePath} (${(stats.size / 1024 / 1024).toFixed(2)} MB)`);
        return archivePath;
    } finally {
        if (existsSync(stagingDir)) rmSync(stagingDir, { recursive: true, force: true });
    }
}

// ── Windows 安装程序 ───────────────────────────────────────────

/**
 * 创建 Windows 安装归档（tar.gz 格式，供更新系统使用）
 */
async function createWindowsSelfExtractingExe(
    buildFolder: string, appFileName: string,
    config: DefaultConfig, buildEnvironment: string,
): Promise<string> {
    console.log("Creating Windows installer archive...");
    const bundleName = `${appFileName}`;
    const bundlePath = join(buildFolder, bundleName);

    if (!existsSync(bundlePath)) {
        throw new Error(`Windows build bundle not found: ${bundlePath}`);
    }

    const installerName = `${appFileName}-Setup`;
    const archiveName = `${installerName}.tar.gz`;
    const archivePath = join(buildFolder, archiveName);

    debugLog("Windows installer bundle 路径:", bundlePath);
    debugLog("buildFolder 内容:", readdirSync(buildFolder));
    if (existsSync(bundlePath)) {
      debugLog("bundle 目录内容:", readdirSync(bundlePath));
    }

    createTarGz(archivePath, buildFolder, [bundleName]);

    if (!existsSync(archivePath)) {
        throw new Error(`Windows installer archive not created: ${archivePath}`);
    }
    const stats = statSync(archivePath);
    console.log(`✓ Windows installer archive created: ${archivePath} (${(stats.size / 1024 / 1024).toFixed(2)} MB)`);
    return archivePath;
}

/**
 * 将 Windows 安装归档包装为 .zip 格式（便于用户直接解压使用）
 */
async function wrapWindowsInstallerInZip(
    buildFolder: string, appFileName: string, artifactFolder: string,
): Promise<string> {
    console.log("Wrapping Windows installer in zip...");
    const bundleName = `${appFileName}`;
    const bundlePath = join(buildFolder, bundleName);

    if (!existsSync(bundlePath)) {
        throw new Error(`Windows build bundle not found: ${bundlePath}`);
    }

    const zipName = `${bundleName}.zip`;
    const zipPath = join(buildFolder, zipName);

    // 使用 7-Zip 创建 zip（兼容映射网络盘 L:）
    createZip(zipPath, bundlePath);

    if (!existsSync(zipPath)) {
        console.error("Warning: Windows zip installer not created");
        return "";
    }

    // Copy to artifacts
    mkdirSync(artifactFolder, { recursive: true });
    const artifactPath = join(artifactFolder, zipName);
    cpSync(zipPath, artifactPath, { dereference: true });
    try { unlinkSync(zipPath); } catch { /* ignore */ }

    const stats = statSync(artifactPath);
    console.log(`✓ Windows zip installer created: ${artifactPath} (${(stats.size / 1024 / 1024).toFixed(2)} MB)`);
    return artifactPath;
}

/**
 * Print build logs from Bun.build in a readable format
 */
function printBuildLogs(logs: any[] | undefined | null) {
    if (!logs || logs.length === 0) return;
    for (const log of logs) {
        if (typeof log === "string") {
            console.error(log);
        } else if (log && typeof log === "object") {
            console.error(log.message || JSON.stringify(log));
        }
    }
}

// ── tar.gz 打包（跨平台，Windows 用 7-Zip 避免映射盘问题） ────
const SEVEN_ZIP = "C:\\Program Files\\7-Zip\\7z.exe";

function createTarGz(archivePath: string, cwd: string, entries: string[]) {
  debugLog("createTarGz 开始: archivePath=", archivePath, "cwd=", cwd, "entries=", entries);
  debugLog("7z.exe 是否存在:", existsSync(SEVEN_ZIP));
  debugLog("cwd 是否存在:", existsSync(cwd));
  for (const e of entries) {
    debugLog("entry 路径:", join(cwd, e), " 是否存在:", existsSync(join(cwd, e)));
  }
  if (OS === "win") {
    // 7-Zip 两遍：先 tar、再 gzip
    // 用 cmd /c cd /d 切换到目录（比 execSync cwd 更可靠，兼容映射网络盘 L:）
    const tarPath = archivePath.replace(/\.gz$/, "");
    const entriesArg = entries.map((e) => `"${e}"`).join(" ");
    debugLog("7z tar 命令:", `cd /d "${cwd}" && "${SEVEN_ZIP}" a -ttar "${tarPath}" -bb0 -r ${entriesArg}`);
    execSync(
      `cd /d "${cwd}" && "${SEVEN_ZIP}" a -ttar "${tarPath}" -bb0 -r ${entriesArg}`,
      { stdio: "inherit" },
    );
    debugLog("中间 tar 文件:", tarPath, " 是否存在:", existsSync(tarPath));
    execSync(
      `cd /d "${cwd}" && "${SEVEN_ZIP}" a -tgzip "${archivePath}" -bb0 "${tarPath}"`,
      { stdio: "inherit" },
    );
    try { unlinkSync(tarPath); } catch { /* ignore */ }
  } else {
    execSync(`tar -czf "${archivePath}" ${entries.map((e) => `"${e}"`).join(" ")}`, {
      cwd, stdio: "inherit",
    });
  }
}

function createZip(zipPath: string, sourceDir: string) {
  debugLog("createZip: zipPath=", zipPath, "sourceDir=", sourceDir);
  debugLog("sourceDir 存在:", existsSync(sourceDir));
  debugLog("sourceDir 内容:", existsSync(sourceDir) ? readdirSync(sourceDir).slice(0, 10) : "N/A");
  // 7-Zip 直接打 zip（cd /d 确保映射盘正常）
  execSync(
    `cd /d "${sourceDir}" && "${SEVEN_ZIP}" a -tzip "${zipPath}" -bb0 -r ".\\*"`,
    { stdio: "inherit" },
  );
}

// ── Foreground takeover ────────────────────────────────────────
export async function takeoverForeground(): Promise<() => void> {
    let restoreFunctions: Array<() => void> = [];
    // Platform-specific foreground takeover logic
    return () => {
        for (const restore of restoreFunctions) { restore(); }
    };
}
