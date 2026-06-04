// Electrobun FFI native bindings — entry point
// Imports and re-exports from split modules + initialization code.

import { CString, FFIType, JSCallback, ptr } from "bun:ffi";
import { createReadStream } from "node:fs";
import { core_, hasFFI, native_, toCString } from "./core-lib";

// ─── Re-exports from split modules ───
export {
  core_,
  getCoreLastError,
  hasFFI,
  native,
  native_,
  toCString,
} from "./core-lib";
export {
  type ApplicationMenuItemConfig,
  ffi as ffiImpl,
  type MenuItemConfig,
  type Rectangle,
} from "./ffi-impl";
export {
  type Cookie,
  type Display,
  type Point,
  Screen,
  Session,
  type StorageType,
} from "./platform";
export { WGPUBridge } from "./wgpu";

import { BrowserView } from "../core/BrowserView";
import { WGPUView } from "../core/WGPUView";
import electrobunEventEmitter from "../events/eventEmitter";

import { ffiImpl } from "./ffi-impl";
import type { CookieFilter } from "./platform";

// ─── PostMessage bridge for carrot workers ───
class PostMessageBridge {
  private requestId = 0;
  private pendingRequests = new Map<
    number,
    {
      resolve: (value: unknown) => void;
      reject: (error: Error) => void;
    }
  >();
  private eventHandlers = new Map<string, Set<(payload: unknown) => void>>();

  constructor() {
    if (
      typeof self !== "undefined" &&
      typeof self.addEventListener === "function"
    ) {
      self.addEventListener("message", (event: MessageEvent) => {
        this.handleMessage(event.data);
      });
    }
  }

  sendAction(action: string, payload?: unknown) {
    self.postMessage({ type: "action", action, payload });
  }

  requestHost<T = unknown>(method: string, params?: unknown): Promise<T> {
    const id = ++this.requestId;
    self.postMessage({ type: "host-request", requestId: id, method, params });
    return new Promise<T>((resolve, reject) => {
      this.pendingRequests.set(id, { resolve: (v) => resolve(v as T), reject });
    });
  }

  on(name: string, handler: (payload: unknown) => void) {
    const handlers = this.eventHandlers.get(name) ?? new Set();
    handlers.add(handler);
    this.eventHandlers.set(name, handlers);
    return () => {
      handlers.delete(handler);
      if (handlers.size === 0) {
        this.eventHandlers.delete(name);
      }
    };
  }

  emit(name: string, payload: unknown) {
    this.eventHandlers.get(name)?.forEach((h) => {
      try {
        h(payload);
      } catch (e) {
        console.error(`[bridge] event handler failed: ${name}`, e);
      }
    });
  }

  private handleMessage(message: any) {
    if (!message || typeof message !== "object" || !("type" in message)) {
      return;
    }
    if (message.type === "host-response") {
      const pending = this.pendingRequests.get(message.requestId);
      if (!pending) {
        return;
      }
      this.pendingRequests.delete(message.requestId);
      if (message.success) {
        pending.resolve(message.payload);
      } else {
        pending.reject(new Error(message.error || "Host request failed"));
      }
    } else if (message.type === "event") {
      this.emit(message.name, message.payload);
    } else if (message.type === "init") {
      this.emit("init", message);
    }
  }
}

const isCarrotWorker = !!(globalThis as any).__bunnyCarrotBootstrap;
export const bridge: PostMessageBridge | null = isCarrotWorker
  ? new PostMessageBridge()
  : null;

function createFfiRequestProxy(
  ffiRequest: Record<string, Function>
): Record<string, Function> {
  if (hasFFI) {
    return ffiRequest;
  }
  return new Proxy(ffiRequest, {
    get(target, method: string) {
      if (typeof method !== "string") {
        return target[method];
      }
      return (params?: unknown) => bridge!.requestHost(method, params);
    },
  });
}

// ─── Host message queue ───
const queuedHostMessageWebviewIdBuf = new Uint32Array(1);

