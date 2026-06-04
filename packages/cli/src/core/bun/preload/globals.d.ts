// Type declarations for Electrobun preload globals
// These are set dynamically per-webview before the preload script runs

declare global {
  interface Window {
    __electrobun: {
      receiveMessageFromHost: (msg: unknown) => void;
      receiveInternalMessageFromHost: (msg: unknown) => void;
      receiveMessageFromBun: (msg: unknown) => void;
      receiveInternalMessageFromBun: (msg: unknown) => void;
    };
    __electrobun_decrypt: (
      encryptedData: string,
      iv: string,
      tag: string
    ) => Promise<string>;
    __electrobun_encrypt: (
      plaintext: string
    ) => Promise<{ encryptedData: string; iv: string; tag: string }>;
    __electrobunBunBridge?: {
      postMessage: (message: string) => void;
    };
    // Event-only bridge (all webviews, including sandboxed)
    __electrobunEventBridge?: {
      postMessage: (message: string) => void;
    };
    // User RPC bridge (trusted webviews only)
    __electrobunHostBridge?: {
      postMessage: (message: string) => void;
    };
    __electrobunHostSocketPort?: number;
    // Internal RPC bridge (trusted webviews only)
    __electrobunInternalBridge?: {
      postMessage: (message: string) => void;
    };
    __electrobunPendingHostMessages?: unknown[];
    __electrobunRpcSocketPort: number;
    __electrobunSecretKeyBytes: number[];
    __electrobunSendToHost: (message: unknown) => void;
    __electrobunWebviewId: number;
    __electrobunWindowId: number;
  }
}

export {};
