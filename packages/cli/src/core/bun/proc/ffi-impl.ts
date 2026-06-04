import { type Pointer, ptr } from "bun:ffi";
import {
  core_,
  ensureWebviewRuntimeConfigured,
  getCoreLastError,
  toCString,
} from "./core-lib";
import { gen } from "./generated-bridge";

// Menu data reference system to avoid serialization overhead
const menuDataRegistry = new Map<string, any>();
let menuDataCounter = 0;
function storeMenuData(data: any): string {
  const id = `menuData_${++menuDataCounter}`;
  menuDataRegistry.set(id, data);
  return id;
}

function getMenuData(id: string): any {
  return menuDataRegistry.get(id);
}

function clearMenuData(id: string): void {
  menuDataRegistry.delete(id);
}

// Shared methods for EB delimiter serialization/deserialization
const ELECTROBUN_DELIMITER = "|EB|";

function serializeMenuAction(action: string, data: any): string {
  const dataId = storeMenuData(data);
  return `${ELECTROBUN_DELIMITER}${dataId}|${action}`;
}

function deserializeMenuAction(encodedAction: string): {
  action: string;
  data: any;
} {
  let actualAction = encodedAction;
  let data;

  if (encodedAction.startsWith(ELECTROBUN_DELIMITER)) {
    const parts = encodedAction.split("|");
    if (parts.length >= 4) {
      const dataId = parts[2]!;
      actualAction = parts.slice(3).join("|");
      data = getMenuData(dataId);
      clearMenuData(dataId);
    }
  }

  return { action: actualAction, data };
}

// ──────────────────────────────────────────────
// _ffiImpl — FFI request implementations
// ──────────────────────────────────────────────

