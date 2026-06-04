/**
 * #[eden_ipc] 宏生成 FFI 客户端冒烟测试
 *
 * 直接调用各个 *APIClient.ts，验证：
 * 1. dlopen 加载正常（不崩溃）
 * 2. 函数签名匹配 Rust 侧
 * 3. 返回值格式符合预期
 *
 * 运行：cd packages/cli && bun test src/core/bun/__tests__/eden-ipc-smoke.test.ts
 */

import { describe, expect, it } from "bun:test";

// ─── 窗口 API ─────────────────────────────────────────────────

describe("windowAPI (WindowAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { windowAPI } = await import("../../generated/WindowAPIClient");
    expect(windowAPI).toBeDefined();
    expect(typeof windowAPI.createWindow).toBe("function");
  });

  it("isMinimized(99999) 对无效窗口返回错误而不是崩溃", async () => {
    const { windowAPI } = await import("../../generated/WindowAPIClient");
    const result = windowAPI.isMinimized(99999);
    // 必须返回东西，不能 crash
    expect(result).toBeDefined();
    // 要么是 boolean，要么是错误对象
    const isBool = typeof result === "boolean";
    const isErr = typeof result === "object" && result !== null && "type" in result;
    expect(isBool || isErr).toBe(true);
  });

  it("isMaximized(99999) 不崩溃", async () => {
    const { windowAPI } = await import("../../generated/WindowAPIClient");
    const result = windowAPI.isMaximized(99999);
    expect(result).toBeDefined();
  });

  it("isFullscreen(99999) 不崩溃", async () => {
    const { windowAPI } = await import("../../generated/WindowAPIClient");
    const result = windowAPI.isFullscreen(99999);
    expect(result).toBeDefined();
  });

  it("isAlwaysOnTop(99999) 不崩溃", async () => {
    const { windowAPI } = await import("../../generated/WindowAPIClient");
    const result = windowAPI.isAlwaysOnTop(99999);
    expect(result).toBeDefined();
  });

  it("unmaximize(99999) 对无效窗口不崩溃", async () => {
    const { windowAPI } = await import("../../generated/WindowAPIClient");
    // unmaximize 返回 void | WindowError，应该不抛异常
    expect(() => {
      const r = windowAPI.unmaximize(99999);
      return r;
    }).not.toThrow();
  });

  it("getWindowBounds(99999) 返回错误", async () => {
    const { windowAPI } = await import("../../generated/WindowAPIClient");
    const result = windowAPI.getWindowBounds(99999);
    // 窗口不存在，应该是错误对象
    const isErr = typeof result === "object" && result !== null && "type" in result;
    expect(isErr).toBe(true);
  });
});

// ─── WebView API ───────────────────────────────────────────────

describe("webViewAPI (WebViewAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(webViewAPI).toBeDefined();
    expect(typeof webViewAPI.navigate).toBe("function");
  });

  it("canGoBack(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    const result = webViewAPI.canGoBack(99999);
    expect(result).toBeDefined();
  });

  it("canGoForward(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    const result = webViewAPI.canGoForward(99999);
    expect(result).toBeDefined();
  });

  it("setTransparent(99999, true) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.setTransparent(99999, true)).not.toThrow();
  });

  it("setPassthrough(99999, false) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.setPassthrough(99999, false)).not.toThrow();
  });

  it("evaluateJavascript(99999, '1+1') 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.evaluateJavascript(99999, "1+1")).not.toThrow();
  });

  it("setNavigationRules(99999, '{}') 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.setNavigationRules(99999, "{}")).not.toThrow();
  });

  it("findInPage(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.findInPage(99999, "test", true, false)).not.toThrow();
  });

  it("stopFind(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.stopFind(99999, true)).not.toThrow();
  });

  it("setPageZoom(99999) / getPageZoom(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.setPageZoom(99999, 1.0)).not.toThrow();
    const zoom = webViewAPI.getPageZoom(99999);
    expect(typeof zoom === "number" || typeof zoom === "object").toBe(true);
  });

  it("loadHtmlContent(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.loadHtmlContent(99999, "<html>")).not.toThrow();
  });

  it("clearTransport(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.clearTransport(99999)).not.toThrow();
  });

  it("openDevtools(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.openDevtools(99999)).not.toThrow();
  });

  it("closeDevtools(99999) 不崩溃", async () => {
    const { webViewAPI } = await import("../../generated/WebViewAPIClient");
    expect(() => webViewAPI.closeDevtools(99999)).not.toThrow();
  });
});

// ─── 托盘 API ─────────────────────────────────────────────────

describe("trayAPI (TrayAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { trayAPI } = await import("../../generated/TrayAPIClient");
    expect(trayAPI).toBeDefined();
    expect(typeof trayAPI.createTray).toBe("function");
  });

  it("getTrayBounds(99999) 不崩溃", async () => {
    const { trayAPI } = await import("../../generated/TrayAPIClient");
    const result = trayAPI.getTrayBounds(99999);
    expect(result).toBeDefined();
  });

  it("showTray(99999) / hideTray(99999) 不崩溃", async () => {
    const { trayAPI } = await import("../../generated/TrayAPIClient");
    expect(() => trayAPI.showTray(99999)).not.toThrow();
    expect(() => trayAPI.hideTray(99999)).not.toThrow();
  });

  it("destroyTray(99999) 不崩溃", async () => {
    const { trayAPI } = await import("../../generated/TrayAPIClient");
    expect(() => trayAPI.destroyTray(99999)).not.toThrow();
  });
});

