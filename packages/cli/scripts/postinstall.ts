#!/usr/bin/env bun

/**
 * Postinstall script — downloads platform-specific core binaries
 * and compiles preload scripts.
 *
 * Runs automatically after `bun install` / `npm install` so that
 * the first `bun dev` / `bun run build` doesn't need a network
 * request.
 *
 * If the download fails (offline, proxy, …) the install still
 * succeeds — the CLI's `ensureCoreDependencies` in index.ts acts
 * as a runtime fallback.
 */

import { copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, unlinkSync } from "fs";
import { join } from "path";
import { platform, arch } from "os";

const PACKAGE_ROOT = join(import.meta.dirname, "..");

// ── Read version ──────────────────────────────────────────────
const VERSION: string = JSON.parse(
  readFileSync(join(PACKAGE_ROOT, "package.json"), "utf-8"),
).version;

// ── Detect platform ───────────────────────────────────────────
const OS: "win" | "linux" | "macos" =
  platform() === "win32" ? "win" : platform() === "darwin" ? "macos" : "linux";
const ARCH: "arm64" | "x64" =
  arch() === "arm64" ? "arm64" : "x64";
const platformName = OS === "macos" ? "darwin" : OS;
const platformDistDir = join(PACKAGE_ROOT, `dist-${OS}-${ARCH}`);

// ── Helper: compile preload scripts ───────────────────────────
async function compilePreloads() {
  const preloadDir = join(PACKAGE_ROOT, "src", "core", "bun", "preload");
  const fullResult = await Bun.build({
    entrypoints: [join(preloadDir, "index.ts")],
    target: "browser",
    format: "esm",
    minify: false,
  });
  if (!fullResult.success) {
    throw new Error("Failed to compile full preload: " + JSON.stringify(fullResult.logs));
  }
  const sandboxedResult = await Bun.build({
    entrypoints: [join(preloadDir, "index-sandboxed.ts")],
    target: "browser",
    format: "esm",
    minify: false,
  });
  if (!sandboxedResult.success) {
    throw new Error("Failed to compile sandboxed preload: " + JSON.stringify(sandboxedResult.logs));
  }
  const fullPreloadJs = "(function(){" + (await fullResult.outputs[0].text()) + "})();";
  const sandboxedPreloadJs = "(function(){" + (await sandboxedResult.outputs[0].text()) + "})();";
  mkdirSync(join(PACKAGE_ROOT, "dist"), { recursive: true });
  mkdirSync(platformDistDir, { recursive: true });
  Bun.write(join(PACKAGE_ROOT, "dist", "preload-full.js"), fullPreloadJs);
  Bun.write(join(PACKAGE_ROOT, "dist", "preload-sandboxed.js"), sandboxedPreloadJs);
  Bun.write(join(platformDistDir, "preload-full.js"), fullPreloadJs);
  Bun.write(join(platformDistDir, "preload-sandboxed.js"), sandboxedPreloadJs);
}

// ── Skip if already cached ────────────────────────────────────
if (existsSync(platformDistDir)) {
  const entries = readdirSync(platformDistDir).filter(
    (f) => !f.startsWith(".") && f !== "api" && f !== "rust-asar",
  );
  if (entries.length > 3) {
    console.log("  Core binaries already exist for " + OS + "-" + ARCH + ".");
    // Still need to compile preload scripts (they're not in the tarball)
    await compilePreloads();
    console.log("  Preload scripts up to date.");
    process.exit(0);
  }
}

// ── Download ──────────────────────────────────────────────────
// GitHub configuration - uses your fork
const GITHUB_OWNER = process.env.ELECTROBUN_GITHUB_OWNER || "eastgold15";
const GITHUB_REPO = process.env.ELECTROBUN_GITHUB_REPO || "electrobun-rust";
const url = "https://github.com/" + GITHUB_OWNER + "/" + GITHUB_REPO + "/releases/download/v" + VERSION + "/electrobun-core-" + platformName + "-" + ARCH + ".tar.gz";
console.log("\n  Downloading core binaries for " + OS + "-" + ARCH + "...");

try {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error("HTTP " + response.status + " " + response.statusText);
  }

  const tempFile = join(PACKAGE_ROOT, ".core-" + OS + "-" + ARCH + "-temp.tar.gz");
  await Bun.write(tempFile, response);

  const { size } = statSync(tempFile);
  console.log("  Downloaded " + (size / 1024 / 1024).toFixed(1) + " MB, extracting...");

  // Extract
  mkdirSync(platformDistDir, { recursive: true });
  const archive = new Bun.Archive(await Bun.file(tempFile).arrayBuffer());
  await archive.extract(platformDistDir);
  unlinkSync(tempFile);

  // ── Compile preload scripts from TypeScript source ──────────
  console.log("  Compiling preload scripts...");
  await compilePreloads();
  console.log("  Preload scripts compiled.");

  // ── Copy shared files to dist/ ──────────────────────────────
  const sharedDistDir = join(PACKAGE_ROOT, "dist");
  const sharedItems = [
    { name: "main.js" },
    { name: "preload-full.js" },
    { name: "preload-sandboxed.js" },
    { name: "api", recursive: true },
  ];
  for (const item of sharedItems) {
    const src = join(platformDistDir, item.name);
    const dest = join(sharedDistDir, item.name);
    if (existsSync(src) && !existsSync(dest)) {
      if (item.recursive) {
        cpSync(src, dest, { recursive: true, dereference: true });
      } else {
        copyFileSync(src, dest);
      }
    }
  }

  console.log("  Core binaries for " + OS + "-" + ARCH + " installed.");
} catch (error: any) {
  console.error("  Warning: Failed to download core binaries: " + error.message);
  console.error("  -> The CLI will download them automatically when first needed.");
}
