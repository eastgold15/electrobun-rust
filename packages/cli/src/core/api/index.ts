/**
 * Electrobun API 定义
 *
 * 这是用户直接使用的 API 接口定义
 * 基于 Eden 风格设计，提供端到端类型安全
 *
 * 使用示例：
 * ```typescript
 * import { electrobun } from '@pori15/electrobun-rust';
 *
 * async function main() {
 *   // 创建窗口
 *   const window = await electrobun.window.create({
 *     width: 1200,
 *     height: 800,
 *     title: 'My App',
 *     url: 'electrobun://app/index.html',
 *   });
 *
 *   // 订阅窗口事件
 *   for await (const event of electrobun.window.onEvent()) {
 *     if (event.type === 'Closed') {
 *       await electrobun.app.quit();
 *     }
 *   }
 * }
 * ```
 */

import type {
  AppError,
  AppEvent,
  // App
  AppInfo,
  AppInitOptions,
  AsyncResult,
  ClipboardError,
  ConfirmDialogOptions,
  DialogError,
  EventStream,
  FindOptions,
  FindResult,
  InjectCSSOptions,
  MessageDialogOptions,
  NotificationError,
  NotificationEvent,
  NotificationOptions,
  // Dialog
  OpenDialogOptions,
  PlatformInfo,
  PrintOptions,
  SaveDialogOptions,
  ScreenshotOptions,
  ShellError,
  ShellOpenOptions,
  SystemInfo,
  Tray,
  TrayBalloonOptions,
  TrayError,
  TrayEvent,
  TrayMenuItem,
  // Tray
  TrayParams,
  WebView,
  WebViewError,
  WebViewEvent,
  // WebView
  WebViewParams,
  Window,
  WindowError,
  WindowEvent,
  // Window
  WindowParams,
  WindowState,
} from "./types";

// ============================================================================
// Window API
// ============================================================================

