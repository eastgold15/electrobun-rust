import { describe, expect, it } from "bun:test";
import { OS, ARCH, getPlatformOS, getPlatformArch } from "./platform";
import type { SupportedOS, SupportedArch } from "./platform";

describe("platform detection", () => {
  it("OS is a valid supported OS", () => {
    const validOSes: SupportedOS[] = ["macos", "win", "linux"];
    expect(validOSes).toContain(OS);
  });

  it("ARCH is a valid supported arch", () => {
    const validArches: SupportedArch[] = ["arm64", "x64"];
    expect(validArches).toContain(ARCH);
  });

  it("getPlatformOS() returns the same value as OS", () => {
    expect(getPlatformOS()).toBe(OS);
  });

  it("getPlatformArch() returns the same value as ARCH", () => {
    expect(getPlatformArch()).toBe(ARCH);
  });

  it("ARCH is a valid supported arch on Windows too", () => {
    // ARCH 现在检测真实架构（arm64 或 x64），不再是硬编码的 x64
    if (OS === "win") {
      expect(["arm64", "x64"]).toContain(ARCH);
    }
  });
});
