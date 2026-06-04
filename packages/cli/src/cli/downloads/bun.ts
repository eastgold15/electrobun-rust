import {
  existsSync,
  mkdirSync,
  renameSync,
  unlinkSync,
  writeFileSync,
  rmSync,
  readdirSync,
  readFileSync,
} from "fs";
import { join } from "path";
import { ELECTROBUN_DEP_PATH, ELECTROBUN_CACHE_PATH, getPlatformPaths } from "../platform-paths";
import { downloadFile, extractZip } from "../../../src/core/shared/download";

const GITHUB_CONFIG = {
  bunOwner: 'blackboardsh',
  bunRepo: 'bun',
};

/**
 * Ensures the correct Bun binary is available for bundling
 */
export async function ensureBunBinary(
  targetOS: "macos" | "win" | "linux",
  targetArch: "arm64" | "x64",
  bunVersion?: string,
  bunnyBun?: string,
): Promise<string> {
  const effectiveVersion = bunnyBun || bunVersion;
  if (!effectiveVersion) {
    return getPlatformPaths(targetOS, targetArch).BUN_BINARY;
  }

  const binExt = targetOS === "win" ? ".exe" : "";
  const cacheSubdir = bunnyBun ? "bunny-bun-override" : "bun-override";
  const overrideDir = join(ELECTROBUN_CACHE_PATH, cacheSubdir, `${targetOS}-${targetArch}`);
  const overrideBinary = join(overrideDir, `bun${binExt}`);
  const versionFile = join(overrideDir, ".bun-version");

  if (existsSync(overrideBinary) && existsSync(versionFile)) {
    const cachedVersion = readFileSync(versionFile, "utf8").trim();
    if (cachedVersion === effectiveVersion) {
      console.log(`${bunnyBun ? "Bunny" : "Custom"} Bun ${effectiveVersion} already cached for ${targetOS}-${targetArch}`);
      return overrideBinary;
    }
    console.log(`Cached Bun version "${cachedVersion}" does not match requested "${effectiveVersion}", re-downloading...`);
    rmSync(overrideDir, { recursive: true, force: true });
  } else if (existsSync(overrideDir)) {
    rmSync(overrideDir, { recursive: true, force: true });
  }

  if (bunnyBun) {
    await downloadBunnyBun(bunnyBun, targetOS, targetArch);
  } else {
    await downloadCustomBun(effectiveVersion, targetOS, targetArch);
  }
  return overrideBinary;
}

async function downloadCustomBun(
  bunVersion: string,
  platformOS: "macos" | "win" | "linux",
  platformArch: "arm64" | "x64",
) {
  const { bunUrlSegment, bunDirName } = getBunDownloadInfo(platformOS, platformArch);
  const overrideDir = join(ELECTROBUN_CACHE_PATH, "bun-override", `${platformOS}-${platformArch}`);
  const binExt = platformOS === "win" ? ".exe" : "";
  const overrideBinary = join(overrideDir, `bun${binExt}`);
  const bunUrl = `https://github.com/oven-sh/bun/releases/download/bun-v${bunVersion}/${bunUrlSegment}`;

  console.log(`Using custom Bun version: ${bunVersion}`);
  await downloadAndSetupBun(bunUrl, bunDirName, overrideDir, overrideBinary, binExt, bunVersion, platformOS, platformArch);
}

async function downloadBunnyBun(
  releaseTag: string,
  platformOS: "macos" | "win" | "linux",
  platformArch: "arm64" | "x64",
) {
  const { bunUrlSegment: assetName, bunDirName: dirName } = getBunDownloadInfo(platformOS, platformArch, true);
  const overrideDir = join(ELECTROBUN_CACHE_PATH, "bunny-bun-override", `${platformOS}-${platformArch}`);
  const binExt = platformOS === "win" ? ".exe" : "";
  const overrideBinary = join(overrideDir, `bun${binExt}`);
  const bunUrl = `https://github.com/${GITHUB_CONFIG.bunOwner}/${GITHUB_CONFIG.bunRepo}/releases/download/${releaseTag}/${assetName}`;

  console.log(`Using Bunny Bun: ${releaseTag}`);
  await downloadAndSetupBun(bunUrl, dirName, overrideDir, overrideBinary, binExt, releaseTag, platformOS, platformArch);
}

