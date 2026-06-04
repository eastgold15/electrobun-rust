// Global type declarations for Electrobun browser environment

interface ElectrobunEncryptResult {
  encryptedData: string;
  iv: string;
  tag: string;
}

interface ElectrobunBridge {
  receiveInternalMessageFromBun: (msg: unknown) => void;
  receiveInternalMessageFromHost: (msg: unknown) => void;
  receiveMessageFromBun: (msg: unknown) => void;
  receiveMessageFromHost: (msg: unknown) => void;
}

interface MessageHandler {
  postMessage: (msg: string) => void;
}

declare global {
  interface Window {
    __electrobun?: ElectrobunBridge;
    __electrobun_decrypt: (
      encryptedData: string,
      iv: string,
      tag: string
    ) => Promise<string>;
    __electrobun_encrypt: (msg: string) => Promise<ElectrobunEncryptResult>;
    __electrobunBunBridge?: MessageHandler;
    __electrobunHostBridge?: MessageHandler;
    __electrobunHostSocketPort?: number;
    __electrobunInternalBridge?: MessageHandler;
    __electrobunPendingHostMessages?: unknown[];
    __electrobunRpcSocketPort: number;
    __electrobunWebviewId: number;
    __electrobunWindowId: number;
  }
}

export {};
