/**
 * Dialog API 类型定义
 *
 * 定义系统对话框（打开文件、保存文件、消息框）相关的所有类型
 */

/**
 * 文件过滤器
 */
export interface FileFilter {
  /** 文件扩展名列表（如：['png', 'jpg', 'jpeg']） */
  extensions: string[];
  /** 过滤器名称（显示在对话框中） */
  name: string;
}

/**
 * 打开文件对话框选项
 */
export interface OpenDialogOptions {
  /** 按钮文本（macOS） */
  buttonLabel?: string;

  /** 默认打开路径 */
  defaultPath?: string;

  /** 是否选择文件夹（默认：false，即选择文件） */
  directory?: boolean;

  /** 文件过滤器 */
  filters?: FileFilter[];

  /** 提示文本（macOS） */
  message?: string;

  /** 是否允许多选（默认：false） */
  multiple?: boolean;

  /** 是否显示隐藏文件（默认：false） */
  showHiddenFiles?: boolean;
  /** 对话框标题 */
  title?: string;
}

/**
 * 保存文件对话框选项
 */
export interface SaveDialogOptions {
  /** 按钮文本（macOS） */
  buttonLabel?: string;

  /** 默认保存路径（包含文件名） */
  defaultPath?: string;

  /** 文件过滤器 */
  filters?: FileFilter[];

  /** 提示文本（macOS） */
  message?: string;

  /** 是否允许覆盖（默认：true） */
  overwrite?: boolean;
  /** 对话框标题 */
  title?: string;
}

/**
 * 消息对话框按钮
 */
export interface MessageDialogButton {
  /** 按钮 ID */
  id: string;

  /** 是否为取消按钮（按 ESC 触发） */
  isCancel?: boolean;

  /** 是否为默认按钮（按回车触发） */
  isDefault?: boolean;

  /** 按钮文本 */
  label: string;
}

/**
 * 消息对话框选项
 */
export interface MessageDialogOptions {
  /** 自定义按钮（如果不提供，使用默认的 OK） */
  buttons?: MessageDialogButton[];

  /** 取消按钮索引 */
  cancelButton?: number;

  /** 默认按钮索引 */
  defaultButton?: number;

  /** 详细说明（可选） */
  detail?: string;

  /** 消息内容 */
  message: string;
  /** 对话框标题 */
  title?: string;

  /** 对话框类型 */
  type?: "info" | "warning" | "error" | "question";
}

/**
 * 确认对话框选项（简化版消息对话框）
 */
export interface ConfirmDialogOptions {
  /** 取消按钮文本（默认："取消"） */
  cancelLabel?: string;

  /** 消息内容 */
  message: string;

  /** 确认按钮文本（默认："确定"） */
  okLabel?: string;
  /** 对话框标题 */
  title?: string;
}

/**
 * 对话框错误类型
 */
export type DialogError =
  | { type: "Cancelled"; message: string }
  | { type: "InvalidPath"; path: string; message: string }
  | { type: "PermissionDenied"; path: string }
  | { type: "NotFound"; path: string }
  | { type: "SystemError"; message: string; code?: number };

/**
 * 文件对话框选项（生成代码使用）
 */
export type FileDialogOptions = OpenDialogOptions | SaveDialogOptions;

/**
 * 文件对话框结果（生成代码使用）
 */
export type FileDialogResult = OpenDialogResult | SaveDialogResult;

/**
 * 消息框结果（生成代码使用）
 */
export type MessageBoxResult = MessageDialogResult | ConfirmDialogResult;

/**
 * 打开对话框结果
 */
export type OpenDialogResult =
  | { success: true; filePaths: string[] }
  | { success: false; error: DialogError };

/**
 * 保存对话框结果
 */
export type SaveDialogResult =
  | { success: true; filePath: string }
  | { success: false; error: DialogError };

/**
 * 消息对话框结果
 */
export type MessageDialogResult =
  | { success: true; buttonId: string }
  | { success: false; error: DialogError };

/**
 * 确认对话框结果
 */
export type ConfirmDialogResult =
  | { success: true; confirmed: boolean }
  | { success: false; error: DialogError };
