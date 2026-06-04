import "./global";

type WgpuEventTypes = "ready";

/**
 * Interface representing an <electrobun-wgpu> custom element.
 * Use this to properly type wgpu elements obtained via querySelector.
 *
 * @example
 * const wgpu = document.querySelector('electrobun-wgpu') as WgpuTagElement;
 * wgpu.toggleTransparent(true);
 */
interface WgpuTagElement extends HTMLElement {
  // Mask management
  addMaskSelector(selector: string): void;
  emit(event: WgpuEventTypes, detail: unknown): void;
  hidden: boolean;
  off(event: WgpuEventTypes, listener: (event: CustomEvent) => void): void;

  // Events - listener receives a CustomEvent with detail property
  on(event: WgpuEventTypes, listener: (event: CustomEvent) => void): void;
  passthroughEnabled: boolean;
  removeMaskSelector(selector: string): void;

  // Debug helper
  runTest(): void;

  // Dimension sync
  syncDimensions(force?: boolean): void;
  toggleHidden(hidden?: boolean): void;
  togglePassthrough(enablePassthrough?: boolean): void;

  // Visibility and interaction
  toggleTransparent(transparent?: boolean): void;
  transparent: boolean;
  // Properties
  wgpuViewId?: number;
}

// Augment global types so querySelector('electrobun-wgpu') returns WgpuTagElement
declare global {
  interface HTMLElementTagNameMap {
    "electrobun-wgpu": WgpuTagElement;
  }
}

export type { WgpuEventTypes, WgpuTagElement };