export const ffiImpl = {
  request: {
    createWindow: (params: {
      url: string | null;
      title: string;
      frame: { width: number; height: number; x: number; y: number };
      styleMask: {
        Borderless: boolean;
        Titled: boolean;
        Closable: boolean;
        Miniaturizable: boolean;
        Resizable: boolean;
        UnifiedTitleAndToolbar: boolean;
        FullScreen: boolean;
        FullSizeContentView: boolean;
        UtilityWindow: boolean;
        DocModalWindow: boolean;
        NonactivatingPanel: boolean;
        HUDWindow: boolean;
      };
      titleBarStyle: string;
      transparent: boolean;
      hidden?: boolean;
      activate?: boolean;
      trafficLightOffset?: { x: number; y: number };
    }): number => {
      const {
        title,
        frame: { x, y, width, height },
        styleMask: {
          Borderless,
          Titled,
          Closable,
          Miniaturizable,
          Resizable,
          FullScreen,
        },
        transparent,
        hidden = false,
      } = params;
      let optionsBits = 0;
      if (transparent) {
        optionsBits |= 0x01;
      }
      if (hidden) {
        optionsBits |= 0x02;
      }
      if (Borderless || !Titled) {
        optionsBits |= 0x04;
      }
      if (Resizable) {
        optionsBits |= 0x08;
      }
      if (Closable) {
        optionsBits |= 0x10;
      }
      optionsBits |= 0x20;
      if (Miniaturizable) {
        optionsBits |= 0x40;
      }
      optionsBits |= 0x80;
      optionsBits |= 0x1_00;
      if (FullScreen) {
        optionsBits |= 0x8_00;
      }
      const windowId = core_.symbols.electrobun_create_window(
        toCString(title),
        x,
        y,
        width,
        height,
        optionsBits
      );
      if (!windowId) {
        throw getCoreLastError() || "Failed to create window";
      }
      return windowId;
    },

    getWindowPointer: (params: { winId: number }): Pointer | null =>
      params.winId as any,

    setTitle: (params: { winId: number; title: string }) => {
      gen.window.setTitle(params);
    },

    closeWindow: (params: { winId: number }) => {
      gen.window.closeWindow(params);
    },

    showWindow: (params: { winId: number; activate?: boolean }) => {
      gen.window.showWindow(params);
    },

    activateWindow: (params: { winId: number }) => {
      gen.window.showWindow({ winId: params.winId });
    },

    hideWindow: (params: { winId: number }) => {
      gen.window.hideWindow(params);
    },

    minimizeWindow: (params: { winId: number }) => {
      gen.window.minimize(params);
    },

    restoreWindow: (params: { winId: number }) => {
      gen.window.restore(params);
    },

    isWindowMinimized: (params: { winId: number }): boolean =>
      gen.window.isMinimized(params),

    maximizeWindow: (params: { winId: number }) => {
      gen.window.maximize(params);
    },

    unmaximizeWindow: (params: { winId: number }) => {
      gen.window.unmaximize(params);
    },

    isWindowMaximized: (params: { winId: number }): boolean =>
      gen.window.isMaximized(params),

    setWindowFullScreen: (params: { winId: number; fullScreen: boolean }) => {
      gen.window.setFullscreen({ winId: params.winId, fullscreen: params.fullScreen });
    },

    isWindowFullScreen: (params: { winId: number }): boolean =>
      gen.window.isFullscreen(params),

    setWindowAlwaysOnTop: (params: { winId: number; alwaysOnTop: boolean }) => {
      gen.window.setAlwaysOnTop({ winId: params.winId, onTop: params.alwaysOnTop });
    },

    isWindowAlwaysOnTop: (params: { winId: number }): boolean =>
      gen.window.isAlwaysOnTop(params),

    setWindowVisibleOnAllWorkspaces: (params: {
      winId: number;
      visibleOnAllWorkspaces: boolean;
    }) => {
      core_.symbols.electrobun_set_window_visible_on_all_workspaces(
        params.winId,
        params.visibleOnAllWorkspaces
      );
    },

    isWindowVisibleOnAllWorkspaces: (params: { winId: number }): boolean =>
      core_.symbols.electrobun_is_window_visible_on_all_workspaces(
        params.winId
      ),

    setWindowPosition: (params: { winId: number; x: number; y: number }) => {
      gen.window.setPosition(params);
    },

    setWindowButtonPosition: (params: {
      winId: number;
      x: number;
      y: number;
    }) => {
      core_.symbols.electrobun_set_window_button_position(
        params.winId,
        params.x,
        params.y
      );
    },

    setWindowSize: (params: {
      winId: number;
      width: number;
      height: number;
    }) => {
      gen.window.setSize(params);
    },

    setWindowFrame: (params: {
      winId: number;
      x?: number;
      y?: number;
      width?: number;
      height?: number;
    }) => {
      gen.window.setFrame({ winId: params.winId, frameless: false });
    },

    getWindowFrame: (params: {
      winId: number;
    }): { x: number; y: number; width: number; height: number } => {
      return gen.window.getBounds(params);
    },

    createWebview: (params: {
      windowId: number;
      secretKey: string;
      url: string | null;
      partition: string | null;
      sandbox: boolean;
      renderer?: string;
      hostWebviewId?: number;
      [key: string]: any;
    }): number => {
      const { windowId, secretKey, url, partition, sandbox } = params;
      ensureWebviewRuntimeConfigured();
      const useWry = true;
      const webviewId = core_.symbols.electrobun_create_webview(
        windowId,
        toCString(url || ""),
        toCString(secretKey),
        toCString(partition || "persist:default"),
        sandbox,
        useWry
      );
      if (!webviewId) {
        throw getCoreLastError() || "Failed to create webview";
      }
      return webviewId;
    },

    getWebviewPointer: (params: { id: number }): Pointer | null =>
      core_.symbols.electrobun_get_webview_pointer(params.id) || null,

    resizeWebview: (params: {
      id: number;
      frame: { x: number; y: number; width: number; height: number };
      masks?: string;
    }) => {
      core_.symbols.electrobun_resize_webview(
        params.id,
        params.frame.width,
        params.frame.height
      );
    },

    loadURLInWebView: (params: { id: number; url: string }) => {
      core_.symbols.electrobun_load_url_in_webview(
        params.id,
        toCString(params.url)
      );
    },

    loadHTMLInWebView: (params: { id: number; html: string }) => {
      core_.symbols.electrobun_load_html_in_webview(
        params.id,
        toCString(params.html)
      );
    },

    updatePreloadScriptToWebView: (params: {
      id: number;
      scriptIdentifier: string;
      script: string;
      allFrames: boolean;
    }) => {
      core_.symbols.electrobun_update_preload_script_to_webview(
        params.id,
        toCString(params.scriptIdentifier)
      );
    },

    webviewCanGoBack: (params: { id: number }) =>
      core_.symbols.electrobun_webview_can_go_back(params.id),
    webviewCanGoForward: (params: { id: number }) =>
      core_.symbols.electrobun_webview_can_go_forward(params.id),
    webviewGoBack: (params: { id: number }) =>
      core_.symbols.electrobun_webview_go_back(params.id),
    webviewGoForward: (params: { id: number }) =>
      core_.symbols.electrobun_webview_go_forward(params.id),
    webviewReload: (params: { id: number }) =>
      core_.symbols.electrobun_webview_reload(params.id),
    webviewRemove: (params: { id: number }) =>
      core_.symbols.electrobun_webview_remove(params.id),

    setWebviewHTMLContent: (params: { id: number; html: string }) => {
      core_.symbols.electrobun_set_webview_html_content(
        params.id,
        toCString(params.html)
      );
    },

    webviewSetTransparent: (params: { id: number; transparent: boolean }) => {
      core_.symbols.electrobun_webview_set_transparent(
        params.id,
        params.transparent
      );
    },

    webviewSetPassthrough: (params: { id: number; passthrough: boolean }) => {
      core_.symbols.electrobun_webview_set_passthrough(
        params.id,
        params.passthrough
      );
    },

    webviewSetHidden: (params: { id: number; hidden: boolean }) => {
      core_.symbols.electrobun_webview_set_hidden(params.id, params.hidden);
    },

    setWebviewNavigationRules: (params: { id: number; rulesJson: string }) => {
      core_.symbols.electrobun_set_webview_navigation_rules(
        params.id,
        toCString(params.rulesJson)
      );
    },

    webviewFindInPage: (params: {
      id: number;
      searchText: string;
      forward: boolean;
      matchCase: boolean;
    }) => {
      core_.symbols.electrobun_webview_find_in_page(
        params.id,
        toCString(params.searchText),
        params.forward,
        params.matchCase
      );
    },

    webviewStopFind: (params: { id: number; stopAll: boolean }) =>
      core_.symbols.electrobun_webview_stop_find(params.id, params.stopAll),

    createWGPUView: (params: {
      windowId: number;
      frame: { x: number; y: number; width: number; height: number };
      autoResize: boolean;
      startTransparent: boolean;
      startPassthrough: boolean;
    }): number => {
      const {
        windowId,
        frame: { x, y, width, height },
        autoResize,
        startTransparent,
        startPassthrough,
      } = params;
      const viewId = core_.symbols.electrobun_create_wgpu_view(
        windowId,
        x,
        y,
        width,
        height,
        autoResize,
        startTransparent,
        startPassthrough
      );
      if (!viewId) {
        throw "Failed to create WGPUView";
      }
      return viewId;
    },

    getWGPUViewPointer: (params: { id: number }): Pointer | null =>
      core_.symbols.electrobun_get_wgpu_view_pointer(params.id) || null,

    wgpuViewSetFrame: (params: {
      id: number;
      x: number;
      y: number;
      width: number;
      height: number;
    }) => {
      core_.symbols.electrobun_set_wgpu_view_frame(
        params.id,
        params.x,
        params.y,
        params.width,
        params.height
      );
    },

    wgpuViewSetTransparent: (params: { id: number; transparent: boolean }) => {
      core_.symbols.electrobun_set_wgpu_view_transparent(
        params.id,
        params.transparent
      );
    },

    wgpuViewSetPassthrough: (params: { id: number; passthrough: boolean }) => {
      core_.symbols.electrobun_set_wgpu_view_passthrough(
        params.id,
        params.passthrough
      );
    },

    wgpuViewSetHidden: (params: { id: number; hidden: boolean }) => {
      core_.symbols.electrobun_set_wgpu_view_hidden(params.id, params.hidden);
    },

    wgpuViewRemove: (params: { id: number }) =>
      core_.symbols.electrobun_remove_wgpu_view(params.id),
    wgpuViewGetNativeHandle: (params: { id: number }): BigInt =>
      core_.symbols.electrobun_get_wgpu_view_native_handle(params.id),
    runWGPUViewTest: (params: { id: number }) =>
      core_.symbols.electrobun_run_wgpu_view_test(params.id),

    evaluateJavascriptWithNoCompletion: (params: {
      id: number;
      js: string;
    }) => {
      core_.symbols.electrobun_evaluate_javascript_with_no_completion(
        params.id,
        toCString(params.js)
      );
    },

    sendHostMessageToWebviewViaTransport: (params: {
      id: number;
      messageJson: string;
    }): boolean =>
      core_.symbols.electrobun_send_host_message_to_webview_via_transport(
        params.id,
        toCString(params.messageJson)
      ),

    clearWebviewHostTransport: (params: { id: number }) =>
      core_.symbols.electrobun_clear_webview_host_transport(params.id),

    webviewOpenDevTools: (params: { id: number }) =>
      core_.symbols.electrobun_webview_open_devtools(params.id),
    webviewCloseDevTools: (params: { id: number }) =>
      core_.symbols.electrobun_webview_close_devtools(params.id),
    webviewToggleDevTools: (params: { id: number }) =>
      core_.symbols.electrobun_webview_toggle_devtools(params.id),

    webviewSetPageZoom: (params: { id: number; zoomLevel: number }) => {
      core_.symbols.electrobun_webview_set_page_zoom(
        params.id,
        params.zoomLevel
      );
    },

    webviewGetPageZoom: (params: { id: number }): number =>
      core_.symbols.electrobun_webview_get_page_zoom(params.id),

    setExitOnLastWindowClosed: (params: { enabled: boolean }) => {
      core_.symbols.electrobun_set_exit_on_last_window_closed(params.enabled);
    },

    quitGracefully: (_params?: { code?: number; timeoutMs?: number }) =>
      core_.symbols.electrobun_quit_gracefully(),

    // Tray
    createTray: (params: {
      title: string;
      image: string;
      template?: boolean;
      width?: number;
      height?: number;
    }): number => {
      return gen.tray.createTray(params);
    },
    showTray: (params: { id: number }): boolean => {
      gen.tray.show(params);
      return true;
    },
    hideTray: (params: { id: number }): void => {
      gen.tray.hide(params);
    },
    setTrayTitle: (params: { id: number; title: string }): void => {
      gen.tray.setTitle(params);
    },
    setTrayImage: (params: { id: number; image: string }): void => {
      gen.tray.setImage(params);
    },
    setTrayMenu: (params: { id: number; menuConfig: string }): void => {
      // 宏生成的 TrayAPI 没有 setMenu，保持手写
      core_.symbols.electrobun_set_tray_menu(
        params.id,
        toCString(params.menuConfig)
      );
    },
    removeTray: (params: { id: number }): void => {
      gen.tray.remove(params);
    },
    getTrayBounds: (params: { id: number }): Rectangle => {
      const buf = new BigInt64Array(4);
      core_.symbols.electrobun_get_tray_bounds(
        params.id,
        ptr(buf),
        ptr(buf, 8),
        ptr(buf, 16),
        ptr(buf, 24)
      );
      return {
        x: Number(buf[0]),
        y: Number(buf[1]),
        width: Number(buf[2]),
        height: Number(buf[3]),
      };
    },

    // Menu / Dialogs
    setApplicationMenu: (params: { menuConfig: string }): void => {
      core_.symbols.electrobun_set_application_menu(
        toCString(params.menuConfig)
      );
    },
    showContextMenu: (params: { menuConfig: string }): void => {
      core_.symbols.electrobun_show_context_menu(toCString(params.menuConfig));
    },
    moveToTrash: (params: { path: string }): boolean =>
      core_.symbols.electrobun_move_to_trash(toCString(params.path)),
    showItemInFolder: (params: { path: string }): void => {
      core_.symbols.electrobun_show_item_in_folder(toCString(params.path));
    },
    openExternal: (params: { url: string }): boolean =>
      core_.symbols.electrobun_open_external(toCString(params.url)),
    openPath: (params: { path: string }): boolean =>
      core_.symbols.electrobun_open_path(toCString(params.path)),

    showNotification: (params: {
      title: string;
      body?: string;
      subtitle?: string;
      silent?: boolean;
    }): void => {
      core_.symbols.electrobun_show_notification(
        toCString(params.title),
        toCString(params.body || ""),
        toCString(params.subtitle || "")
      );
    },

    setDockIconVisible: (params: { visible: boolean }): void => {
      core_.symbols.electrobun_set_dock_icon_visible(params.visible);
    },
    isDockIconVisible: (): boolean =>
      core_.symbols.electrobun_is_dock_icon_visible(),

    openFileDialog: (params: {
      startingFolder: string;
      allowedFileTypes: string;
      allowsMultipleSelection: boolean;
      canChooseFiles?: boolean;
      canChooseDirectory?: boolean;
    }): string => {
      const filePath = core_.symbols.electrobun_open_file_dialog(
        toCString("Open File"),
        toCString(params.startingFolder),
        toCString(params.allowedFileTypes),
        params.allowsMultipleSelection
      );
      return filePath.toString();
    },

    showMessageBox: (params: {
      title: string;
      message: string;
      kind?: string;
      type?: string;
      buttons?: string[];
    }): number => {
      const kindMap: Record<string, number> = {
        info: 0,
        warning: 1,
        error: 2,
        question: 3,
      };
      return core_.symbols.electrobun_show_message_box(
        toCString(params.title),
        toCString(params.message),
        kindMap[params.kind ?? "info"] ?? 0
      );
    },

    // Clipboard
    clipboardReadText: (): string | null => {
      const r = core_.symbols.electrobun_clipboard_read_text();
      return r ? r.toString() : null;
    },
    clipboardWriteText: (params: { text: string }): void => {
      core_.symbols.electrobun_clipboard_write_text(toCString(params.text));
    },
    clipboardReadImage: (): Uint8Array | null => {
      const dataPtr = core_.symbols.electrobun_clipboard_read_image();
      if (!dataPtr) {
        return null;
      }
      const result = new Uint8Array(0);
      return result;
    },
    clipboardWriteImage: (params: { pngData: Uint8Array }): void => {
      core_.symbols.electrobun_clipboard_write_image(ptr(params.pngData));
    },
    clipboardClear: (): void => {
      core_.symbols.electrobun_clipboard_clear();
    },
    clipboardAvailableFormats: (): string[] => {
      const r = core_.symbols.electrobun_clipboard_available_formats();
      if (!r) {
        return [];
      }
      return r
        .toString()
        .split(",")
        .filter((f) => f.length > 0);
    },
  },
  // Internal functions for menu data management
  internal: {
    storeMenuData,
    getMenuData,
    clearMenuData,
    serializeMenuAction,
    deserializeMenuAction,
  },
};

export interface Rectangle {
  height: number;
  width: number;
  x: number;
  y: number;
}

export type MenuItemConfig =
  | { type: "divider" | "separator" }
  | {
      type: "normal";
      label: string;
      tooltip?: string;
      action?: string;
      data?: any;
      submenu?: Array<MenuItemConfig>;
      enabled?: boolean;
      checked?: boolean;
      hidden?: boolean;
    };

export type ApplicationMenuItemConfig =
  | { type: "divider" | "separator" }
  | {
      type?: "normal";
      label: string;
      tooltip?: string;
      action?: string;
      data?: any;
      submenu?: Array<ApplicationMenuItemConfig>;
      enabled?: boolean;
      checked?: boolean;
      hidden?: boolean;
      accelerator?: string;
    }
  | {
      type?: "normal";
      label?: string;
      tooltip?: string;
      role?: string;
      data?: any;
      submenu?: Array<ApplicationMenuItemConfig>;
      enabled?: boolean;
      checked?: boolean;
      hidden?: boolean;
      accelerator?: string;
    };

export const ffi = {
  request: ffiImpl.request,
  internal: ffiImpl.internal,
};
