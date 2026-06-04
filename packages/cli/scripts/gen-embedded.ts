#!/usr/bin/env bun
/**
 * Generate src/cli/templates/embedded.ts from the ../../templates/ directory.
 * Run once; the generated file is committed to the repo so the CLI can
 * always be invoked from source without a build step.
 *
 * NOTE: build-sdk.ts also has a generateTemplateEmbeddings() that does the same
 * thing but with extra platform handling. Keep changes synced.
 */

import { existsSync, readdirSync, readFileSync, writeFileSync, mkdirSync } from "fs";
import { join, dirname } from "path";

// templates/ is at repo root, 3 levels up from scripts/
const PACKAGE_ROOT = join(import.meta.dirname, "..");
const TEMPLATES_DIR = join(PACKAGE_ROOT, "..", "..", "templates");
const OUTPUT_FILE = join(PACKAGE_ROOT, "src", "templates", "embedded.ts");
const { version } = JSON.parse(readFileSync(join(PACKAGE_ROOT, "package.json"), "utf-8"));

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
      else {
        // 内容检测：含 null 字节则为二进制，否则 UTF-8 文本
        const raw = readFileSync(fp);
        files[rp] = raw.includes(0) ? "base64:" + raw.toString("base64") : raw.toString("utf-8");
      }
    }
  }
  readDir(tdir);

  // Pin @pori15/electrobun-rust (or legacy electrobun) version in template
  if (files["package.json"]) {
    const pkg = JSON.parse(files["package.json"]);
    for (const depName of ["@pori15/electrobun-rust", "electrobun"]) {
      const dep = pkg.dependencies?.[depName];
      if (typeof dep === "string" && (dep === "latest" || dep.startsWith("file:"))) {
        pkg.dependencies[depName] = version;
      }
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
