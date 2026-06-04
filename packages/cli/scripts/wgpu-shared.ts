/**
 * WGPU Dawn 库文件名常量（无路径，仅文件名）
 *
 * 各消费方（build.ts/cli/gen 模板）用这些基础文件名 + 各自路径拼接。
 * 原生 C++ 文件（nativeWrapper.cpp/mm）因路径格式不同暂独立维护。
 *
 * 修改库文件名时，还需同步更新：
 *   - src/native/win/nativeWrapper.cpp   LoadLibraryW
 *   - src/native/macos/nativeWrapper.mm   dlopen
 *   - src/native/linux/nativeWrapper.cpp  dlopen
 */

/** 按平台索引的 WGPU 库文件名 */
export const WGPU_LIB_FILENAMES = {
  darwin: ["libwebgpu_dawn.dylib"],
  win32: ["webgpu_dawn.dll", "libwebgpu_dawn.dll"],
  linux: ["libwebgpu_dawn.so"],
} as const;

/** WGPU 平台列表 */
export type WgpuPlatform = keyof typeof WGPU_LIB_FILENAMES;
