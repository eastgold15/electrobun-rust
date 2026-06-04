import { arch, platform } from "os";

export type SupportedOS = "macos" | "win" | "linux";
export type SupportedArch = "arm64" | "x64";

// Cache platform() result to avoid multiple system calls
const platformName = platform();
const archName = arch();

// Determine OS once
export const OS: SupportedOS = (() => {
  switch (platformName) {
    case "win32":
      return "win";
    case "darwin":
      return "macos";
    case "linux":
      return "linux";
    default:
      throw new Error(`Unsupported platform: ${platformName}`);
  }
})();

// Determine ARCH once, with Windows ARM64 fallback
export const ARCH: SupportedArch = (() => {
  // 检测真实架构（Windows ARM64 可通过 x64 模拟运行，但原生 ARM64 更优）
  switch (archName) {
    case "arm64":
      return "arm64";
    case "x64":
      return "x64";
    default:
      // 未知架构 fallback 到 x64
      return "x64";
  }
})();

// Export functions for backwards compatibility if needed
export function getPlatformOS(): SupportedOS {
  return OS;
}

export function getPlatformArch(): SupportedArch {
  return ARCH;
}
