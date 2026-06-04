import type { Pointer } from "bun:ffi";

import { WGPUView } from "../core/WGPUView";
import { native_ } from "./core-lib";

export const WGPUBridge = {
  available: !!native_?.symbols?.wgpuInstanceCreateSurfaceMainThread,
  instanceCreateSurface: (
    instancePtr: Pointer,
    descriptorPtr: Pointer
  ): Pointer =>
    native_.symbols.wgpuInstanceCreateSurfaceMainThread(
      instancePtr as any,
      descriptorPtr as any
    ) as Pointer,
  surfaceConfigure: (surfacePtr: Pointer, configPtr: Pointer) =>
    native_.symbols.wgpuSurfaceConfigureMainThread(
      surfacePtr as any,
      configPtr as any
    ),
  surfaceGetCurrentTexture: (surfacePtr: Pointer, surfaceTexturePtr: Pointer) =>
    native_.symbols.wgpuSurfaceGetCurrentTextureMainThread(
      surfacePtr as any,
      surfaceTexturePtr as any
    ),
  surfacePresent: (surfacePtr: Pointer): number =>
    native_.symbols.wgpuSurfacePresentMainThread(surfacePtr as any),
  queueOnSubmittedWorkDone: (
    queuePtr: Pointer,
    callbackInfoPtr: Pointer
  ): bigint =>
    native_.symbols.wgpuQueueOnSubmittedWorkDoneShim(
      queuePtr as any,
      callbackInfoPtr as any
    ),
  bufferMapAsync: (
    bufferPtr: Pointer,
    mode: bigint,
    offset: bigint,
    size: bigint,
    callbackInfoPtr: Pointer
  ): bigint =>
    native_.symbols.wgpuBufferMapAsyncShim(
      bufferPtr as any,
      mode as any,
      offset as any,
      size as any,
      callbackInfoPtr as any
    ),
  instanceWaitAny: (
    instancePtr: Pointer,
    futureId: bigint,
    timeoutNs: bigint
  ): number =>
    native_.symbols.wgpuInstanceWaitAnyShim(
      instancePtr as any,
      futureId as any,
      timeoutNs as any
    ),
  bufferReadSync: (
    instancePtr: Pointer,
    bufferPtr: Pointer,
    offset: bigint,
    size: bigint,
    timeoutNs: bigint,
    outSizePtr: Pointer
  ): Pointer =>
    native_.symbols.wgpuBufferReadSyncShim(
      instancePtr as any,
      bufferPtr as any,
      offset as any,
      size as any,
      timeoutNs as any,
      outSizePtr as any
    ) as Pointer,
  bufferReadSyncInto: (
    instancePtr: Pointer,
    bufferPtr: Pointer,
    offset: bigint,
    size: bigint,
    timeoutNs: bigint,
    dstPtr: Pointer
  ): number =>
    native_.symbols.wgpuBufferReadSyncIntoShim(
      instancePtr as any,
      bufferPtr as any,
      offset as any,
      size as any,
      timeoutNs as any,
      dstPtr as any
    ),
  bufferReadbackBegin: (
    bufferPtr: Pointer,
    offset: bigint,
    size: bigint,
    dstPtr: Pointer
  ): Pointer =>
    native_.symbols.wgpuBufferReadbackBeginShim(
      bufferPtr as any,
      offset as any,
      size as any,
      dstPtr as any
    ) as Pointer,
  bufferReadbackStatus: (jobPtr: Pointer): number =>
    native_.symbols.wgpuBufferReadbackStatusShim(jobPtr as any),
  bufferReadbackFree: (jobPtr: Pointer) =>
    native_.symbols.wgpuBufferReadbackFreeShim(jobPtr as any),
  runTest: (viewId: number) => {
    const view = WGPUView.getById(viewId);
    if (!view?.ptr) {
      console.error(`wgpuRunGPUTest: WGPUView not found for id ${viewId}`);
      return;
    }
    native_.symbols.wgpuRunGPUTest(view.ptr);
  },
  createAdapterDeviceMainThread: (
    instancePtr: Pointer,
    surfacePtr: Pointer,
    outAdapterDevicePtr: Pointer
  ) =>
    native_.symbols.wgpuCreateAdapterDeviceMainThread(
      instancePtr as any,
      surfacePtr as any,
      outAdapterDevicePtr as any
    ),
  createSurfaceForView: (
    instancePtr: Pointer,
    viewPtr: Pointer
  ): Pointer | null => {
    if (!native_?.symbols?.wgpuCreateSurfaceForView) {
      return null;
    }
    return native_.symbols.wgpuCreateSurfaceForView(
      instancePtr as any,
      viewPtr as any
    ) as Pointer;
  },
};
