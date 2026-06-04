#!/usr/bin/env bun

import { execSync } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import process from "process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const platform = process.platform;
const arch = process.arch;

// Map Node.js platform/arch to our naming
const platformMap = {
	darwin: "darwin",
	linux: "linux",
	win32: "win",
};

const archMap = {
	x64: "x64",
	arm64: "arm64",
};

const platformName = platformMap[platform] || platform;
const archName = archMap[arch] || arch;

console.log(`Packaging Electrobun for ${platformName}-${archName}...`);

// Build everything including CLI (no CI mode needed)
console.log("Building full release...");
try {
	execSync("bun scripts/build-sdk.ts --release", { stdio: "inherit" });
} catch (error) {
	console.error("Build failed:", error.message);
	process.exit(1);
}

// Build CLI binary
console.log("Building CLI binary...");
if (!fs.existsSync("bin")) {
	fs.mkdirSync("bin", { recursive: true });
}

// Use baseline target for Windows to ensure compatibility with ARM64 emulation
const compileTarget =
	platform === "win32" ? "--target=bun-windows-x64-baseline" : "";
const vendoredBun = path.join(
	"vendors",
	"bun",
	platform === "win32" ? "bun.exe" : "bun",
);

// Workaround for Windows 2025 runner cross-drive issues with Bun cache
if (platform === "win32" && process.env.GITHUB_ACTIONS) {
	// Set Bun cache to same drive as workspace
	const workspaceDrive = process.cwd().substring(0, 2);
	const bunCacheDir = `${workspaceDrive}\\temp\\bun-cache`;
	console.log(`Setting BUN_INSTALL_CACHE_DIR to: ${bunCacheDir}`);

	// Ensure cache directory exists
	fs.mkdirSync(bunCacheDir, { recursive: true });

	// Set environment variable directly in the command for Windows
	execSync(
		`set "BUN_INSTALL_CACHE_DIR=${bunCacheDir}" && "${vendoredBun}" build src/cli/index.ts --compile ${compileTarget} --outfile bin/electrobun`,
		{ stdio: "inherit", shell: true },
	);
} else {
	execSync(
		`"${vendoredBun}" build src/cli/index.ts --compile ${compileTarget} --outfile bin/electrobun`,
		{ stdio: "inherit" },
	);
}

// Create separate tarballs for CLI and core binaries
const distPath = path.join(__dirname, "..", "dist");
const cliOutputFile = path.join(
	__dirname,
	"..",
	`electrobun-cli-${platformName}-${archName}.tar.gz`,
);
const coreOutputFile = path.join(
	__dirname,
	"..",
	`electrobun-core-${platformName}-${archName}.tar.gz`,
);

console.log(`Creating CLI tarball: ${cliOutputFile}`);

// Check if dist exists
if (!fs.existsSync(distPath)) {
	console.error("Error: dist directory not found");
	process.exit(1);
}

// Create a tar.gz file using system tar (preserves file permissions)
function createTarGz(tarGzPath, cwd, entries) {
	execSync(
		`tar -czf "${tarGzPath}" ${entries.map((e) => `"${e}"`).join(" ")}`,
		{
			cwd,
			stdio: "pipe",
		},
	);
}

async function createTarballs() {
	// Validate that we have platform-specific binaries, not just npm files
	const expectedBinaries = [
		platform === "win32" ? "electrobun.exe" : "electrobun",
		platform === "win32" ? "bun.exe" : "bun",
	];

	const missingBinaries = expectedBinaries.filter(
		(binary) => !fs.existsSync(path.join(distPath, binary)),
	);

	if (missingBinaries.length > 0) {
		console.error(
			`Error: Missing expected binaries in dist/: ${missingBinaries.join(", ")}`,
		);
		console.error("This suggests the build failed or was incomplete.");
		console.error("Contents of dist/:");
		if (fs.existsSync(distPath)) {
			fs.readdirSync(distPath).forEach((file) => console.error(`  ${file}`));
		} else {
			console.error("  (dist directory does not exist)");
		}
		process.exit(1);
	}

	console.log("Validation passed: Found expected platform binaries in dist/");

	// 1. Create CLI-only tarball
	const binPath = path.join(__dirname, "..", "bin");
	const cliSrc = path.join(
		binPath,
		"electrobun" + (platform === "win32" ? ".exe" : ""),
	);

	if (fs.existsSync(cliSrc)) {
		console.log(`Creating CLI tarball: ${cliOutputFile}`);

		// Create CLI tarball directly from bin directory (system tar preserves permissions)
		createTarGz(cliOutputFile, binPath, [
			"electrobun" + (platform === "win32" ? ".exe" : ""),
		]);

		const cliStats = fs.statSync(cliOutputFile);
		const cliSizeMB = (cliStats.size / 1024 / 1024).toFixed(2);
		console.log(`CLI tarball size: ${cliSizeMB} MB`);
	}

	// 2. Create core binaries tarball (exclude CLI binary)
	const coreFiles = fs
		.readdirSync(distPath)
		.filter((file) => !file.startsWith("electrobun"));

	if (coreFiles.length > 0) {
		console.log(`Creating core binaries tarball: ${coreOutputFile}`);

		createTarGz(coreOutputFile, distPath, coreFiles);

		const coreStats = fs.statSync(coreOutputFile);
		const coreSizeMB = (coreStats.size / 1024 / 1024).toFixed(2);
		console.log(`Core binaries tarball size: ${coreSizeMB} MB`);
	}
}

createTarballs().catch((err) => {
	console.error("Error creating tarballs:", err);
	process.exit(1);
});
