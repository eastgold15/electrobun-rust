/**
 * WebView API 类型定义
 *
 * 定义 WebView 控制、导航、执行 JavaScript 相关的所有类型
 */

/**
 * WebView 导航策略
 */
export type NavigationPolicy =
  | "allow" // 允许导航
  | "deny" // 拒绝导航
  | "cancel"; // 取消当前导航

/**
 * 导航动作类型
 */
export type NavigationAction =
  | { type: "linkClicked"; url: string; isMainFrame: boolean }
  | { type: "formSubmitted"; url: string; method: string; isMainFrame: boolean }
  | { type: "backForward"; url: string; isMainFrame: boolean }
  | { type: "reload"; url: string; isMainFrame: boolean }
  | { type: "formResubmitted"; url: string; isMainFrame: boolean }
  | { type: "other"; url: string; isMainFrame: boolean };

/**
 * 导航决策结果
 */
export type NavigationDecision =
  | { action: "allow" }
  | { action: "cancel" }
  | { action: "redirect"; url: string };

/**
 * WebView 加载状态
 */
export type WebViewLoadState =
  | { type: "started"; url: string }
  | { type: "progress"; progress: number; url: string }
  | { type: "committed"; url: string; title?: string }
  | { type: "finished"; url: string; title?: string }
  | { type: "failed"; url: string; error: string; errorCode: number };

/**
 * HTTP 请求头
 */
export interface HttpHeaders {
  [key: string]: string;
}

/**
 * HTTP 响应信息
 */
export interface HttpResponseInfo {
  /** 编码 */
  charset?: string;

  /** 响应头 */
  headers: HttpHeaders;

  /** MIME 类型 */
  mimeType?: string;
  /** 状态码 */
  statusCode: number;

  /** 状态文本 */
  statusText: string;
}

/**
 * WebView 创建参数
 */
export interface WebViewParams {
  /** 是否启用上下文菜单（默认：true） */
  contextMenu?: boolean;

  /** 是否启用开发者工具（默认：false） */
  devtools?: boolean;

  /** 自定义请求头 */
  headers?: HttpHeaders;

  /** 初始 HTML 内容 */
  html?: string;

  /** 初始 Cookies */
  initialCookies?: Cookie[];

  /** 是否启用 JavaScript（默认：true） */
  javascript?: boolean;

  /** 是否透明背景（默认：false） */
  transparent?: boolean;

  /** 初始 URL */
  url?: string;

  /** 用户代理字符串 */
  userAgent?: string;

  /** 是否启用 WebGL（默认：true） */
  webgl?: boolean;
  /** 父窗口 ID */
  windowId: number;

  /** 是否启用缩放（默认：true） */
  zoom?: boolean;

  /** 初始缩放比例（默认：1.0） */
  zoomFactor?: number;
}

/**
 * WebView 对象
 */
export interface WebView {
  /** 是否可以后退 */
  canGoBack: boolean;

  /** 是否可以前进 */
  canGoForward: boolean;
  /** WebView ID */
  id: number;

  /** 是否正在加载 */
  isLoading: boolean;

  /** 加载进度（0-100） */
  loadProgress: number;

  /** 当前标题 */
  title: string;

  /** 当前 URL */
  url: string;

  /** 父窗口 ID */
  windowId: number;

  /** 当前缩放比例 */
  zoomFactor: number;
}

/**
 * Cookie
 */
export interface Cookie {
  /** 域名 */
  domain?: string;

  /** 过期时间（Unix 时间戳，毫秒） */
  expires?: number;

  /** 是否 HttpOnly */
  httpOnly?: boolean;
  /** Cookie 名称 */
  name: string;

  /** 路径 */
  path?: string;

  /** SameSite 策略 */
  sameSite?: "strict" | "lax" | "none";

  /** 是否安全 */
  secure?: boolean;

  /** Cookie 值 */
  value: string;
}

/**
 * Cookie 存储策略
 */
export type CookiePolicy =
  | "acceptAll" // 接受所有 Cookie
  | "acceptNone" // 拒绝所有 Cookie
  | "acceptOnlyFromVisited"; // 只接受来自访问过的站点的 Cookie

