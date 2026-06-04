import { join, dirname } from "path";
import { existsSync } from "fs";
import { ARCH, OS } from "../core/shared/platform";

/**
 * Walks up from projectRoot to find electrobun in node_modules (supports hoisted monorepo layouts)
 */
export function resolveElectrobunDir(): string {
  const projectRoot = process.cwd();
  let dir = projectRoot;
  while (dir !== dirname(dir)) {
    const candidate = join(dir, "node_modules", "@pori15", "electrobun-rust");
    if (existsSync(join(candidate, "package.json"))) {
      return candidate;
    }
    dir = dirname(dir);
  }
  return join(projectRoot, "node_modules", "@pori15", "electrobun-rust");
}

export const ELECTROBUN_DEP_PATH = resolveElectrobunDir();
export const ELECTROBUN_CACHE_PATH = join(dirname(ELECTROBUN_DEP_PATH), ".electrobun-cache");

/**
 * Get platform-specific paths
 */
export function getPlatformPaths(
  targetOS: "macos" | "win" | "linux",
  targetArch: "arm64" | "x64",
) {
  const binExt = targetOS === "win" ? ".exe" : "";
  const platformDistDir = join(
    ELECTROBUN_DEP_PATH,
    `dist-${targetOS}-${targetArch}`,
  );
  const sharedDistDir = join(ELECTROBUN_DEP_PATH, "dist");

  return {
    BUN_BINARY: join(platformDistDir, "bun") + binExt,
    LAUNCHER_DEV: join(platformDistDir, "electrobun") + binExt,
    LAUNCHER_RELEASE: join(platformDistDir, "launcher") + binExt,
    CORE_MACOS: join(platformDistDir, "libelectrobun_core.dylib"),
    CORE_WIN: join(platformDistDir, "electrobun_core.dll"),
    CORE_LINUX: join(platformDistDir, "libelectrobun_core.so"),
    WEBVIEW2LOADER_WIN: join(platformDistDir, "WebView2Loader.dll"),
    BSPATCH: join(platformDistDir, "bspatch") + binExt,
    EXTRACTOR: join(platformDistDir, "extractor") + binExt,
    BSDIFF: join(platformDistDir, "bsdiff") + binExt,
    ZSTD: join(platformDistDir, "rust-zstd") + binExt,
    MAIN_JS: join(sharedDistDir, "main.js"),
    API_DIR: join(sharedDistDir, "api"),
    PRELOAD_FULL_JS: join(platformDistDir, "preload-full.js"),
    PRELOAD_SANDBOXED_JS: join(platformDistDir, "preload-sandboxed.js"),
  };
}

// Default PATHS for host platform (backward compatibility)
export const _PATHS = getPlatformPaths(OS, ARCH);
