import { describe, expect, it, beforeEach, afterEach } from "bun:test";
import { existsSync, mkdirSync, writeFileSync, rmSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

// Mock process.cwd by changing directory
const ORIGINAL_CWD = process.cwd();

describe("defaultConfig", () => {
  let mod: typeof import("./config");

  beforeEach(async () => {
    // Re-import to get fresh module state
    mod = await import("./config");
  });

  it("has default app name", () => {
    expect(mod.defaultConfig.app.name).toBe("MyApp");
  });

  it("has default app identifier", () => {
    expect(mod.defaultConfig.app.identifier).toBe("com.example.myapp");
  });

  it("has default version", () => {
    expect(mod.defaultConfig.app.version).toBe("0.1.0");
  });

  it("has default build folder", () => {
    expect(mod.defaultConfig.build.buildFolder).toBe("build");
  });

  it("has default artifact folder", () => {
    expect(mod.defaultConfig.build.artifactFolder).toBe("artifacts");
  });

  it("has default entrypoint", () => {
    expect(mod.defaultConfig.build.bun.entrypoint).toBe("src/bun/index.ts");
  });

  it("has default mainProcess set to bun", () => {
    expect(mod.defaultConfig.build.mainProcess).toBe("bun");
  });

  it("has useAsar default to false", () => {
    expect(mod.defaultConfig.build.useAsar).toBe(false);
  });

  it("has macOS default entitlements for JIT", () => {
    const entitlements = mod.defaultConfig.build.mac.entitlements;
    expect(entitlements["com.apple.security.cs.allow-jit"]).toBe(true);
    expect(
      entitlements["com.apple.security.cs.allow-unsigned-executable-memory"],
    ).toBe(true);
    expect(
      entitlements["com.apple.security.cs.disable-library-validation"],
    ).toBe(true);
  });

  it("has codesign default to false", () => {
    expect(mod.defaultConfig.build.mac.codesign).toBe(false);
  });

  it("has createDmg default to true", () => {
    expect(mod.defaultConfig.build.mac.createDmg).toBe(true);
  });

  it("has empty script hooks by default", () => {
    expect(mod.defaultConfig.scripts.preBuild).toBe("");
    expect(mod.defaultConfig.scripts.postBuild).toBe("");
    expect(mod.defaultConfig.scripts.postWrap).toBe("");
    expect(mod.defaultConfig.scripts.postPackage).toBe("");
  });

  it("has generatePatch default to true", () => {
    expect(mod.defaultConfig.release.generatePatch).toBe(true);
  });

  it("does NOT include carrot in default config", () => {
    // Carrot was removed — this should not exist
    expect((mod.defaultConfig.build as any).carrot).toBeUndefined();
  });
});

describe("findConfigFile", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = join(tmpdir(), `electrobun-config-test-${Date.now()}`);
    mkdirSync(tempDir, { recursive: true });
    process.chdir(tempDir);
  });

  afterEach(() => {
    process.chdir(ORIGINAL_CWD);
    try {
      rmSync(tempDir, { recursive: true, force: true });
    } catch { /* ignore */ }
  });

  it("returns null when no electrobun.config.ts exists", async () => {
    const { findConfigFile } = await import("./config");
    expect(findConfigFile()).toBeNull();
  });

  it("returns path when electrobun.config.ts exists", () => {
    const configPath = join(tempDir, "electrobun.config.ts");
    writeFileSync(configPath, "export default {};");

    // Re-import to pick up new cwd
    const { findConfigFile } = require("./config") as typeof import("./config");
    expect(findConfigFile()).toBe(configPath);
  });
});
