/**
 * 宏生成 FFI 客户端适配器
 *
 * 将 hand-written ffi-impl.ts 的调用转发到 #[eden_ipc] 宏生成的 Client 类。
 * 逐步替换手写的 core_.symbols.electrobun_xxx 调用。
 *
 * 使用方法：在 ffi-impl.ts 中 import { gen } from "./generated-bridge"
 * 然后替换 gen.window.createWindow(params) 等调用。
 */

import { windowAPI } from "../../generated/WindowAPIClient";
import { appAPI } from "../../generated/AppAPIClient";
import { trayAPI } from "../../generated/TrayAPIClient";
import { webViewAPI } from "../../generated/WebViewAPIClient";
import { dialogAPI } from "../../generated/DialogAPIClient";
import { clipboardAPI } from "../../generated/ClipboardAPIClient";
import { displayAPI } from "../../generated/DisplayAPIClient";
import { sessionAPI } from "../../generated/SessionAPIClient";
import { shortcutsAPI } from "../../generated/ShortcutsAPIClient";
import { coreAPI } from "../../generated/CoreAPIClient";
import { wgpuAPI } from "../../generated/WgpuAPIClient";

// ─── Window ───────────────────────────────────────────────────

function windowCreateWindow(params: {
  url: string | null;
  title: string;
  frame: { width: number; height: number; x: number; y: number };
  styleMask: Record<string, boolean>;
  titleBarStyle: string;
  transparent: boolean;
  hidden?: boolean;
  activate?: boolean;
  trafficLightOffset?: { x: number; y: number };
}): number {
  const { title, frame, transparent, hidden } = params;
  const result = windowAPI.createWindow({
    width: frame.width,
    height: frame.height,
    title,
    transparent: transparent ?? false,
    frameless: false,
    url: params.url ?? undefined,
    html: undefined,
    icon: undefined,
  });
  if (result && typeof result === "object" && "type" in result) {
    throw new Error(`Window creation failed: ${(result as any).type}`);
  }
  return (result as any).id ?? 0;
}

function windowCloseWindow(params: { winId: number }): boolean {
  const result = windowAPI.closeWindow(params.winId);
  return !result || !("message" in result);
}

function windowSetTitle(params: { winId: number; title: string }): void {
  windowAPI.setWindowTitle(params.winId, params.title);
}

function windowShowWindow(params: { winId: number; activate?: boolean }): void {
  windowAPI.showWindow(params.winId);
}

function windowHideWindow(params: { winId: number }): void {
  windowAPI.hideWindow(params.winId);
}

function windowMinimize(params: { winId: number }): void {
  windowAPI.minimizeWindow(params.winId);
}

function windowMaximize(params: { winId: number }): void {
  windowAPI.maximizeWindow(params.winId);
}

function windowRestore(params: { winId: number }): void {
  windowAPI.restoreWindow(params.winId);
}

function windowSetSize(params: {
  winId: number;
  width: number;
  height: number;
}): void {
  windowAPI.setWindowSize(params.winId, params.width, params.height);
}

function windowSetPosition(params: {
  winId: number;
  x: number;
  y: number;
}): void {
  windowAPI.setWindowPosition(params.winId, params.x, params.y);
}

function windowSetFullscreen(params: {
  winId: number;
  fullscreen: boolean;
}): void {
  windowAPI.setWindowFullscreen(params.winId, params.fullscreen);
}

function windowSetAlwaysOnTop(params: {
  winId: number;
  onTop: boolean;
}): void {
  windowAPI.setWindowAlwaysOnTop(params.winId, params.onTop);
}

function windowGetBounds(params: {
  winId: number;
}): { x: number; y: number; width: number; height: number } {
  const result = windowAPI.getWindowBounds(params.winId);
  if (result && typeof result === "object" && "type" in result) {
    throw new Error(`getWindowBounds failed: ${(result as any).type}`);
  }
  return result as unknown as { x: number; y: number; width: number; height: number };
}

function windowSetFrame(params: { winId: number; frameless: boolean }): void {
  windowAPI.setWindowFrame(params.winId, params.frameless);
}

// ─── Tray ──────────────────────────────────────────────────────

function trayCreateTray(params: {
  title: string;
  image: string;
  template?: boolean;
  width?: number;
  height?: number;
}): number {
  const result = trayAPI.createTray({
    icon: params.image,
    tooltip: params.title,
  });
  if (typeof result !== "number") {
    throw new Error("Tray creation failed");
  }
  return result;
}

function traySetTitle(params: { id: number; title: string }): void {
  trayAPI.setTrayTitle(params.id, params.title);
}

function traySetImage(params: { id: number; image: string }): void {
  trayAPI.setTrayImage(params.id, params.image);
}

function trayShow(params: { id: number }): void {
  trayAPI.showTray(params.id);
}

function trayHide(params: { id: number }): void {
  trayAPI.hideTray(params.id);
}

function trayRemove(params: { id: number }): void {
  trayAPI.destroyTray(params.id);
}

// ─── 统一导出 ───────────────────────────────────────────────────

/**
 * 宏生成 API 的适配器对象
 * 用法：gen.window.createWindow({...})
 */
export const gen = {
  /** @eden_ipc WindowAPI */
  window: {
    createWindow: windowCreateWindow,
    closeWindow: windowCloseWindow,
    setTitle: windowSetTitle,
    showWindow: windowShowWindow,
    hideWindow: windowHideWindow,
    minimize: windowMinimize,
    maximize: windowMaximize,
    restore: windowRestore,
    setSize: windowSetSize,
    setPosition: windowSetPosition,
    setFullscreen: windowSetFullscreen,
    setAlwaysOnTop: windowSetAlwaysOnTop,
    getBounds: windowGetBounds,
    setFrame: windowSetFrame,
  },

  /** @eden_ipc TrayAPI */
  tray: {
    createTray: trayCreateTray,
    setTitle: traySetTitle,
    setImage: traySetImage,
    show: trayShow,
    hide: trayHide,
    remove: trayRemove,
  },

  /** 直接暴露原始 Client 实例（用于特殊场景） */
  raw: {
    window: windowAPI,
    app: appAPI,
    tray: trayAPI,
    webview: webViewAPI,
    dialog: dialogAPI,
    clipboard: clipboardAPI,
    display: displayAPI,
    session: sessionAPI,
    shortcuts: shortcutsAPI,
    core: coreAPI,
    wgpu: wgpuAPI,
  },
};
