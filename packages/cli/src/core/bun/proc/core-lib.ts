import { join } from "path";

// todo: set up FFI, this is already in the webworker.

import { type CString, dlopen, ptr, suffix } from "bun:ffi";
import {
  preloadScript,
  preloadScriptSandboxed,
} from "../preload/.generated/compiled";
import { CORE_SYMBOLS } from "./ffi-symbols";

export function getWindowPtr(winId: number) {
  // 窗口指针在事件循环线程上无法直接返回，传 winId 让 native 侧自己查找
  return winId as any;
}

export function getCoreLastError(): string | null {
  const error = core?.symbols.electrobun_core_last_error();
  if (!error) {
    return null;
  }

  const message = error.toString();
  return message.length > 0 ? message : null;
}

let webviewRuntimeConfigured = false;

export function ensureWebviewRuntimeConfigured() {
  if (webviewRuntimeConfigured) {
    return;
  }

  const configured = core?.symbols.electrobun_configure_webview_runtime(
    0,
    toCString(preloadScript),
    toCString(preloadScriptSandboxed)
  );

  if (!configured) {
    throw getCoreLastError() || "Failed to configure webview runtime";
  }

  // Set views root for the `views://` custom protocol
  try {
    const binDir = join(process.execPath, "..");
    const viewsRoot = join(binDir, "..", "Resources", "app", "views");
    core?.symbols.electrobun_set_views_root(toCString(viewsRoot));
  } catch (e) {
    console.warn("[CORE] Failed to set views root:", e);
  }

  webviewRuntimeConfigured = true;
}

const core = (() => {
  try {
    // Use execPath (the bun binary's dir) instead of cwd to avoid issues
    // when the app is launched from shortcuts or different working directories.
    // native.ts already does this for libNativeWrapper — do the same for core.
    const corePath = join(
      process.execPath,
      "..",
      process.platform === "win32"
        ? "electrobun_core.dll"
        : `libelectrobun_core.${suffix}`
    );
    console.log("[CORE] Loading:", corePath);
    return dlopen(corePath, {
      ...CORE_SYMBOLS,
    });
  } catch {
    return null;
  }
})();

export const native = (() => {
  const coreDll =
    process.platform === "win32"
      ? "electrobun_core.dll"
      : `libelectrobun_core.${suffix}`;
  const corePath = join(process.execPath, "..", coreDll);
  let native: any;
  try {
    native = dlopen(corePath, {
      ...CORE_SYMBOLS,
    });
  } catch (e) {
    console.log("[CORE] Failed to load native:", e);
    return null;
  }

  return native;
})();

export const hasFFI = native !== null && core !== null;

// Non-null assertions for internal use (only access when hasFFI is confirmed true)
export const core_ = core!;
export const native_ = native!;

// Note: When passed over FFI JS will GC the buffer/pointer. Make sure to use strdup() or something
// on the c side to duplicate the string so objc/c++ gc can own it
export function toCString(jsString: string, addNullTerminator = true): CString {
  let appendWith = "";

  if (addNullTerminator && !jsString.endsWith("\0")) {
    appendWith = "\0";
  }
  const buff = Buffer.from(jsString + appendWith, "utf8");

  // @ts-expect-error - This is valid in Bun
  return ptr(buff);
}
