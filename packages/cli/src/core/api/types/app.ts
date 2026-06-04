/**
 * App API 类型定义
 *
 * 定义应用生命周期、全局事件和系统相关的所有类型
 */

/**
 * 应用信息
 */
export interface AppInfo {
  /** 应用标识符（如：com.example.app） */
  bundleId?: string;

  /** 版权信息 */
  copyright?: string;

  /** 应用描述 */
  description?: string;

  /** 主页 URL */
  homepage?: string;

  /** 应用图标路径 */
  icon?: string;
  /** 应用名称 */
  name: string;

  /** 应用版本 */
  version: string;
}

/**
 * 应用事件类型
 */
export type AppEvent =
  | { type: "Ready" }
  | { type: "WillFinishLaunching" }
  | { type: "DidFinishLaunching" }
  | { type: "WillQuit" }
  | { type: "DidQuit" }
  | { type: "Activated"; hasVisibleWindows: boolean }
  | { type: "Deactivated" }
  | { type: "OpenURL"; url: string }
  | { type: "OpenFile"; filePath: string }
  | {
      type: "ContinueActivity";
      activityType: string;
      userInfo: Record<string, unknown>;
    }
  | { type: "UpdateAvailable"; version: string; releaseNotes?: string }
  | { type: "BeforeQuit" }
  | { type: "SecondInstance"; argv: string[]; cwd: string };

/**
 * 应用退出策略
 */
export type AppQuitStrategy =
  | "default" // 默认行为：最后一个窗口关闭时退出
  | "never" // 从不自动退出
  | "lastWindow"; // 最后一个窗口关闭时退出（macOS 上保持运行）

/**
 * 应用初始化选项
 */
export interface AppInitOptions {
  /** 是否启用自动更新检查（默认：false） */
  autoUpdater?: boolean;

  /** 是否启用开发者工具（默认：false） */
  devTools?: boolean;

  /** 是否启用硬件加速（默认：true） */
  hardwareAcceleration?: boolean;
  /** 应用信息 */
  info: AppInfo;

  /** 退出策略（默认：'default'） */
  quitStrategy?: AppQuitStrategy;

  /** 是否启用沙盒（默认：true） */
  sandbox?: boolean;

  /** 是否允许单实例（默认：false） */
  singleInstance?: boolean;

  /** 更新检查间隔（毫秒，默认：24小时） */
  updateCheckInterval?: number;
}

/**
 * 应用错误类型
 */
export type AppError =
  | { type: "AlreadyInitialized"; message: string }
  | { type: "NotInitialized"; message: string }
  | { type: "InitializationFailed"; message: string }
  | { type: "SingleInstanceLockFailed"; message: string }
  | { type: "QuitFailed"; message: string }
  | { type: "SystemError"; message: string; code?: number };

/**
 * 平台信息
 */
export interface PlatformInfo {
  /** 架构 */
  arch: "x64" | "arm64";

  /** 是否深色模式 */
  isDarkMode: boolean;

  /** 是否生产环境 */
  isProduction: boolean;

  /** 是否沙盒环境 */
  isSandboxed: boolean;

  /** 语言设置 */
  locale: string;
  /** 操作系统类型 */
  os: "windows" | "macos" | "linux";

  /** 操作系统版本 */
  osVersion: string;
}

/**
 * 系统信息
 */
export interface SystemInfo {
  /** 应用数据目录 */
  appDataDir: string;

  /** 缓存目录 */
  cacheDir: string;

  /** CPU 架构 */
  cpuArch: string;

  /** CPU 核心数 */
  cpuCount: number;

  /** 可用内存（字节） */
  freeMemory: number;

  /** 主目录 */
  homeDir: string;

  /** 主机名 */
  hostname: string;

  /** 日志目录 */
  logsDir: string;

  /** 临时目录 */
  tempDir: string;
  /** 总内存（字节） */
  totalMemory: number;

  /** 用户名 */
  username: string;
}

/**
 * 剪贴板内容类型
 */
export type ClipboardContent =
  | { type: "text"; text: string }
  | { type: "html"; html: string; text?: string }
  | { type: "image"; imagePath: string }
  | { type: "file"; filePaths: string[] };

/**
 * 剪贴板错误类型
 */
export type ClipboardError =
  | { type: "Empty"; message: string }
  | { type: "UnsupportedFormat"; format: string }
  | { type: "ReadFailed"; message: string }
  | { type: "WriteFailed"; message: string }
  | { type: "SystemError"; message: string; code?: number };

/**
 * Shell 打开选项
 */
export interface ShellOpenOptions {
  /** 指定应用路径（可选） */
  application?: string;
  /** 是否使用默认应用打开（默认：true） */
  useDefault?: boolean;

  /** 是否等待完成（默认：false） */
  wait?: boolean;

  /** 工作目录 */
  workingDirectory?: string;
}

/**
 * Shell 错误类型
 */
export type ShellError =
  | { type: "NotFound"; path: string }
  | { type: "PermissionDenied"; path: string }
  | { type: "ExecutionFailed"; command: string; message: string }
  | { type: "SystemError"; message: string; code?: number };

/**
 * 通知选项
 */
export interface NotificationOptions {
  /** 操作按钮 */
  actions?: Array<{
    id: string;
    text: string;
  }>;

  /** 通知内容 */
  body?: string;

  /** 通知图标 */
  icon?: string;

  /** 通知 ID（用于替换） */
  id?: string;

  /** 通知图片（macOS） */
  image?: string;

  /** 是否静默（不播放声音） */
  silent?: boolean;

  /** 超时时间（毫秒，默认：5000） */
  timeout?: number;
  /** 通知标题 */
  title: string;
}

/**
 * 通知事件类型
 */
export type NotificationEvent =
  | { type: "Shown"; notificationId: string }
  | { type: "Clicked"; notificationId: string }
  | { type: "Closed"; notificationId: string; byUser: boolean }
  | { type: "Action"; notificationId: string; actionId: string };

/**
 * 通知错误类型
 */
export type NotificationError =
  | { type: "PermissionDenied"; message: string }
  | { type: "NotSupported"; message: string }
  | { type: "ShowFailed"; message: string }
  | { type: "SystemError"; message: string; code?: number };