const drainQueuedHostMessages = () => {
  if (!core_) {
    return;
  }
  for (;;) {
    const messagePtr = core_.symbols.electrobun_pop_next_queued_host_message(
      ptr(queuedHostMessageWebviewIdBuf)
    ) as bigint | null;
    if (!messagePtr) {
      return;
    }
    try {
      const rawMessage = new CString(messagePtr as any).toString();
      if (!rawMessage) {
        continue;
      }
      const webview = BrowserView.ensureWrapped(
        queuedHostMessageWebviewIdBuf[0]!
      );
      if (!webview) {
        continue;
      }
      webview.rpcHandler?.(JSON.parse(rawMessage));
    } catch (err) {
      console.error("error draining queued host message:", err);
    } finally {
      core_.symbols.electrobun_free_core_string(messagePtr as any);
    }
  }
};

if (core_) {
  const wakeupReadFd =
    core_.symbols.electrobun_get_host_message_wakeup_read_fd();
  if (typeof wakeupReadFd === "number" && wakeupReadFd >= 0) {
    try {
      const wakeupStream = createReadStream("/dev/null", {
        fd: wakeupReadFd,
        autoClose: false,
      });
      wakeupStream.on("data", () => drainQueuedHostMessages());
      wakeupStream.on("error", () => setInterval(drainQueuedHostMessages, 16));
    } catch {
      setInterval(drainQueuedHostMessages, 16);
    }
  } else {
    setInterval(drainQueuedHostMessages, 16);
  }
  drainQueuedHostMessages();
}

// ─── ffi proxy ───
export const ffi = {
  request: createFfiRequestProxy(
    ffiImpl.request as unknown as Record<string, Function>
  ) as typeof ffiImpl.request,
  internal: ffiImpl.internal,
};

// ─── Worker process handlers ───
process.on("uncaughtException", (err) => {
  console.error("Uncaught exception in worker:", err);
  if (native_) {
    native_.symbols.stopEventLoop();
    native_.symbols.waitForShutdownComplete(5000);
    native_.symbols.forceExit(1);
  } else {
    process.exit(1);
  }
});

process.on("unhandledRejection", (reason) => {
  console.error("Unhandled rejection in worker:", reason);
});

process.on("SIGINT", () => {
  console.log("[electrobun] Received SIGINT, running quit sequence...");
  const { quit } = require("../core/Utils");
  quit();
});

process.on("SIGTERM", () => {
  console.log("[electrobun] Received SIGTERM, running quit sequence...");
  const { quit } = require("../core/Utils");
  quit();
});

// ─── JSCallback setup ───
const _getMimeType = new JSCallback(
  (filePath: any) => {
    const _filePath = new CString(filePath).toString();
    const mimeType = Bun.file(_filePath).type;
    return toCString(mimeType.split(";")[0]!);
  },
  { args: [FFIType.cstring], returns: FFIType.cstring }
);

const _getHTMLForWebviewSync = new JSCallback(
  (webviewId: any) => {
    const webview = BrowserView.ensureWrapped(webviewId);
    return toCString(webview?.html || "");
  },
  { args: [FFIType.u32], returns: FFIType.cstring }
);

// TODO: 等 Rust 侧实现 setURLOpenHandler 后恢复 JSCallback 注册
// if (native_) native_.symbols.setJSUtils(getMimeType, getHTMLForWebviewSync);

// Native-only init
const globalShortcutHandlers = new Map<string, () => void>();

if (native_) {
  const urlOpenCallback = new JSCallback(
    (urlPtr: any) => {
      const url = new CString(urlPtr).toString();
      const handler = electrobunEventEmitter.events.app.openUrl;
      const event = handler({ url });
      electrobunEventEmitter.emitEvent(event);
    },
    { args: [FFIType.cstring], returns: "void", threadsafe: true }
  );
  if (process.platform === "darwin") {
    native_.symbols.setURLOpenHandler(urlOpenCallback);
  }

  const appReopenCallback = new JSCallback(
    () => {
      if (process.platform === "darwin") {
        core_.symbols.electrobun_set_dock_icon_visible(true);
      }
      const handler = electrobunEventEmitter.events.app.reopen;
      const event = handler({});
      electrobunEventEmitter.emitEvent(event);
    },
    { args: [], returns: "void", threadsafe: true }
  );
  if (process.platform === "darwin") {
    native_.symbols.setAppReopenHandler(appReopenCallback);
  }

  const quitRequestedCallback = new JSCallback(
    () => {
      const { quit } = require("../core/Utils");
      quit();
    },
    { args: [], returns: "void", threadsafe: true }
  );
  core_.symbols.electrobun_set_quit_requested_handler(quitRequestedCallback);

  const globalShortcutCallback = new JSCallback(
    (acceleratorPtr: any) => {
      const accelerator = new CString(acceleratorPtr).toString();
      const handler = globalShortcutHandlers.get(accelerator);
      if (handler) {
        handler();
      }
    },
    { args: [FFIType.cstring], returns: "void", threadsafe: true }
  );
  native_.symbols.setGlobalShortcutCallback(globalShortcutCallback);
}

