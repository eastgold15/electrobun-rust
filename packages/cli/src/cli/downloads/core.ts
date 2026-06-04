import {
  existsSync,
  mkdirSync,
  cpSync,
  readdirSync,
  statSync,
  unlinkSync,
  realpathSync,
} from "fs";
import { join, dirname, basename } from "path";
import { ELECTROBUN_DEP_PATH, getPlatformPaths } from "../platform-paths";
import { ELECTROBUN_VERSION } from "../../../src/core/shared/electrobun-version";
import { ARCH, OS } from "../../../src/core/shared/platform";
import { downloadFile, extractTarGz } from "../../../src/core/shared/download";



const GITHUB_CONFIG = {
  owner: process.env.ELECTROBUN_GITHUB_OWNER || 'eastgold15',
  electrobunRepo: process.env.ELECTROBUN_GITHUB_REPO || 'electrobun-rust',
  bunOwner: 'blackboardsh',
  bunRepo: 'bun',
  dawnOwner: 'blackboardsh',
  dawnRepo: 'electrobun-dawn',
};

/**
 * Ensures core binaries are available for the target platform
 */
export async function ensureCoreDependencies(
  targetOS?: "macos" | "win" | "linux",
  targetArch?: "arm64" | "x64",
) {
  const platformOS = targetOS || OS;
  const platformArch = targetArch || ARCH;
  const platformPaths = getPlatformPaths(platformOS, platformArch);

  const requiredBinaries = [
    platformPaths.BUN_BINARY,
    platformPaths.BSDIFF,
    platformPaths.BSPATCH,
  ];
  if (platformOS === "macos" || platformOS === "win") {
    requiredBinaries.push(platformPaths.LAUNCHER_RELEASE);
  }

  const requiredSharedFiles = [platformPaths.MAIN_JS];

  const missingBinaries = requiredBinaries.filter((file) => !existsSync(file));
  const missingSharedFiles = requiredSharedFiles.filter(
    (file) => !existsSync(file),
  );

  if (missingBinaries.length === 0 && missingSharedFiles.length > 0) {
    console.log(
      `Shared files missing (expected in production): ${missingSharedFiles.map((f) => f.replace(ELECTROBUN_DEP_PATH, ".")).join(", ")}`,
    );
  }

  if (missingBinaries.length === 0) {
    return;
  }

  console.log(
    `Core dependencies not found for ${platformOS}-${platformArch}. Missing files:`,
    missingBinaries.map((f) => f.replace(ELECTROBUN_DEP_PATH, ".")).join(", "),
  );

  // 先尝试从本地 Rust 编译产物复制（开发模式），再回退到 GitHub 下载
  // 注意：ELECTROBUN_DEP_PATH 可能是 symlink，需要用 realpathSync 解析到真实位置
  const platformDistDir = join(ELECTROBUN_DEP_PATH, `dist-${platformOS}-${platformArch}`);
  const PACKAGE_DIR = existsSync(ELECTROBUN_DEP_PATH) ? realpathSync(ELECTROBUN_DEP_PATH) : ELECTROBUN_DEP_PATH;
  const TARGET_DIR = join(PACKAGE_DIR, "..", "..", "target");
  const binExt = platformOS === "win" ? ".exe" : "";
  const localArtifacts: Record<string, string> = {
    [platformPaths.LAUNCHER_RELEASE]: join(TARGET_DIR, "debug", `launcher${binExt}`),
    [platformPaths.BUN_BINARY]: "", // bun 没有本地 Rust 编译，走下载
    [platformPaths.BSDIFF]: join(TARGET_DIR, "debug", `bsdiff${binExt}`),
    [platformPaths.BSPATCH]: join(TARGET_DIR, "debug", `bspatch${binExt}`),
    [platformPaths.ZSTD]: join(TARGET_DIR, "debug", `rust-zstd${binExt}`),
  };
  if (platformOS === "win") {
    localArtifacts[join(platformDistDir, "electrobun_core.dll")] = join(TARGET_DIR, "debug", "electrobun_core.dll");
  } else if (platformOS === "macos") {
    localArtifacts[join(platformDistDir, "libelectrobun_core.dylib")] = join(TARGET_DIR, "debug", "libelectrobun_core.dylib");
  } else {
    localArtifacts[join(platformDistDir, "libelectrobun_core.so")] = join(TARGET_DIR, "debug", "libelectrobun_core.so");
  }

  let localFound = false;
  for (const [dest, src] of Object.entries(localArtifacts)) {
    if (!src) continue;
    if (existsSync(src) && !existsSync(dest)) {
      mkdirSync(dirname(dest), { recursive: true });
      cpSync(src, dest, { dereference: true });
      localFound = true;
      console.log(`  ✓ 从本地 Rust 构建复制: ${basename(src)}`);
    }
  }

  if (localFound) {
    const allFound = requiredBinaries.every((f) => existsSync(f));
    if (allFound) {
      console.log(`✓ 本地 Rust 构建产物已就绪`);
      return;
    }
    console.log(`部分文件仍缺失，尝试从 GitHub 下载...`);
  }

  console.log(`Downloading core binaries for ${platformOS}-${platformArch}...`);

  const version = `v${ELECTROBUN_VERSION}`;
  const platformName =
    platformOS === "macos" ? "darwin" : platformOS === "win" ? "win" : "linux";
  const archName = platformArch;
  const coreTarballUrl = `https://github.com/${GITHUB_CONFIG.owner}/${GITHUB_CONFIG.electrobunRepo}/releases/download/${version}/electrobun-core-${platformName}-${archName}.tar.gz`;

  console.log(`Downloading core binaries from: ${coreTarballUrl}`);

  try {
    const platformDistPath = join(
      ELECTROBUN_DEP_PATH,
      `dist-${platformOS}-${platformArch}`,
    );
    mkdirSync(platformDistPath, { recursive: true });

    const tempFile = join(platformDistPath, `..`, `.core-temp.tar.gz`);
    await downloadFile(coreTarballUrl, tempFile, {
      label: `Core ${platformOS}-${platformArch}`,
    });
    await extractTarGz(tempFile, platformDistPath);
    try { unlinkSync(tempFile); } catch { /* ignore */ }

    try {
      const extractedFiles = readdirSync(platformDistPath);
      console.log(`Extracted files to ${platformDistPath}:`, extractedFiles);
      for (const file of extractedFiles) {
        const filePath = join(platformDistPath, file);
        const st = statSync(filePath);
        if (st.isDirectory()) {
          const subFiles = readdirSync(filePath);
          console.log(`  ${file}/: ${subFiles.join(", ")}`);
        }
      }
    } catch (e) {
      console.error("Could not list extracted files:", e);
    }

    const postExtractionBinaries = [
      platformPaths.BUN_BINARY,
      platformPaths.BSDIFF,
      platformPaths.BSPATCH,
      platformPaths.ZSTD,
    ];
    if (platformOS === "macos" || platformOS === "win") {
      postExtractionBinaries.push(platformPaths.LAUNCHER_RELEASE);
    }

    const missingAfter = postExtractionBinaries.filter(
      (file) => !existsSync(file),
    );
    if (missingAfter.length > 0) {
      console.error(
        `Missing binaries after extraction: ${missingAfter.map((f) => f.replace(ELECTROBUN_DEP_PATH, ".")).join(", ")}`,
      );
      console.error("This suggests the tarball structure is different than expected");
    }

    // Development fallback: copy shared files from platform-specific download
    const sharedDistPath = join(ELECTROBUN_DEP_PATH, "dist");
    const fallbackItems = [
      { name: "main.js" },
      { name: "preload-full.js" },
      { name: "preload-sandboxed.js" },
      { name: "api", recursive: true },
    ];

    for (const item of fallbackItems) {
      const src = join(platformDistPath, item.name);
      const dest = join(sharedDistPath, item.name);
      if (existsSync(src) && !existsSync(dest)) {
        console.log(`Development fallback: copying ${item.name} from platform-specific download to shared dist/`);
        mkdirSync(sharedDistPath, { recursive: true });
        if (item.recursive) {
          cpSync(src, dest, { dereference: true, recursive: true });
        } else {
          cpSync(src, dest, { dereference: true });
        }
      }
    }

    console.log(`Core dependencies for ${platformOS}-${platformArch} downloaded and cached successfully`);
  } catch (error: any) {
    console.error(`Failed to download core dependencies for ${platformOS}-${platformArch}:`, error.message);
    console.error("Please ensure you have an internet connection and the release exists.");
    process.exit(1);
  }
}
