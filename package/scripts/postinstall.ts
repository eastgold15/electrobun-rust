#!/usr/bin/env bun

/**
 * Postinstall script — downloads platform-specific core binaries
 *
 * Runs automatically after `bun install` / `npm install` so that
 * the first `bun dev` / `bun run build` doesn't need a network
 * request.
 *
 * If the download fails (offline, proxy, …) the install still
 * succeeds — the CLI's `ensureCoreDependencies` in index.ts acts
 * as a runtime fallback.
 */

import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, unlinkSync } from "fs";
import { join } from "path";
import { platform, arch } from "os";

const PACKAGE_ROOT = join(import.meta.dirname, "..");

// ── Read version ──────────────────────────────────────────────
const { version: VERSION } = JSON.parse(
	readFileSync(join(PACKAGE_ROOT, "package.json"), "utf-8"),
);

// ── Detect platform ───────────────────────────────────────────
const OS: "win" | "linux" | "macos" =
	platform() === "win32" ? "win" : platform() === "darwin" ? "macos" : "linux";
const ARCH: "arm64" | "x64" =
	arch() === "arm64" ? "arm64" : "x64";
const platformName = OS === "macos" ? "darwin" : OS;
const platformDistDir = join(PACKAGE_ROOT, `dist-${OS}-${ARCH}`);

// ── Skip if already cached ────────────────────────────────────
if (existsSync(platformDistDir)) {
	const entries = readdirSync(platformDistDir).filter(
		(f) => !f.startsWith(".") && f !== "api" && f !== "zig-sdk" && f !== "zig-asar",
	);
	if (entries.length > 3) {
		console.log(`  ✔ Core binaries already exist for ${OS}-${ARCH}, skipping download.`);
		process.exit(0);
	}
}

// ── Download ──────────────────────────────────────────────────
const url = `https://github.com/blackboardsh/electrobun/releases/download/v${VERSION}/electrobun-core-${platformName}-${ARCH}.tar.gz`;
console.log(`\n  Downloading core binaries for ${OS}-${ARCH}…`);

try {
	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`HTTP ${response.status} ${response.statusText}`);
	}

	const tempFile = join(PACKAGE_ROOT, `.core-${OS}-${ARCH}-temp.tar.gz`);
	await Bun.write(tempFile, response);

	const { size } = statSync(tempFile);
	console.log(`  Downloaded ${(size / 1024 / 1024).toFixed(1)} MB, extracting…`);

	// Extract
	mkdirSync(platformDistDir, { recursive: true });
	const archive = new Bun.Archive(await Bun.file(tempFile).arrayBuffer());
	await archive.extract(platformDistDir);
	unlinkSync(tempFile);

	console.log(`  ✔ Core binaries for ${OS}-${ARCH} installed.`);
} catch (error: any) {
	console.error(`  ⚠ Failed to download core binaries: ${error.message}`);
	console.error(`  → The CLI will download them automatically when first needed.`);
}
