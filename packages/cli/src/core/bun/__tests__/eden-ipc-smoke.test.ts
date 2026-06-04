/**
 * #[eden_ipc] 宏生成 FFI 客户端冒烟测试
 *
 * 注意：宏生成的 FFI 函数需要 launcher 先调用 XXX_INSTANCE_init
 * 注入实例。纯测试环境下只能验证 dlopen 加载正常。
 *
 * 运行：cd packages/cli && bun test src/core/bun/__tests__/eden-ipc-smoke.test.ts
 * 前置条件：electrobun_core.dll + 运行时依赖（WebView2Loader 等）在 bun 同级目录
 *           cp target/debug/*.dll "$(dirname $(bun -e 'console.log(process.execPath)'))/"
 */

import { describe, expect, it } from "bun:test";

// ─── 窗口 API ─────────────────────────────────────────────────

describe("windowAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/WindowAPIClient");
    expect(mod.windowAPI).toBeDefined();
    expect(typeof mod.windowAPI.createWindow).toBe("function");
    expect(typeof mod.windowAPI.closeWindow).toBe("function");
  });

  it("所有 api 函数名称存在", async () => {
    const mod = await import("../../generated/WindowAPIClient");
    const methods = [
      "createWindow", "closeWindow", "showWindow", "hideWindow",
      "minimizeWindow", "maximizeWindow", "restoreWindow", "focusWindow",
      "setWindowTitle", "setWindowSize", "setWindowPosition",
      "setWindowFullscreen", "setWindowAlwaysOnTop", "setWindowFrame",
      "getWindowBounds", "isMinimized", "isMaximized", "isFullscreen",
      "isAlwaysOnTop", "unmaximize",
    ];
    for (const m of methods) {
      expect(typeof (mod.windowAPI as any)[m]).toBe("function");
    }
  });
});

// ─── WebView API ───────────────────────────────────────────────

describe("webViewAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/WebViewAPIClient");
    expect(mod.webViewAPI).toBeDefined();
    expect(typeof mod.webViewAPI.navigate).toBe("function");
  });

  it("所有 api 函数名称存在", async () => {
    const mod = await import("../../generated/WebViewAPIClient");
    const methods = [
      "createWebview", "closeWebview", "navigate", "navigateBack", "navigateForward",
      "canGoBack", "canGoForward", "reload", "loadHtml", "setWebviewBounds",
      "setWebviewVisible", "setTransparent", "setPassthrough", "resize",
      "sendMessage", "evaluateJavascript", "setNavigationRules", "findInPage",
      "stopFind", "openDevtools", "closeDevtools", "setPageZoom", "getPageZoom",
      "loadHtmlContent", "updatePreloadScript", "clearTransport",
    ];
    for (const m of methods) {
      expect(typeof (mod.webViewAPI as any)[m]).toBe("function");
    }
  });
});

// ─── 托盘 API ─────────────────────────────────────────────────

describe("trayAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/TrayAPIClient");
    expect(mod.trayAPI).toBeDefined();
    expect(typeof mod.trayAPI.createTray).toBe("function");
  });

  it("所有 api 函数名称存在", async () => {
    const mod = await import("../../generated/TrayAPIClient");
    for (const m of ["createTray", "destroyTray", "setTrayImage", "setTrayTitle",
      "showTray", "hideTray", "setTrayMenu", "getTrayBounds"]) {
      expect(typeof (mod.trayAPI as any)[m]).toBe("function");
    }
  });
});

// ─── 核心 API ─────────────────────────────────────────────────

describe("coreAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/CoreAPIClient");
    expect(mod.coreAPI).toBeDefined();
  });

  it("所有 api 函数名称存在", async () => {
    const mod = await import("../../generated/CoreAPIClient");
    for (const m of ["quitGracefully", "stopEventLoop", "forceExit",
      "setExitOnLastWindowClosed", "getPlatform"]) {
      expect(typeof (mod.coreAPI as any)[m]).toBe("function");
    }
  });
});

// ─── 剪贴板 API ───────────────────────────────────────────────

describe("clipboardAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/ClipboardAPIClient");
    expect(mod.clipboardAPI).toBeDefined();
  });

  it("readText 函数存在", async () => {
    const mod = await import("../../generated/ClipboardAPIClient");
    expect(typeof mod.clipboardAPI.readText).toBe("function");
  });
});

// ─── 对话框 API ───────────────────────────────────────────────

describe("dialogAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/DialogAPIClient");
    expect(mod.dialogAPI).toBeDefined();
  });
});

// ─── Session API ───────────────────────────────────────────────

describe("sessionAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/SessionAPIClient");
    expect(mod.sessionAPI).toBeDefined();
  });
});

// ─── 快捷键 API ───────────────────────────────────────────────

describe("shortcutsAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/ShortcutsAPIClient");
    expect(mod.shortcutsAPI).toBeDefined();
  });
});

// ─── 显示 API ─────────────────────────────────────────────────

describe("displayAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/DisplayAPIClient");
    expect(mod.displayAPI).toBeDefined();
  });
});

// ─── App API ──────────────────────────────────────────────────

describe("appAPI dlopen", () => {
  it("加载客户端不崩溃", async () => {
    const mod = await import("../../generated/AppAPIClient");
    expect(mod.appAPI).toBeDefined();
  });
});