// ── 共享辅助函数 ────────────────────────────────────────────────────

/**
 * 获取 Bun 下载的 URL 段和目录名
 */
function getBunDownloadInfo(
  platformOS: "macos" | "win" | "linux",
  platformArch: "arm64" | "x64",
  isBunnyBun = false,
): { bunUrlSegment: string; bunDirName: string } {
  if (platformOS === "win") {
    const suffix = platformArch === "arm64" ? "aarch64" : "x64-baseline";
    const bunnySuffix = platformArch === "arm64" ? "arm64" : "x64";
    return {
      bunUrlSegment: isBunnyBun ? `bun-windows-${bunnySuffix}.zip` : `bun-windows-${suffix}.zip`,
      bunDirName: isBunnyBun ? `bun-windows-${bunnySuffix}` : `bun-windows-${suffix}`,
    };
  }
  if (platformOS === "macos") {
    const suffix = platformArch === "arm64" ? (isBunnyBun ? "arm64" : "aarch64") : "x64";
    return {
      bunUrlSegment: `bun-darwin-${suffix}.zip`,
      bunDirName: `bun-darwin-${suffix}`,
    };
  }
  // Linux
  const suffix = platformArch === "arm64" ? "arm64" : "x64";
  return {
    bunUrlSegment: `bun-linux-${suffix}.zip`,
    bunDirName: `bun-linux-${suffix}`,
  };
}

/**
 * 下载并解压 Bun 二进制，设置到指定目录
 */
async function downloadAndSetupBun(
  bunUrl: string,
  dirName: string,
  overrideDir: string,
  overrideBinary: string,
  binExt: string,
  version: string,
  platformOS: "macos" | "win" | "linux",
  _platformArch: "arm64" | "x64",
): Promise<void> {
  mkdirSync(overrideDir, { recursive: true });
  const tempZipPath = join(overrideDir, "temp.zip");

  try {
    await downloadFile(bunUrl, tempZipPath, { label: "Bun" });

    console.log("Extracting Bun...");
    const extractDir = join(overrideDir, dirName);
    await extractZip(tempZipPath, overrideDir);

    const extractedBinary = join(extractDir, `bun${binExt}`);
    if (existsSync(extractedBinary)) {
      renameSync(extractedBinary, overrideBinary);
    } else {
      // Try top-level binary (some packages extract directly to overrideDir)
      const topLevelBinary = join(overrideDir, `bun${binExt}`);
      if (existsSync(topLevelBinary) && topLevelBinary !== overrideBinary) {
        // Already at the right place
      } else {
        throw new Error(`Bun binary not found after extraction at ${extractedBinary}`);
      }
    }

    // Copy .dat files (ICU data)
    if (existsSync(extractDir)) {
      for (const file of readdirSync(extractDir)) {
        if (file.endsWith(".dat")) {
          renameSync(join(extractDir, file), join(overrideDir, file));
        }
      }
    }

    writeFileSync(join(overrideDir, ".bun-version"), version);

    // Cleanup
    if (existsSync(tempZipPath)) unlinkSync(tempZipPath);
    if (existsSync(extractDir)) rmSync(extractDir, { recursive: true, force: true });

    console.log(`Bun ${version} for ${platformOS} set up successfully`);
  } catch (error: any) {
    if (existsSync(overrideDir)) {
      try { rmSync(overrideDir, { recursive: true, force: true }); } catch { }
    }
    console.error(`Failed to set up Bun:`, error.message);
    process.exit(1);
  }
}
