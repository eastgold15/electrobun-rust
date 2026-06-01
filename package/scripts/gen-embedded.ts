#!/usr/bin/env bun
/**
 * Generate src/cli/templates/embedded.ts from the ../templates/ directory.
 * Run once; the generated file is committed to the repo so the CLI can
 * always be invoked from source without a build step.
 */

import { existsSync, readdirSync, readFileSync, writeFileSync, mkdirSync } from "fs";
import { join, dirname } from "path";

const TEMPLATES_DIR = join(import.meta.dirname, "..", "..", "templates");
const OUTPUT_FILE = join(import.meta.dirname, "..", "src", "cli", "templates", "embedded.ts");
const { version } = JSON.parse(readFileSync(join(import.meta.dirname, "..", "package.json"), "utf-8"));

if (!existsSync(TEMPLATES_DIR)) {
  console.log("No templates directory found, skipping");
  process.exit(0);
}

const templates: Record<string, { name: string; files: Record<string, string> }> = {};
const templateNames = readdirSync(TEMPLATES_DIR, { withFileTypes: true })
  .filter((d) => d.isDirectory())
  .map((d) => d.name);

const SKIP = new Set([
  "node_modules", ".git", "build", "dist", ".next", ".DS_Store",
  "package-lock.json", "bun.lock", "bun.lockb", "yarn.lock",
]);

for (const tname of templateNames) {
  const tdir = join(TEMPLATES_DIR, tname);
  const files: Record<string, string> = {};

  function readDir(dir: string, base: string = "") {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (SKIP.has(entry.name) || (entry.name.startsWith(".") && entry.name !== ".gitignore")) continue;
      const fp = join(dir, entry.name);
      const rp = join(base, entry.name).replace(/\\/g, "/");
      if (entry.isDirectory()) readDir(fp, rp);
      else files[rp] = readFileSync(fp, "utf-8");
    }
  }
  readDir(tdir);

  // Pin electrobun version in template
  if (files["package.json"]) {
    const pkg = JSON.parse(files["package.json"]);
    const dep = pkg.dependencies?.electrobun;
    if (typeof dep === "string" && (dep === "latest" || dep.startsWith("file:"))) {
      pkg.dependencies.electrobun = version;
    }
    files["package.json"] = JSON.stringify(pkg, null, "\t") + "\n";
  }

  templates[tname] = { name: tname, files };
}

const output = `// Auto-generated file. Do not edit directly.
// Generated from templates/ directory

export interface Template {
  name: string;
  files: Record<string, string>;
}

export const templates: Record<string, Template> = ${JSON.stringify(templates, null, 2)};

export function getTemplateNames(): string[] {
  return Object.keys(templates);
}

export function getTemplate(name: string): Template | undefined {
  return templates[name];
}
`;

const outDir = dirname(OUTPUT_FILE);
if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });
writeFileSync(OUTPUT_FILE, output);

const totalFiles = Object.values(templates).reduce((a, t) => a + Object.keys(t.files).length, 0);
console.log(`  ✔ Generated embedded.ts with ${templateNames.length} templates, ${totalFiles} files`);
