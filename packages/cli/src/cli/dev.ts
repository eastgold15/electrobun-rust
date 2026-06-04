import { existsSync, watch } from "fs";
import { join, relative } from "path";
import { ARCH, OS } from "../core/shared/platform";
import { getAppFileName, getMacOSBundleDisplayName } from "../core/shared/naming";
import type { DefaultConfig } from "./config";

/**
 * Launch the already-built dev bundle
 */
export async function runApp(
  config: DefaultConfig,
  options?: { onExit?: () => void },
): Promise<{ kill: () => void; exited: Promise<number> }> {
  const projectRoot = process.cwd();
  const buildEnvironment = "dev";
  const appFileName = getAppFileName(config.app.name, buildEnvironment);
  const macOSBundleDisplayName = getMacOSBundleDisplayName(config.app.name, buildEnvironment);
  const buildSubFolder = `${buildEnvironment}-${OS}-${ARCH}`;
  const buildFolder = join(projectRoot, config.build.buildFolder, buildSubFolder);
  const bundleFileName = OS === "macos" ? `${macOSBundleDisplayName}.app` : appFileName;

  let mainProc: any;
  // macOS: launcher 在 Contents/MacOS/launcher
  // Windows/Linux: launcher 在 bin/launcher(.exe)
  const bundleExecPath = OS === "macos"
    ? join(buildFolder, bundleFileName, "Contents", "MacOS")
    : join(buildFolder, bundleFileName, "bin");

  // Linux 使用 wry/WebKitGTK，无需 libNativeWrapper（CEF 已废弃）
  const launcherName = `${appFileName}${OS === "win" ? ".exe" : ""}`;
  mainProc = Bun.spawn([join(bundleExecPath, launcherName)], {
    stdio: ["inherit", "inherit", "inherit"],
    cwd: bundleExecPath,
  });

  if (!mainProc) throw new Error("Failed to spawn app process");

  const exitedPromise = mainProc.exited.then((code: number) => {
    options?.onExit?.();
    return code ?? 0;
  });

  return {
    kill: () => { try { mainProc.kill(); } catch {} },
    exited: exitedPromise,
  };
}

/**
 * Run the built app with signal handling (Ctrl+C for graceful shutdown)
 */
export async function runAppWithSignalHandling(config: DefaultConfig) {
  const handle = await runApp(config);
  let sigintCount = 0;

  process.on("SIGINT", () => {
    sigintCount++;
    if (sigintCount === 1) {
      console.log("\n[electrobun dev] Shutting down gracefully... (press Ctrl+C again to force quit)");
    } else {
      console.log("\n[electrobun dev] Force quitting...");
      process.exit(0);
    }
  });

  const code = await handle.exited;
  process.exit(code);
}

/**
 * Run the app in dev watch mode
 */
export async function runDevWatch(config: DefaultConfig) {
  const projectRoot = process.cwd();

  function shouldIgnore(fullPath: string): boolean {
    const ignorePatterns = [
      "node_modules", ".git", "target", "build", "dist",
      ...(config.build.watchIgnore || []),
    ];
    const relPath = relative(projectRoot, fullPath);
    return ignorePatterns.some((p) => relPath.includes(p));
  }

  let rebuildTimeout: ReturnType<typeof setTimeout> | null = null;
  const debounceMs = 300;

  async function triggerRebuild() {
    if (rebuildTimeout) clearTimeout(rebuildTimeout);
    rebuildTimeout = setTimeout(async () => {
      console.log("🔄 Rebuilding...");
      try {
        const { runBuild } = await import("./build");
        await runBuild(config, "dev");
        console.log("✅ Rebuild complete. Restarting app...");
      } catch (error) {
        console.error("❌ Rebuild failed:", error);
      }
    }, debounceMs);
  }

  function startWatchers() {
    const watchDirs = [
      join(projectRoot, "src"),
      ...(config.build.watch || []).map((d: string) => join(projectRoot, d)),
    ];
    for (const dir of watchDirs) {
      if (existsSync(dir)) {
        watch(dir, { recursive: true }, (eventType, filename) => {
          if (filename && !shouldIgnore(join(dir, filename.toString()))) {
            triggerRebuild();
          }
        });
        console.log(`Watching: ${dir}`);
      }
    }
  }

  function cleanup() { /* watcher cleanup */ }

  process.on("SIGINT", () => { cleanup(); process.exit(0); });
  process.on("SIGTERM", () => { cleanup(); process.exit(0); });

  await triggerRebuild();
  startWatchers();
  console.log("👀 Watching for changes...");
}