// ─── 核心 API ─────────────────────────────────────────────────

describe("coreAPI (CoreAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { coreAPI } = await import("../../generated/CoreAPIClient");
    expect(coreAPI).toBeDefined();
  });

  it("getPlatform() 返回 windows/macos/linux", async () => {
    const { coreAPI } = await import("../../generated/CoreAPIClient");
    const platform = coreAPI.getPlatform();
    expect(typeof platform).toBe("string");
    expect(["windows", "macos", "linux"]).toContain(platform);
  });

  it("setExitOnLastWindowClosed(true) 不崩溃", async () => {
    const { coreAPI } = await import("../../generated/CoreAPIClient");
    expect(() => coreAPI.setExitOnLastWindowClosed(true)).not.toThrow();
  });

  it("stopEventLoop 函数存在", async () => {
    const { coreAPI } = await import("../../generated/CoreAPIClient");
    expect(typeof coreAPI.stopEventLoop).toBe("function");
  });
});

// ─── 剪贴板 API ───────────────────────────────────────────────

describe("clipboardAPI (ClipboardAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { clipboardAPI } = await import("../../generated/ClipboardAPIClient");
    expect(clipboardAPI).toBeDefined();
  });

  it("readText() 返回字符串", async () => {
    const { clipboardAPI } = await import("../../generated/ClipboardAPIClient");
    const text = clipboardAPI.readText();
    expect(typeof text === "string" || typeof text === "object").toBe(true);
  });

  it("writeText / clear 不崩溃", async () => {
    const { clipboardAPI } = await import("../../generated/ClipboardAPIClient");
    expect(() => clipboardAPI.writeText("eden-ipc-test")).not.toThrow();
    expect(() => clipboardAPI.clear()).not.toThrow();
  });
});

// ─── 对话框 API ───────────────────────────────────────────────

describe("dialogAPI (DialogAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { dialogAPI } = await import("../../generated/DialogAPIClient");
    expect(dialogAPI).toBeDefined();
  });

  it("showMessageBox 函数存在", async () => {
    const { dialogAPI } = await import("../../generated/DialogAPIClient");
    expect(typeof dialogAPI.showMessageBox).toBe("function");
  });

  it("moveToTrash('') 不崩溃", async () => {
    const { dialogAPI } = await import("../../generated/DialogAPIClient");
    // 空路径应该返回错误，但不崩溃
    expect(() => dialogAPI.moveToTrash("")).not.toThrow();
  });
});

// ─── Session API ───────────────────────────────────────────────

describe("sessionAPI (SessionAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { sessionAPI } = await import("../../generated/SessionAPIClient");
    expect(sessionAPI).toBeDefined();
  });

  it("clearCookies / clearStorageData 不崩溃", async () => {
    const { sessionAPI } = await import("../../generated/SessionAPIClient");
    expect(() => sessionAPI.clearCookies()).not.toThrow();
    expect(() => sessionAPI.clearStorageData()).not.toThrow();
  });
});

// ─── 快捷键 API ───────────────────────────────────────────────

describe("shortcutsAPI (ShortcutsAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { shortcutsAPI } = await import("../../generated/ShortcutsAPIClient");
    expect(shortcutsAPI).toBeDefined();
  });

  it("isRegistered('') 不崩溃", async () => {
    const { shortcutsAPI } = await import("../../generated/ShortcutsAPIClient");
    expect(() => shortcutsAPI.isRegistered("")).not.toThrow();
  });

  it("unregisterAll 不崩溃", async () => {
    const { shortcutsAPI } = await import("../../generated/ShortcutsAPIClient");
    expect(() => shortcutsAPI.unregisterAll()).not.toThrow();
  });
});

// ─── 显示 API ─────────────────────────────────────────────────

describe("displayAPI (DisplayAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { displayAPI } = await import("../../generated/DisplayAPIClient");
    expect(displayAPI).toBeDefined();
  });

  it("getAllDisplays() 返回数组或错误", async () => {
    const { displayAPI } = await import("../../generated/DisplayAPIClient");
    const result = displayAPI.getAllDisplays();
    expect(result).toBeDefined();
  });

  it("getPrimaryDisplay() 返回对象或错误", async () => {
    const { displayAPI } = await import("../../generated/DisplayAPIClient");
    const result = displayAPI.getPrimaryDisplay();
    expect(result).toBeDefined();
  });
});

// ─── App API ──────────────────────────────────────────────────

describe("appAPI (AppAPIClient)", () => {
  it("加载客户端不崩溃", async () => {
    const { appAPI } = await import("../../generated/AppAPIClient");
    expect(appAPI).toBeDefined();
  });

  it("getAppName() 返回字符串", async () => {
    const { appAPI } = await import("../../generated/AppAPIClient");
    const name = appAPI.getAppName();
    expect(typeof name).toBe("string");
  });

  it("getAppVersion() 返回字符串", async () => {
    const { appAPI } = await import("../../generated/AppAPIClient");
    const version = appAPI.getAppVersion();
    expect(typeof version).toBe("string");
  });

  it("getAppDataPath() 返回字符串", async () => {
    const { appAPI } = await import("../../generated/AppAPIClient");
    const path = appAPI.getAppDataPath();
    expect(typeof path).toBe("string");
  });
});
