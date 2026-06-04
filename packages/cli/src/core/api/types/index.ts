/**
 * Electrobun API 类型定义
 *
 * 所有类型定义都从这里导出
 * 这些类型将用于：
 * 1. TypeScript 开发时的类型检查
 * 2. 生成 Rust 代码和 FFI 绑定
 * 3. 生成 API 文档
 */

// App 相关
export * from "./app";

// Dialog 相关
export * from "./dialog";

// Tray 相关
export * from "./tray";
// WebView 相关
export * from "./webview";
// Window 相关
export * from "./window";

// ─── 以下为生成的 API Client 所需的错误/信息类型 ───

export type SessionError = { type: string; message: string };
export type ShortcutError = { type: string; message: string };
export interface DisplayInfo {
  id: number;
  bounds: { x: number; y: number; width: number; height: number };
  workArea: { x: number; y: number; width: number; height: number };
  scaleFactor: number;
  isPrimary: boolean;
}
export type DisplayError = { type: string; message: string };
export type CoreError = { type: string; message: string };

// WGPU 类型
export interface WgpuAdapterOpts {
  powerPreference?: "high-performance" | "low-power" | null;
  forceFallbackAdapter?: boolean | null;
}
export interface WgpuDeviceOpts {
  requiredFeatures?: string[] | null;
  requiredLimits?: Record<string, number> | null;
}
export interface WgpuSurfaceOpts {
  width: number;
  height: number;
  format?: string | null;
}
export type WgpuError = { message: string };

// Window 边界
export interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * 通用结果类型
 * 用于所有可能失败的 API 调用
 */
export type Result<T, E> =
  | { success: true; data: T }
  | { success: false; error: E };

/**
 * 异步结果类型
 * 用于异步 API 调用
 */
export type AsyncResult<T, E> = Promise<Result<T, E>>;

/**
 * 事件流类型
 * 用于订阅实时事件
 */
export interface EventStream<T> {
  /** 订阅事件 */
  subscribe(callback: (event: T) => void): () => void;

  /** 异步迭代器 */
  [Symbol.asyncIterator](): AsyncIterator<T>;
}

/**
 * 可取消的操作
 */
export interface Cancellable {
  /** 取消操作 */
  cancel(): void;

  /** 是否已取消 */
  isCancelled: boolean;
}

/**
 * 资源管理器
 * 用于管理需要手动释放的资源
 */
export interface Disposable {
  /** 释放资源 */
  dispose(): void;

  /** 是否已释放 */
  isDisposed: boolean;
}

/**
 * 版本信息
 */
export interface VersionInfo {
  /** 构建元数据 */
  build?: string;
  /** 主版本号 */
  major: number;

  /** 次版本号 */
  minor: number;

  /** 修订版本号 */
  patch: number;

  /** 预发布版本标识 */
  prerelease?: string;

  /** 完整版本字符串 */
  version: string;
}

/**
 * 矩形区域
 */
export interface Rect {
  height: number;
  width: number;
  x: number;
  y: number;
}

/**
 * 尺寸
 */
export interface Size {
  height: number;
  width: number;
}

/**
 * 点坐标
 */
export interface Point {
  x: number;
  y: number;
}

/**
 * 边距
 */
export interface Insets {
  bottom: number;
  left: number;
  right: number;
  top: number;
}
