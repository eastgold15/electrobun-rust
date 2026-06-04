/**
 * Electrobun 统一 FFI 客户端
 *
 * 汇集所有生成的 API Client，运行时通过此入口访问原生功能。
 * 替代旧的 proc/native 桥接层。
 */
export { AppClient, appAPI } from "./generated/AppAPIClient";
export { WindowClient, windowAPI } from "./generated/WindowAPIClient";
export { WebViewClient, webViewAPI } from "./generated/WebViewAPIClient";
export { DialogClient, dialogAPI } from "./generated/DialogAPIClient";
export { TrayClient, trayAPI } from "./generated/TrayAPIClient";
export { ClipboardClient, clipboardAPI } from "./generated/ClipboardAPIClient";
export { SessionClient, sessionAPI } from "./generated/SessionAPIClient";
export { ShortcutsClient, shortcutsAPI } from "./generated/ShortcutsAPIClient";
export { DisplayClient, displayAPI } from "./generated/DisplayAPIClient";
export { CoreClient, coreAPI } from "./generated/CoreAPIClient";
export { WgpuClient, wgpuAPI } from "./generated/WgpuAPIClient";
