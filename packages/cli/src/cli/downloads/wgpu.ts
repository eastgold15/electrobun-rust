import {
  existsSync,
  mkdirSync,
  renameSync,
  rmSync,
  readdirSync,
  readFileSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from "fs";
import { join } from "path";
import { ELECTROBUN_DEP_PATH, ELECTROBUN_CACHE_PATH } from "../platform-paths";
import { downloadFile, extractTarGz } from "../../../src/core/shared/download";

const GITHUB_CONFIG = {
  dawnOwner: 'blackboardsh',
  dawnRepo: 'electrobun-dawn',
};

/**
 * Returns the effective WGPU directory path
 */
export function getEffectiveWGPUDir(
  platformOS: "macos" | "win" | "linux",
  platformArch: "arm64" | "x64",
): string {
  return join(
    ELECTROBUN_CACHE_PATH,
    "wgpu",
    `${platformOS}-${platformArch}`,
  );
}

/**
 * Ensures WGPU dependencies are available
 */
export async function ensureWGPUDependencies(
  targetOS?: "macos" | "win" | "linux",
  targetArch?: "arm64" | "x64",
  wgpuVersion?: string,
): Promise<string> {
  const platformOS = targetOS || (process.platform === "win32" ? "win" : process.platform === "darwin" ? "macos" : "linux") as "macos" | "win" | "linux";
  const platformArch = targetArch || (process.arch === "arm64" ? "arm64" : "x64") as "arm64" | "x64";
  const wgpuDir = getEffectiveWGPUDir(platformOS, platformArch);
  const versionFile = join(wgpuDir, ".wgpu-version");

  const normalizedVersion =
    wgpuVersion && wgpuVersion.length > 0
      ? wgpuVersion.startsWith("v") ? wgpuVersion : `v${wgpuVersion}`
      : "latest";

  if (existsSync(wgpuDir) && existsSync(versionFile)) {
    const cachedVersion = readFileSync(versionFile, "utf8").trim();
    if (cachedVersion === normalizedVersion) {
      console.log(`WGPU ${normalizedVersion} already cached for ${platformOS}-${platformArch} at ${wgpuDir}`);
      return wgpuDir;
    }
    console.log(`Cached WGPU version "${cachedVersion}" does not match requested "${normalizedVersion}", re-downloading...`);
    rmSync(wgpuDir, { recursive: true, force: true });
  } else if (existsSync(wgpuDir)) {
    rmSync(wgpuDir, { recursive: true, force: true });
  }

  const platformName = platformOS === "macos" ? "darwin" : platformOS === "win" ? "win32" : "linux";
  const archName = platformArch;
  const baseUrl =
    normalizedVersion === "latest"
      ? `https://github.com/${GITHUB_CONFIG.dawnOwner}/${GITHUB_CONFIG.dawnRepo}/releases/latest/download`
      : `https://github.com/${GITHUB_CONFIG.dawnOwner}/${GITHUB_CONFIG.dawnRepo}/releases/download/${normalizedVersion}`;
  const tarballUrl = `${baseUrl}/electrobun-dawn-${platformName}-${archName}.tar.gz`;

  try {
    console.log(`WGPU dependencies not found for ${platformOS}-${platformArch}, downloading...`);
    const tempFile = join(ELECTROBUN_DEP_PATH, `wgpu-${platformOS}-${platformArch}.tar.gz`);
    await downloadFile(tarballUrl, tempFile, {
      label: `WGPU ${platformOS}-${platformArch}`,
    });

    const tempExtractDir = join(ELECTROBUN_DEP_PATH, `wgpu-extract-${platformOS}-${platformArch}`);
    mkdirSync(tempExtractDir, { recursive: true });
    await extractTarGz(tempFile, tempExtractDir);

    mkdirSync(wgpuDir, { recursive: true });
    const extractedItems = readdirSync(tempExtractDir);

    const moveAll = (fromDir: string) => {
      for (const item of readdirSync(fromDir)) {
        const src = join(fromDir, item);
        const dest = join(wgpuDir, item);
        if (existsSync(dest)) rmSync(dest, { recursive: true, force: true });
        renameSync(src, dest);
      }
    };

    if (extractedItems.length === 1) {
      const firstItem = extractedItems[0]!;
      const single = join(tempExtractDir, firstItem);
      const st = statSync(single);
      if (st.isDirectory()) {
        moveAll(single);
      } else {
        moveAll(tempExtractDir);
      }
    } else {
      moveAll(tempExtractDir);
    }

    writeFileSync(versionFile, normalizedVersion);
    rmSync(tempExtractDir, { recursive: true, force: true });
    unlinkSync(tempFile);

    console.log(`✓ WGPU dependencies for ${platformOS}-${platformArch} downloaded and cached successfully`);
    return wgpuDir;
  } catch (error: any) {
    if (existsSync(wgpuDir)) {
      try { rmSync(wgpuDir, { recursive: true, force: true }); } catch { }
    }
    console.error(`Failed to download WGPU dependencies:`, error.message);
    process.exit(1);
  }
}
