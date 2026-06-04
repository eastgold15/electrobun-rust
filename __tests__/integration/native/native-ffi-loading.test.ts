/**
 * Native FFI DLL 加载测试
 *
 * 验证 electrobun_core.dll 能被 dlopen 加载，
 * 并且 FFI 签名声明与 DLL 实际导出符号兼容。
 *
 * 用法: bun test __tests__/integration/native/native-ffi-loading.test.ts
 */

import { dlopen, FFIType, suffix } from "bun:ffi";
import { join, dirname } from "path";
import { existsSync, readFileSync } from "fs";
import { describe, it, expect } from "bun:test";

function findClosestDist(): string {
  let dir = import.meta.dir;
  for (let i = 0; i < 6; i++) {
    const candidate = join(dir, "dist");
    if (existsSync(candidate)) return candidate;
    // Also check packages/cli/dist
    const pkgCandidate = join(dir, "packages", "cli", "dist");
    if (existsSync(pkgCandidate)) return pkgCandidate;
    dir = dirname(dir);
  }
  throw new Error("Cannot find dist/ directory — build SDK first (bun run build:dev)");
}

const distDir = findClosestDist();
const isWin = process.platform === "win32";
const coreLibName = isWin ? "electrobun_core.dll" : `libelectrobun_core.${suffix}`;
const coreLibPath = join(distDir, coreLibName);

// ── electrobun_core.dll symbols ──────────────────────────────────────────
// These are the core FFI symbols exported by the Rust electrobun-core crate.
const CORE_SYMBOLS = {
  electrobun_core_last_error: { args: [] as any[], returns: FFIType.cstring },
  electrobun_free_core_string: { args: [FFIType.ptr], returns: FFIType.void },
  electrobun_core_run_main_thread: {
    args: [FFIType.cstring, FFIType.cstring, FFIType.cstring, FFIType.i32],
    returns: FFIType.i32,
  },
  electrobun_create_window: {
    args: [FFIType.cstring, FFIType.f64, FFIType.f64, FFIType.f64, FFIType.f64, FFIType.u32, FFIType.cstring, FFIType.bool, FFIType.cstring, FFIType.bool, FFIType.bool, FFIType.f64, FFIType.f64, FFIType.function, FFIType.function, FFIType.function, FFIType.function, FFIType.function, FFIType.function],
    returns: FFIType.u32,
  },
  electrobun_show_window: { args: [FFIType.u32, FFIType.bool], returns: FFIType.bool },
  electrobun_close_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_activate_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_hide_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_set_window_title: { args: [FFIType.u32, FFIType.cstring], returns: FFIType.bool },
  electrobun_set_window_frame: { args: [FFIType.u32, FFIType.f64, FFIType.f64, FFIType.f64, FFIType.f64], returns: FFIType.bool },
  electrobun_set_window_fullscreen: { args: [FFIType.u32, FFIType.bool], returns: FFIType.bool },
  electrobun_get_window_pointer: { args: [FFIType.u32], returns: FFIType.ptr },
  electrobun_get_platform: { args: [] as any[], returns: FFIType.cstring },
  electrobun_quit_gracefully: { args: [] as any[], returns: FFIType.void },
  electrobun_stop_event_loop: { args: [] as any[], returns: FFIType.void },
  electrobun_set_exit_on_last_window_closed: { args: [FFIType.bool], returns: FFIType.void },
  electrobun_get_primary_display: { args: [] as any[], returns: FFIType.cstring },
  electrobun_get_all_displays: { args: [] as any[], returns: FFIType.cstring },
  electrobun_get_cursor_screen_point: { args: [FFIType.ptr], returns: FFIType.bool },
} as const;

// ── Tests ────────────────────────────────────────────────────────────────

describe("Native FFI DLL Loading", () => {
  describe("electrobun_core.dll", () => {
    it("exists in dist/", () => {
      expect(existsSync(coreLibPath)).toBe(true);
    });

    it("loads with all declared symbols", () => {
      try {
        const core = dlopen(coreLibPath, CORE_SYMBOLS);
        expect(core).toBeTruthy();
        // Verify a few key functions are callable
        expect(core.symbols.electrobun_get_platform).toBeFunction();
        expect(core.symbols.electrobun_get_primary_display).toBeFunction();
        expect(core.symbols.electrobun_quit_gracefully).toBeFunction();
      } catch (e: any) {
        throw new Error(`Failed to load core DLL: ${e.message}`);
      }
    });

    it("returns platform string", () => {
      const core = dlopen(coreLibPath, {
        electrobun_get_platform: { args: [] as any[], returns: FFIType.cstring },
      });
      const platform = new TextDecoder().decode(
        (core.symbols.electrobun_get_platform() as Uint8Array),
      );
      const validPlatforms = ["macos", "win", "linux"];
      expect(validPlatforms).toContain(platform.replace(/\0/g, ""));
    });
  });
});
