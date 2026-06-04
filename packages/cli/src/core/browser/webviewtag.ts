import "./global.d.ts";

type WebviewEventTypes =
  | "did-navigate"
  | "did-navigate-in-page"
  | "did-commit-navigation"
  | "dom-ready"
  | "host-message"
  | "new-window-open";

/**
 * Interface representing an <electrobun-webview> custom element.
 * Use this to properly type webview elements obtained via querySelector.
 *
 * @example
 * const webview = document.querySelector('electrobun-webview') as WebviewTagElement;
 * webview.loadURL('https://example.com');
 * webview.toggleHidden(false);
 */
interface WebviewTagElement extends HTMLElement {
  // Mask management
  addMaskSelector(selector: string): void;

  // Navigation
  canGoBack(): Promise<boolean>;
  canGoForward(): Promise<boolean>;
  closeDevTools(): void;
  emit(event: WebviewEventTypes, detail: unknown): void;

  // JavaScript execution
  executeJavascript(js: string): void;

  // Find in page
  findInPage(
    searchText: string,
    options?: { forward?: boolean; matchCase?: boolean }
  ): void;
  goBack(): void;
  goForward(): void;
  hidden: boolean;
  hiddenMirrorMode: boolean;
  html: string | null;
  loadHTML(html: string): void;
  loadURL(url: string): void;
  maskSelectors: Set<string>;
  off(event: WebviewEventTypes, listener: (event: CustomEvent) => void): void;

  // Events - listener receives a CustomEvent with detail property
  on(event: WebviewEventTypes, listener: (event: CustomEvent) => void): void;

  // Developer tools
  openDevTools(): void;
  partition: string | null;
  passthroughEnabled: boolean;
  preload: string | null;
  reload(): void;
  removeMaskSelector(selector: string): void;
  renderer: "native";

  // Navigation rules
  setNavigationRules(rules: string[]): void;

  // Attribute-backed properties (getters/setters)
  src: string | null;
  stopFindInPage(): void;

  // Dimension sync
  syncDimensions(force?: boolean): void;
  toggleDevTools(): void;
  toggleHidden(hidden?: boolean, bypassState?: boolean): void;
  togglePassthrough(enablePassthrough?: boolean, bypassState?: boolean): void;

  // Visibility and interaction
  toggleTransparent(transparent?: boolean, bypassState?: boolean): void;
  transparent: boolean;
  // Properties
  webviewId?: number;
}

// Augment global types so querySelector('electrobun-webview') returns WebviewTagElement
declare global {
  interface HTMLElementTagNameMap {
    "electrobun-webview": WebviewTagElement;
  }
}

export type { WebviewEventTypes, WebviewTagElement };