/**
 * WebView 事件类型
 */
export type WebViewEvent =
  | { type: "Created"; webviewId: number; windowId: number }
  | { type: "Destroyed"; webviewId: number }
  | {
      type: "NavigationStarting";
      webviewId: number;
      url: string;
      action: NavigationAction;
    }
  | {
      type: "NavigationCompleted";
      webviewId: number;
      url: string;
      success: boolean;
    }
  | { type: "LoadStateChanged"; webviewId: number; state: WebViewLoadState }
  | { type: "TitleChanged"; webviewId: number; title: string }
  | { type: "UrlChanged"; webviewId: number; url: string }
  | { type: "FaviconChanged"; webviewId: number; faviconUrl: string }
  | { type: "FullscreenChanged"; webviewId: number; isFullscreen: boolean }
  | { type: "ZoomFactorChanged"; webviewId: number; zoomFactor: number }
  | {
      type: "JavaScriptMessage";
      webviewId: number;
      channel: string;
      message: unknown;
    }
  | {
      type: "ConsoleMessage";
      webviewId: number;
      level: "debug" | "log" | "info" | "warning" | "error";
      message: string;
      line?: number;
      source?: string;
    }
  | {
      type: "BeforeContextMenu";
      webviewId: number;
      x: number;
      y: number;
      preventDefault: () => void;
    }
  | {
      type: "NewWindowRequest";
      webviewId: number;
      url: string;
      windowName: string;
      features: string;
      preventDefault: () => void;
    };

/**
 * WebView 错误类型
 */
export type WebViewError =
  | { type: "InvalidURL"; url: string; message: string }
  | { type: "LoadFailed"; url: string; error: string; errorCode: number }
  | { type: "JavaScriptError"; message: string; source?: string; line?: number }
  | { type: "NotFound"; webviewId: number }
  | { type: "WindowNotFound"; windowId: number }
  | { type: "SystemError"; message: string; code?: number };

/** WebView 选项（生成代码使用） */
export type WebViewOptions = WebViewParams;

/** WebView 边界 */
export interface WebViewBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * 查找选项
 */
export interface FindOptions {
  /** 是否向后搜索（默认：false） */
  backward?: boolean;

  /** 是否区分大小写（默认：false） */
  caseSensitive?: boolean;
  /** 搜索文本 */
  searchText: string;

  /** 是否全字匹配（默认：false） */
  wholeWord?: boolean;
}

/**
 * 查找结果
 */
export interface FindResult {
  /** 当前选中的索引 */
  activeMatchIndex: number;

  /** 是否完成搜索 */
  completed: boolean;
  /** 匹配数量 */
  matchCount: number;
}

/**
 * 截图选项
 */
export interface ScreenshotOptions {
  /** 输出格式 */
  format?: "png" | "jpeg";

  /** 图片质量（0-100，仅 jpeg） */
  quality?: number;
  /** 截图区域（不指定则截取整个页面） */
  rect?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };

  /** 缩放比例（默认：1.0） */
  scale?: number;
}

/**
 * 打印选项
 */
export interface PrintOptions {
  /** 页脚文本 */
  footer?: string;

  /** 页眉文本 */
  header?: string;

  /** 页边距 */
  margins?: {
    top?: number;
    bottom?: number;
    left?: number;
    right?: number;
  };

  /** 纸张尺寸 */
  paperSize?:
    | "A4"
    | "Letter"
    | "Legal"
    | "Tabloid"
    | { width: number; height: number };

  /** 是否打印背景 */
  printBackground?: boolean;

  /** 打印机名称（静默打印时使用） */
  printer?: string;
  /** 是否静默打印（不显示对话框） */
  silent?: boolean;
}

/**
 * 注入 CSS 选项
 */
export interface InjectCSSOptions {
  /** 是否在所有 frame 中注入（默认：false） */
  allFrames?: boolean;
  /** CSS 内容 */
  css: string;
}

/**
 * 执行 JavaScript 结果
 */
export type ExecuteJavaScriptResult<T = unknown> =
  | { success: true; result: T }
  | { success: false; error: string };