// ─── GlobalShortcut module (uses local handlers) ───
export const GlobalShortcut = {
  register: (accelerator: string, callback: () => void): boolean => {
    if (!native_ || globalShortcutHandlers.has(accelerator)) {
      return false;
    }
    const result = native_.symbols.registerGlobalShortcut(
      toCString(accelerator)
    );
    if (result) {
      globalShortcutHandlers.set(accelerator, callback);
    }
    return result;
  },
  unregister: (accelerator: string): boolean => {
    if (!native_) {
      return false;
    }
    const result = native_.symbols.unregisterGlobalShortcut(
      toCString(accelerator)
    );
    if (result) {
      globalShortcutHandlers.delete(accelerator);
    }
    return result;
  },
  unregisterAll: (): void => {
    if (native_) {
      native_.symbols.unregisterAllGlobalShortcuts();
    }
    globalShortcutHandlers.clear();
  },
  isRegistered: (accelerator: string): boolean => {
    if (!native_) {
      return false;
    }
    return native_.symbols.isGlobalShortcutRegistered(toCString(accelerator));
  },
};

// ─── Types for internal RPC ───
type WebviewTagInitParams = {
  url: string | null;
  html: string | null;
  preload: string | null;
  renderer: "native";
  partition: string | null;
  frame: { x: number; y: number; width: number; height: number };
  hostWebviewId: number;
  windowId: number;
  navigationRules: string | null;
  sandbox: boolean;
  transparent: boolean;
  passthrough: boolean;
};

type WgpuTagInitParams = {
  windowId: number;
  frame: { x: number; y: number; width: number; height: number };
  transparent: boolean;
  passthrough: boolean;
};

// ─── Internal RPC handlers ───
export const internalRpcHandlers = {
  request: {
    webviewTagInit: (params: WebviewTagInitParams) => {
      const {
        hostWebviewId,
        windowId,
        renderer,
        html,
        preload,
        partition,
        frame,
        navigationRules,
        sandbox,
        transparent,
        passthrough,
      } = params;
      const url = params.url || html ? params.url : "https://electrobun.dev";
      const webviewForTag = new BrowserView({
        url,
        html,
        preload,
        partition,
        frame,
        hostWebviewId,
        autoResize: false,
        windowId,
        renderer,
        navigationRules,
        sandbox,
        startTransparent: transparent,
        startPassthrough: passthrough,
      });
      return webviewForTag.id;
    },
    wgpuTagInit: (params: WgpuTagInitParams) => {
      const { windowId, frame, transparent, passthrough } = params;
      const viewForTag = new WGPUView({
        windowId,
        frame,
        autoResize: false,
        startTransparent: transparent,
        startPassthrough: passthrough,
      });
      return viewForTag.id;
    },
    webviewTagCanGoBack: (params: { id: number }) =>
      core_.symbols.electrobun_webview_can_go_back(params.id),
    webviewTagCanGoForward: (params: { id: number }) =>
      core_.symbols.electrobun_webview_can_go_forward(params.id),
  },
  message: {
    webviewTagResize: (params: {
      id: number;
      frame: { width: number; height: number };
      masks: string;
    }) => {
      core_.symbols.electrobun_resize_webview(
        params.id,
        params.frame.width,
        params.frame.height
      );
    },
    wgpuTagResize: (params: {
      id: number;
      frame: { x: number; y: number; width: number; height: number };
      masks: string;
    }) => {
      const view = WGPUView.getById(params.id);
      if (!view?.ptr) {
        console.error(`wgpuTagResize: WGPUView not found for id ${params.id}`);
        return;
      }
      native_.symbols.resizeWebview(
        view.ptr,
        params.frame.x,
        params.frame.y,
        params.frame.width,
        params.frame.height,
        toCString(params.masks ?? "[]")
      );
    },
  },
};

export type { CookieFilter };
