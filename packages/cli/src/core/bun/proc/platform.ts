import { ptr } from "bun:ffi";
import { core_, hasFFI, native_, toCString } from "./core-lib";
// ─── GlobalShortcut ───
export const GlobalShortcut = {
  register: (accelerator: string, callback: () => void): boolean => {
    if (
      !native_ ||
      (globalThis as any).__globalShortcutHandlers?.has(accelerator)
    ) {
      return false;
    }
    const result = native_.symbols.registerGlobalShortcut(
      toCString(accelerator)
    );
    if (result) {
      if (!(globalThis as any).__globalShortcutHandlers) {
        (globalThis as any).__globalShortcutHandlers = new Map<
          string,
          () => void
        >();
      }
      (globalThis as any).__globalShortcutHandlers.set(accelerator, callback);
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
      (globalThis as any).__globalShortcutHandlers?.delete(accelerator);
    }
    return result;
  },
  unregisterAll: (): void => {
    if (native_) {
      native_.symbols.unregisterAllGlobalShortcuts();
    }
    (globalThis as any).__globalShortcutHandlers?.clear();
  },
  isRegistered: (accelerator: string): boolean => {
    if (!native_) {
      return false;
    }
    return native_.symbols.isGlobalShortcutRegistered(toCString(accelerator));
  },
};

// ─── Rectangle & Display types ───
export interface Rectangle {
  height: number;
  width: number;
  x: number;
  y: number;
}
export interface Display {
  bounds: Rectangle;
  id: number;
  isPrimary: boolean;
  scaleFactor: number;
  workArea: Rectangle;
}
export interface Point {
  x: number;
  y: number;
}

// ─── Screen ───
export const Screen = {
  getPrimaryDisplay: (): Display => {
    const jsonStr = hasFFI
      ? core_.symbols.electrobun_get_primary_display()
      : null;
    if (!jsonStr) {
      return {
        id: 0,
        bounds: { x: 0, y: 0, width: 0, height: 0 },
        workArea: { x: 0, y: 0, width: 0, height: 0 },
        scaleFactor: 1,
        isPrimary: true,
      };
    }
    try {
      return JSON.parse(jsonStr.toString());
    } catch {
      return {
        id: 0,
        bounds: { x: 0, y: 0, width: 0, height: 0 },
        workArea: { x: 0, y: 0, width: 0, height: 0 },
        scaleFactor: 1,
        isPrimary: true,
      };
    }
  },
  getAllDisplays: (): Display[] => {
    const jsonStr = hasFFI ? core_.symbols.electrobun_get_all_displays() : null;
    if (!jsonStr) {
      return [];
    }
    try {
      return JSON.parse(jsonStr.toString());
    } catch {
      return [];
    }
  },
  getCursorScreenPoint: (): Point => {
    const buf = new BigInt64Array(2);
    const ok =
      hasFFI &&
      core_.symbols.electrobun_get_cursor_screen_point(ptr(buf), ptr(buf, 8));
    if (!ok) {
      return { x: 0, y: 0 };
    }
    return { x: Number(buf[0]), y: Number(buf[1]) };
  },
  getMouseButtons: (): bigint => {
    try {
      return hasFFI ? BigInt(core_.symbols.electrobun_get_mouse_buttons()) : 0n;
    } catch {
      return 0n;
    }
  },
};

// ─── Cookie / Session ───
export interface Cookie {
  domain?: string;
  expirationDate?: number;
  httpOnly?: boolean;
  name: string;
  path?: string;
  sameSite?: "no_restriction" | "lax" | "strict";
  secure?: boolean;
  value: string;
}
export interface CookieFilter {
  domain?: string;
  name?: string;
  path?: string;
  secure?: boolean;
  session?: boolean;
  url?: string;
}
export type StorageType =
  | "cookies"
  | "localStorage"
  | "sessionStorage"
  | "indexedDB"
  | "webSQL"
  | "cache"
  | "all";

class SessionCookies {
  private partitionId: string;
  constructor(partitionId: string) {
    this.partitionId = partitionId;
  }
  get(filter?: CookieFilter): Cookie[] {
    const filterJson = JSON.stringify(filter || {});
    const result = native_.symbols.sessionGetCookies(
      toCString(this.partitionId),
      toCString(filterJson)
    );
    if (!result) {
      return [];
    }
    try {
      return JSON.parse(result.toString());
    } catch {
      return [];
    }
  }
  set(cookie: Cookie): boolean {
    return native_.symbols.sessionSetCookie(
      toCString(this.partitionId),
      toCString(JSON.stringify(cookie))
    );
  }
  remove(url: string, name: string): boolean {
    return native_.symbols.sessionRemoveCookie(
      toCString(this.partitionId),
      toCString(url),
      toCString(name)
    );
  }
  clear(): void {
    native_.symbols.sessionClearCookies(toCString(this.partitionId));
  }
}

class SessionInstance {
  readonly partition: string;
  readonly cookies: SessionCookies;
  constructor(partition: string) {
    this.partition = partition;
    this.cookies = new SessionCookies(partition);
  }
  clearStorageData(types: StorageType[] | "all" = "all"): void {
    const typesArray = types === "all" ? ["all"] : types;
    native_.symbols.sessionClearStorageData(
      toCString(this.partition),
      toCString(JSON.stringify(typesArray))
    );
  }
}

const sessionCache = new Map<string, SessionInstance>();

export const Session = {
  fromPartition: (partition: string): SessionInstance => {
    let session = sessionCache.get(partition);
    if (!session) {
      session = new SessionInstance(partition);
      sessionCache.set(partition, session);
    }
    return session;
  },
  get defaultSession(): SessionInstance {
    return Session.fromPartition("persist:default");
  },
};
