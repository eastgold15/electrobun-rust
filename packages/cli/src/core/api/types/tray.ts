/**
 * Tray API 类型定义
 *
 * 定义系统托盘图标和菜单相关的所有类型
 */

/**
 * 托盘菜单项类型
 */
export type TrayMenuItem =
  | TrayMenuNormalItem
  | TrayMenuSeparatorItem
  | TrayMenuSubmenuItem
  | TrayMenuCheckboxItem;

/**
 * 普通菜单项
 */
export interface TrayMenuNormalItem {
  /** 快捷键（如："Cmd+O"） */
  accelerator?: string;

  /** 是否可用（默认：true） */
  enabled?: boolean;

  /** 图标路径（可选） */
  icon?: string;

  /** 菜单项 ID（唯一标识） */
  id: string;

  /** 显示文本 */
  label: string;

  /** 提示文本 */
  tooltip?: string;
  type: "normal";

  /** 是否可见（默认：true） */
  visible?: boolean;
}

/**
 * 分隔线菜单项
 */
export interface TrayMenuSeparatorItem {
  type: "separator";
}

/**
 * 子菜单项
 */
export interface TrayMenuSubmenuItem {
  /** 是否可用（默认：true） */
  enabled?: boolean;

  /** 图标路径（可选） */
  icon?: string;

  /** 菜单项 ID */
  id: string;

  /** 显示文本 */
  label: string;

  /** 子菜单项列表 */
  submenu: TrayMenuItem[];
  type: "submenu";

  /** 是否可见（默认：true） */
  visible?: boolean;
}

/**
 * 复选框菜单项
 */
export interface TrayMenuCheckboxItem {
  /** 是否选中（默认：false） */
  checked?: boolean;

  /** 是否可用（默认：true） */
  enabled?: boolean;

  /** 图标路径（可选） */
  icon?: string;

  /** 菜单项 ID */
  id: string;

  /** 显示文本 */
  label: string;
  type: "checkbox";

  /** 是否可见（默认：true） */
  visible?: boolean;
}

/**
 * 托盘创建参数
 */
export interface TrayParams {
  /** 托盘图标路径（支持 .png, .ico, .icns） */
  icon: string;

  /** 右键菜单项 */
  menu?: TrayMenuItem[];

  /** 托盘提示文本（鼠标悬停显示） */
  tooltip?: string;

  /** 是否显示托盘图标（默认：true） */
  visible?: boolean;
}

/** 托盘选项（生成代码使用） */
export type TrayOptions = TrayParams;

/**
 * 托盘对象
 */
export interface Tray {
  /** 托盘图标路径 */
  icon: string;
  /** 托盘唯一 ID */
  id: number;

  /** 是否可见 */
  isVisible: boolean;

  /** 托盘提示文本 */
  tooltip: string;
}

/**
 * 托盘事件类型
 */
export type TrayEvent =
  | {
      type: "Clicked";
      trayId: number;
      x: number;
      y: number;
      button: "left" | "right" | "middle";
    }
  | {
      type: "DoubleClicked";
      trayId: number;
      x: number;
      y: number;
      button: "left" | "right" | "middle";
    }
  | { type: "MenuItemClicked"; trayId: number; menuItemId: string }
  | {
      type: "MenuItemToggled";
      trayId: number;
      menuItemId: string;
      checked: boolean;
    }
  | { type: "BalloonShown"; trayId: number }
  | { type: "BalloonClicked"; trayId: number }
  | { type: "BalloonClosed"; trayId: number };

/**
 * 托盘错误类型
 */
export type TrayError =
  | { type: "IconNotFound"; path: string }
  | { type: "InvalidIconFormat"; path: string; message: string }
  | { type: "TrayNotFound"; trayId: number }
  | { type: "MenuItemNotFound"; trayId: number; menuItemId: string }
  | { type: "SystemError"; message: string; code?: number };

/**
 * 气泡通知选项
 */
export interface TrayBalloonOptions {
  /** 内容 */
  content: string;

  /** 图标类型 */
  iconType?: "none" | "info" | "warning" | "error";

  /** 显示时间（毫秒，默认：5000） */
  timeout?: number;
  /** 标题 */
  title: string;
}
