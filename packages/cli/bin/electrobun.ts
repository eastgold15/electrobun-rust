#!/usr/bin/env bun
/**
 * Electrobun CLI entry point
 *
 * Instead of downloading a prebuilt binary, this runs the TypeScript CLI
 * source directly via Bun. The postinstall script handles downloading core
 * runtime binaries (bun.exe, launcher, etc.) so no network is needed at
 * build time.
 */

import { join } from "path";
import { spawn } from "bun";

const cliPath = join(import.meta.dirname, "..", "src", "cli", "index.ts");

const child = spawn({
  cmd: ["bun", "run", cliPath, ...process.argv.slice(2)],
  stdout: "inherit",
  stderr: "inherit",
  stdin: "inherit",
});

process.exit(await child.exited);