export interface WindowAPI {
  /**
   * 失焦窗口
   * @param windowId 窗口 ID
   */
  blur(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 关闭窗口
   * @param windowId 窗口 ID
   */
  close(windowId: number): AsyncResult<void, WindowError>;
  /**
   * 创建新窗口
   * @param params 窗口创建参数
   * @returns 创建的窗口对象
   */
  create(params: WindowParams): AsyncResult<Window, WindowError>;

  /**
   * 聚焦窗口
   * @param windowId 窗口 ID
   */
  focus(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 获取窗口信息
   * @param windowId 窗口 ID
   */
  get(windowId: number): AsyncResult<Window, WindowError>;

  /**
   * 获取所有窗口
   */
  getAll(): AsyncResult<Window[], WindowError>;

  /**
   * 隐藏窗口
   * @param windowId 窗口 ID
   */
  hide(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 最大化窗口
   * @param windowId 窗口 ID
   */
  maximize(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 最小化窗口
   * @param windowId 窗口 ID
   */
  minimize(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 订阅窗口事件
   * @returns 事件流
   */
  onEvent(): EventStream<WindowEvent>;

  /**
   * 订阅特定窗口事件
   * @param windowId 窗口 ID
   */
  onWindowEvent(windowId: number): EventStream<WindowEvent>;

  /**
   * 恢复窗口
   * @param windowId 窗口 ID
   */
  restore(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 恢复窗口状态
   * @param state 窗口状态
   */
  restoreState(state: WindowState): AsyncResult<Window, WindowError>;

  /**
   * 保存窗口状态
   * @param windowId 窗口 ID
   */
  saveState(windowId: number): AsyncResult<WindowState, WindowError>;

  /**
   * 设置窗口置顶
   * @param windowId 窗口 ID
   * @param alwaysOnTop 是否置顶
   */
  setAlwaysOnTop(
    windowId: number,
    alwaysOnTop: boolean
  ): AsyncResult<void, WindowError>;

  /**
   * 全屏切换
   * @param windowId 窗口 ID
   * @param fullscreen 是否全屏
   */
  setFullscreen(
    windowId: number,
    fullscreen: boolean
  ): AsyncResult<void, WindowError>;

  /**
   * 设置窗口透明度
   * @param windowId 窗口 ID
   * @param opacity 透明度（0.0 - 1.0）
   */
  setOpacity(windowId: number, opacity: number): AsyncResult<void, WindowError>;

  /**
   * 设置窗口位置
   * @param windowId 窗口 ID
   * @param x X 坐标
   * @param y Y 坐标
   */
  setPosition(
    windowId: number,
    x: number,
    y: number
  ): AsyncResult<void, WindowError>;

  /**
   * 设置窗口大小
   * @param windowId 窗口 ID
   * @param width 宽度
   * @param height 高度
   */
  setSize(
    windowId: number,
    width: number,
    height: number
  ): AsyncResult<void, WindowError>;

  /**
   * 设置窗口标题
   * @param windowId 窗口 ID
   * @param title 新标题
   */
  setTitle(windowId: number, title: string): AsyncResult<void, WindowError>;

  /**
   * 显示窗口
   * @param windowId 窗口 ID
   */
  show(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 开始窗口拖拽（用于无边框窗口）
   * @param windowId 窗口 ID
   */
  startDrag(windowId: number): AsyncResult<void, WindowError>;

  /**
   * 停止窗口拖拽
   * @param windowId 窗口 ID
   */
  stopDrag(windowId: number): AsyncResult<void, WindowError>;
}

// ============================================================================
// Dialog API
// ============================================================================

export interface DialogAPI {
  /**
   * 显示确认对话框
   * @param options 对话框选项
   */
  showConfirm(options: ConfirmDialogOptions): AsyncResult<boolean, DialogError>;

  /**
   * 显示错误对话框
   * @param title 标题
   * @param message 消息
   */
  showError(title: string, message: string): AsyncResult<void, DialogError>;

  /**
   * 显示信息对话框
   * @param title 标题
   * @param message 消息
   */
  showInfo(title: string, message: string): AsyncResult<void, DialogError>;

  /**
   * 显示消息对话框
   * @param options 对话框选项
   */
  showMessage(options: MessageDialogOptions): AsyncResult<string, DialogError>;
  /**
   * 显示打开文件对话框
   * @param options 对话框选项
   */
  showOpen(
    options?: OpenDialogOptions
  ): AsyncResult<string[] | null, DialogError>;

  /**
   * 显示保存文件对话框
   * @param options 对话框选项
   */
  showSave(
    options?: SaveDialogOptions
  ): AsyncResult<string | null, DialogError>;

  /**
   * 显示警告对话框
   * @param title 标题
   * @param message 消息
   */
  showWarning(title: string, message: string): AsyncResult<void, DialogError>;
}

// ============================================================================
// Tray API
// ============================================================================

export interface TrayAPI {
  /**
   * 创建系统托盘
   * @param params 托盘参数
   */
  create(params: TrayParams): AsyncResult<Tray, TrayError>;

  /**
   * 销毁托盘
   * @param trayId 托盘 ID
   */
  destroy(trayId: number): AsyncResult<void, TrayError>;

  /**
   * 隐藏托盘
   * @param trayId 托盘 ID
   */
  hide(trayId: number): AsyncResult<void, TrayError>;

  /**
   * 订阅托盘事件
   */
  onEvent(): EventStream<TrayEvent>;

  /**
   * 订阅特定托盘事件
   * @param trayId 托盘 ID
   */
  onTrayEvent(trayId: number): EventStream<TrayEvent>;

  /**
   * 设置托盘图标
   * @param trayId 托盘 ID
   * @param iconPath 图标路径
   */
  setIcon(trayId: number, iconPath: string): AsyncResult<void, TrayError>;

  /**
   * 设置托盘菜单
   * @param trayId 托盘 ID
   * @param menu 菜单项
   */
  setMenu(trayId: number, menu: TrayMenuItem[]): AsyncResult<void, TrayError>;

  /**
   * 设置托盘提示文本
   * @param trayId 托盘 ID
   * @param tooltip 提示文本
   */
  setTooltip(trayId: number, tooltip: string): AsyncResult<void, TrayError>;

  /**
   * 显示托盘
   * @param trayId 托盘 ID
   */
  show(trayId: number): AsyncResult<void, TrayError>;

  /**
   * 显示气泡通知
   * @param trayId 托盘 ID
   * @param options 气泡选项
   */
  showBalloon(
    trayId: number,
    options: TrayBalloonOptions
  ): AsyncResult<void, TrayError>;
}

// ============================================================================
// App API
// ============================================================================

export interface AppAPI {
  /**
   * 添加最近文档
   * @param path 文档路径
   */
  addRecentDocument(path: string): AsyncResult<void, AppError>;

  /**
   * 退出前清理
   */
  beforeQuit(): AsyncResult<void, AppError>;

  /**
   * 清空最近文档
   */
  clearRecentDocuments(): AsyncResult<void, AppError>;

  /**
   * 获取应用信息
   */
  getInfo(): AsyncResult<AppInfo, AppError>;

  /**
   * 获取应用本地标识符
   */
  getLocale(): AsyncResult<string, AppError>;

  /**
   * 获取应用名称
   */
  getName(): AsyncResult<string, AppError>;

  /**
   * 获取应用路径
   * @param name 路径名称
   */
  getPath(
    name:
      | "home"
      | "appData"
      | "userData"
      | "temp"
      | "exe"
      | "module"
      | "desktop"
      | "documents"
      | "downloads"
      | "music"
      | "pictures"
      | "videos"
      | "logs"
      | "crashDumps"
  ): AsyncResult<string, AppError>;

  /**
   * 获取平台信息
   */
  getPlatformInfo(): AsyncResult<PlatformInfo, AppError>;

  /**
   * 获取系统信息
   */
  getSystemInfo(): AsyncResult<SystemInfo, AppError>;

  /**
   * 获取应用版本
   */
  getVersion(): AsyncResult<string, AppError>;

  /**
   * 是否为单实例主进程
   */
  hasSingleInstanceLock(): AsyncResult<boolean, AppError>;
  /**
   * 初始化应用
   * @param options 初始化选项
   */
  init(options: AppInitOptions): AsyncResult<void, AppError>;

  /**
   * 订阅应用事件
   */
  onEvent(): EventStream<AppEvent>;

  /**
   * 退出应用
   * @param exitCode 退出码（默认：0）
   */
  quit(exitCode?: number): AsyncResult<void, AppError>;

  /**
   * 释放单实例锁
   */
  releaseSingleInstanceLock(): AsyncResult<void, AppError>;

  /**
   * 请求单实例锁
   */
  requestSingleInstanceLock(): AsyncResult<boolean, AppError>;

  /**
   * 设置应用路径
   * @param name 路径名称
   * @param path 路径值
   */
  setPath(name: string, path: string): AsyncResult<void, AppError>;

  /**
   * 设置用户任务（Windows）
   * @param tasks 任务列表
   */
  setUserTasks(
    tasks: Array<{
      program: string;
      arguments: string;
      title: string;
      description: string;
      iconPath?: string;
      iconIndex?: number;
    }>
  ): AsyncResult<void, AppError>;
}

// ============================================================================
// Clipboard API
// ============================================================================

export interface ClipboardAPI {
  /**
   * 清空剪贴板
   */
  clear(): AsyncResult<void, ClipboardError>;

  /**
   * 检查剪贴板格式是否可用
   * @param format 格式类型
   */
  has(
    format: "text" | "html" | "image" | "file"
  ): AsyncResult<boolean, ClipboardError>;

  /**
   * 读取剪贴板文件列表
   */
  readFiles(): AsyncResult<string[], ClipboardError>;

  /**
   * 读取剪贴板 HTML
   */
  readHTML(): AsyncResult<string, ClipboardError>;

  /**
   * 读取剪贴板图片
   */
  readImage(): AsyncResult<string, ClipboardError>; // 返回图片路径
  /**
   * 读取剪贴板文本
   */
  readText(): AsyncResult<string, ClipboardError>;

  /**
   * 写入剪贴板文件列表
   * @param filePaths 文件路径列表
   */
  writeFiles(filePaths: string[]): AsyncResult<void, ClipboardError>;

  /**
   * 写入剪贴板 HTML
   * @param html HTML 内容
   * @param text 纯文本内容（可选）
   */
  writeHTML(html: string, text?: string): AsyncResult<void, ClipboardError>;

  /**
   * 写入剪贴板图片
   * @param imagePath 图片路径
   */
  writeImage(imagePath: string): AsyncResult<void, ClipboardError>;

  /**
   * 写入剪贴板文本
   * @param text 文本内容
   */
  writeText(text: string): AsyncResult<void, ClipboardError>;
}

// ============================================================================
// Shell API
// ============================================================================

export interface ShellAPI {
  /**
   * 播放哔声
   */
  beep(): AsyncResult<void, ShellError>;
  /**
   * 使用默认应用打开文件/URL
   * @param path 文件路径或 URL
   * @param options 打开选项
   */
  open(path: string, options?: ShellOpenOptions): AsyncResult<void, ShellError>;

  /**
   * 在文件管理器中显示文件
   * @param path 文件路径
   */
  showItemInFolder(path: string): AsyncResult<void, ShellError>;

  /**
   * 将文件移动到回收站
   * @param path 文件路径
   */
  trashItem(path: string): AsyncResult<void, ShellError>;
}

// ============================================================================
// Notification API
// ============================================================================

export interface NotificationAPI {
  /**
   * 检查通知权限
   */
  checkPermission(): AsyncResult<
    "granted" | "denied" | "default",
    NotificationError
  >;

  /**
   * 关闭通知
   * @param notificationId 通知 ID
   */
  close(notificationId: string): AsyncResult<void, NotificationError>;

  /**
   * 订阅通知事件
   */
  onEvent(): EventStream<NotificationEvent>;

  /**
   * 请求通知权限
   */
  requestPermission(): AsyncResult<"granted" | "denied", NotificationError>;
  /**
   * 显示通知
   * @param options 通知选项
   */
  show(options: NotificationOptions): AsyncResult<string, NotificationError>;
}

// ============================================================================
// WebView API
// ============================================================================

export interface WebViewAPI {
  /**
   * 截图
   * @param webviewId WebView ID
   * @param options 截图选项
   * @returns 图片路径
   */
  capturePage(
    webviewId: number,
    options?: ScreenshotOptions
  ): AsyncResult<string, WebViewError>;

  /**
   * 关闭开发者工具
   * @param webviewId WebView ID
   */
  closeDevTools(webviewId: number): AsyncResult<void, WebViewError>;
  /**
   * 创建 WebView
   * @param params WebView 参数
   */
  create(params: WebViewParams): AsyncResult<WebView, WebViewError>;

  /**
   * 销毁 WebView
   * @param webviewId WebView ID
   */
  destroy(webviewId: number): AsyncResult<void, WebViewError>;

  /**
   * 执行 JavaScript
   * @param webviewId WebView ID
   * @param code JavaScript 代码
   * @returns 执行结果
   */
  executeJavaScript<T = unknown>(
    webviewId: number,
    code: string
  ): AsyncResult<T, WebViewError>;

  /**
   * 查找文本
   * @param webviewId WebView ID
   * @param options 查找选项
   */
  findInPage(
    webviewId: number,
    options: FindOptions
  ): AsyncResult<FindResult, WebViewError>;

  /**
   * 获取 WebView
   * @param webviewId WebView ID
   */
  get(webviewId: number): AsyncResult<WebView, WebViewError>;

  /**
   * 获取所有 WebView
   */
  getAll(): AsyncResult<WebView[], WebViewError>;

  /**
   * 获取缩放比例
   * @param webviewId WebView ID
   */
  getZoomFactor(webviewId: number): AsyncResult<number, WebViewError>;

  /**
   * 后退
   * @param webviewId WebView ID
   */
  goBack(webviewId: number): AsyncResult<void, WebViewError>;

  /**
   * 前进
   * @param webviewId WebView ID
   */
  goForward(webviewId: number): AsyncResult<void, WebViewError>;

  /**
   * 注入 CSS
   * @param webviewId WebView ID
   * @param options CSS 注入选项
   */
  insertCSS(
    webviewId: number,
    options: InjectCSSOptions
  ): AsyncResult<void, WebViewError>;

  /**
   * 检查开发者工具是否打开
   * @param webviewId WebView ID
   */
  isDevToolsOpened(webviewId: number): AsyncResult<boolean, WebViewError>;

  /**
   * 加载 HTML
   * @param webviewId WebView ID
   * @param html HTML 内容
   * @param baseURL 基础 URL（可选）
   */
  loadHTML(
    webviewId: number,
    html: string,
    baseURL?: string
  ): AsyncResult<void, WebViewError>;

  /**
   * 加载 URL
   * @param webviewId WebView ID
   * @param url URL
   */
  loadURL(webviewId: number, url: string): AsyncResult<void, WebViewError>;

  /**
   * 订阅 WebView 事件
   */
  onEvent(): EventStream<WebViewEvent>;

  /**
   * 订阅特定 WebView 事件
   * @param webviewId WebView ID
   */
  onWebViewEvent(webviewId: number): EventStream<WebViewEvent>;

  /**
   * 打开开发者工具
   * @param webviewId WebView ID
   */
  openDevTools(webviewId: number): AsyncResult<void, WebViewError>;

  /**
   * 打印
   * @param webviewId WebView ID
   * @param options 打印选项
   */
  print(
    webviewId: number,
    options?: PrintOptions
  ): AsyncResult<void, WebViewError>;

  /**
   * 重新加载
   * @param webviewId WebView ID
   */
  reload(webviewId: number): AsyncResult<void, WebViewError>;

  /**
   * 移除注入的 CSS
   * @param webviewId WebView ID
   * @param key CSS key
   */
  removeInsertedCSS(
    webviewId: number,
    key: string
  ): AsyncResult<void, WebViewError>;

  /**
   * 发送消息到 WebView
   * @param webviewId WebView ID
   * @param channel 消息通道
   * @param message 消息内容
   */
  send(
    webviewId: number,
    channel: string,
    message: unknown
  ): AsyncResult<void, WebViewError>;

  /**
   * 设置缩放比例
   * @param webviewId WebView ID
   * @param factor 缩放比例
   */
  setZoomFactor(
    webviewId: number,
    factor: number
  ): AsyncResult<void, WebViewError>;

  /**
   * 停止加载
   * @param webviewId WebView ID
   */
  stop(webviewId: number): AsyncResult<void, WebViewError>;

  /**
   * 停止查找
   * @param webviewId WebView ID
   * @param clearSelection 是否清除选择
   */
  stopFindInPage(
    webviewId: number,
    clearSelection?: boolean
  ): AsyncResult<void, WebViewError>;
}

// ============================================================================
// Main Electrobun API
// ============================================================================

/**
 * Electrobun 主 API
 *
 * 这是用户使用的唯一入口点
 */
export interface ElectrobunAPI {
  /** 应用 API */
  app: AppAPI;

  /** 剪贴板 API */
  clipboard: ClipboardAPI;

  /** 对话框 API */
  dialog: DialogAPI;

  /** 通知 API */
  notification: NotificationAPI;

  /** Shell API */
  shell: ShellAPI;

  /** 系统托盘 API */
  tray: TrayAPI;

  /** WebView API */
  webview: WebViewAPI;
  /** 窗口 API */
  window: WindowAPI;
}

/**
 * 创建 Electrobun API 客户端
 * @param corePath electrobun_core.dll 路径
 * @returns Electrobun API 实例
 */
export declare function createElectrobun(corePath: string): ElectrobunAPI;

/**
 * 全局 Electrobun 实例
 * （在应用初始化后可用）
 */
export declare const electrobun: ElectrobunAPI;

// 重新导出所有类型
export * from "./types";
