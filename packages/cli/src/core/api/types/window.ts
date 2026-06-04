/**
 * Window API 类型定义
 *
 * 定义窗口创建、管理和事件相关的所有类型
 * 这些类型将用于生成 Rust 代码和 FFI 绑定
 */

/**
 * 窗口创建参数
 */
export interface WindowParams {
  /** 是否置顶显示（默认：false） */
  alwaysOnTop?: boolean;

  /** 窗口背景颜色（格式：#RRGGBB 或 #RRGGBBAA） */
  backgroundColor?: string;

  /** 是否在屏幕居中显示（默认：true） */
  center?: boolean;

  /** 是否无边框（默认：false） */
  frameless?: boolean;

  /** 是否全屏显示（默认：false） */
  fullscreen?: boolean;

  /** 是否允许进入全屏（macOS，默认：true） */
  fullscreenable?: boolean;

  /** 是否显示窗口阴影（macOS，默认：true） */
  hasShadow?: boolean;

  /** 窗口高度（像素） */
  height: number;

  /** 初始 HTML 内容（与 url 二选一） */
  html?: string;

  /** 窗口图标路径（支持 .png, .ico） */
  icon?: string;

  /** 最大高度（像素） */
  maxHeight?: number;

  /** 最大宽度（像素） */
  maxWidth?: number;

  /** 最小高度（像素） */
  minHeight?: number;

  /** 最小宽度（像素） */
  minWidth?: number;

  /** 窗口透明度（0.0 - 1.0，默认：1.0） */
  opacity?: number;

  /** 是否可调整大小（默认：true） */
  resizable?: boolean;

  /** 是否在任务栏显示（默认：false，即显示） */
  skipTaskbar?: boolean;

  /** 窗口标题 */
  title: string;

  /** 是否透明背景（默认：false） */
  transparent?: boolean;

  /** 初始加载的 URL */
  url?: string;
  /** 窗口宽度（像素） */
  width: number;
}

/**
 * 窗口对象（创建后返回）
 */
export interface Window {
  /** 当前高度（像素） */
  height: number;
  /** 窗口唯一 ID */
  id: number;

  /** 是否已关闭 */
  isClosed: boolean;

  /** 是否聚焦 */
  isFocused: boolean;

  /** 是否全屏 */
  isFullscreen: boolean;

  /** 是否最大化 */
  isMaximized: boolean;

  /** 是否最小化 */
  isMinimized: boolean;

  /** 是否可见 */
  isVisible: boolean;

  /** 窗口标题 */
  title: string;

  /** 当前宽度（像素） */
  width: number;

  /** 窗口位置 X（像素） */
  x: number;

  /** 窗口位置 Y（像素） */
  y: number;
}

/**
 * 窗口事件类型
 */
export type WindowEvent =
  | { type: "Created"; windowId: number }
  | { type: "Shown"; windowId: number }
  | { type: "Hidden"; windowId: number }
  | { type: "Focused"; windowId: number }
  | { type: "Blurred"; windowId: number }
  | { type: "Resized"; windowId: number; width: number; height: number }
  | { type: "Moved"; windowId: number; x: number; y: number }
  | { type: "Closed"; windowId: number }
  | { type: "Minimized"; windowId: number }
  | { type: "Maximized"; windowId: number }
  | { type: "Restored"; windowId: number }
  | { type: "FullscreenEntered"; windowId: number }
  | { type: "FullscreenExited"; windowId: number };

/**
 * 窗口错误类型
 */
export type WindowError =
  | { type: "InvalidSize"; message: string; width?: number; height?: number }
  | { type: "InvalidPosition"; message: string; x?: number; y?: number }
  | { type: "InvalidTitle"; message: string }
  | { type: "CreationFailed"; message: string }
  | { type: "WindowNotFound"; windowId: number }
  | { type: "AlreadyClosed"; windowId: number }
  | { type: "NotResizable"; windowId: number }
  | { type: "SystemError"; message: string; code?: number };

/**
 * 窗口大小约束
 */
export interface WindowSizeConstraints {
  maxHeight?: number;
  maxWidth?: number;
  minHeight?: number;
  minWidth?: number;
}

/**
 * 窗口位置
 */
export interface WindowPosition {
  x: number;
  y: number;
}

/**
 * 窗口大小
 */
export interface WindowSize {
  height: number;
  width: number;
}

/** 窗口边界（生成代码使用） */
export interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * 窗口状态（用于保存/恢复）
 */
export interface WindowState {
  height: number;
  id: number;
  isFullscreen: boolean;
  isMaximized: boolean;
  isVisible: boolean;
  title: string;
  width: number;
  x: number;
  y: number;
}
