# CEF 版本管理

内部参考文档，说明 Electrobun 如何管理 CEF（Chromium Embedded Framework）版本、构建和分发。

## Tarball 布局

Electrobun 发布为每个平台提供 3 个 tarball：

| Tarball | 内容 | 来源 |
|---------|----------|--------|
| `electrobun-cli-*` | CLI 二进制文件 | `bin/` |
| `electrobun-core-*` | 平台二进制文件，包括 `process_helper` | `dist/`（不包括 `cef/` 目录和以 "electrobun" 开头的文件） |
| `electrobun-cef-*` | 仅 CEF 运行时文件（无 electrobun 代码） | `dist/cef/` |

`process_helper` 包含在 **core** tarball 中，而不是 CEF tarball 中。这意味着 CEF tarball 仅包含上游 CEF 分发的文件，可以独立替换。

## CEF 如何构建

默认的 CEF 版本硬编码在 `package/build.ts` 中：

```typescript
const CEF_VERSION = `144.0.11+ge135be2`;
const CHROMIUM_VERSION = `144.0.7559.97`;
```

当运行 `bun build.ts` 时，`vendorCEF()` 执行以下操作：

1. **下载** 来自 `cef-builds.spotifycdn.com` 的 CEF 最小发行版
2. **构建 `libcef_dll_wrapper.a`** 使用 cmake（CEF 稳定 C API 的薄 C++ 包装器）
3. **从源代码编译 `process_helper`**（`src/native/{platform}/cef_process_helper_*`）

然后 `copyToDist()` 将 CEF 运行时文件复制到 `dist/cef/`，将 `process_helper` 复制到 `dist/`。

### 链接关系

```
process_helper
  静态链接 libcef_dll_wrapper.a  （在构建时编译）
    调用 CEF C API 符号（cef_execute_process 等）
      在运行时从 libcef.so / .dll / .framework 解析

libNativeWrapper
  静态链接 libcef_dll_wrapper.a  （在构建时编译）
    通过以下方式运行时加载 libcef：
      macOS: weak_framework
      Windows: DELAYLOAD
      Linux: dlopen (cef_loader.cpp)
```

`libcef_dll_wrapper.a` 是 `process_helper` 和 `libNativeWrapper` 的链接时依赖项。它不包含任何 CEF 实现——它只是将 C++ 调用转发到 CEF 的 C API，后者在运行时从实际的 CEF 共享库中解析。

## 发布工作流缓存

发布工作流（`.github/workflows/release.yml`）缓存两件事以避免重复工作：

### CEF vendor 缓存
```
key: cef-{platform}-{arch}-{cef_version}
path: package/vendors/cef
```
涵盖 CEF 下载和 `libcef_dll_wrapper.a` 构建。缓存命中时，cmake 不会重新运行。

### process_helper 缓存
```
key: process-helper-{platform}-{arch}-{cef_version}-{cef_process_helper_* 源文件的哈希}
path: package/src/native/build/process_helper[.exe]
```
`process_helper` 很少更改。此缓存在 CEF 版本和助手源代码都未更改时跳过其编译。`build.ts` 检查二进制文件是否存在，如果存在则跳过构建。

`libNativeWrapper` 未被缓存，因为它经常更改。

## 自定义 CEF 版本（终端用户流程）

通过 npm 使用 electrobun 的开发者可以在他们的 `electrobun.config.ts` 中覆盖 CEF 版本：

```typescript
export default {
  build: {
    cefVersion: "145.0.1+gabcdef0+chromium-145.0.7600.50",
    // ...
  },
} satisfies ElectrobunConfig;
```

设置后，CLI 的 `downloadAndExtractCustomCEF()` 函数：

1. 从 Spotify CDN 下载最小发行版
2. 解压它
3. 将运行时文件从 `Release/` 和 `Resources/` 复制到扁平的 `cef/` 布局
4. 写入 `.cef-version` 标记用于缓存检测

不进行编译。`process_helper` 已经在 core tarball 中，并通过稳定的 C API 与替换的 CEF 运行时一起工作。

### C API 兼容性

CEF 的 C API 设计为在同一主版本线内保持 ABI 稳定性。`process_helper` 静态链接针对发布版默认 CEF 头文件编译的 `libcef_dll_wrapper.a`。当开发者使用不同的 CEF 版本时，C API 必须兼容。跨主版本时，可能会出现破坏性更改。

## 每周 CEF 版本检查

`.github/workflows/cef-check.yml` 每周运行（周一 09:00 UTC），也可以手动触发。它运行 `package/scripts/check-latest-cef.ts`，该脚本：

1. 获取 `https://cef-builds.spotifycdn.com/linux64_builds_index.json`
2. 找到最新的稳定版本
3. 与 `build.ts` 中的 `CEF_VERSION` 进行比较
4. 如果不同则发出 `::notice` 注解

## 更新 CEF 版本

1. 更新 `package/build.ts` 中的 `CEF_VERSION` 和 `CHROMIUM_VERSION`
2. 在本地删除 `vendors/cef/`（或 `.cef-version` 标记——过时检测会自动清理它）
3. 运行 `bun build.ts` ——它将下载新的 CEF，重新构建 `libcef_dll_wrapper.a` 和 `process_helper`
4. 使用 kitchen 应用测试（从 `package/` 运行 `bun dev`）
5. 发布工作流的 CEF vendor 缓存键包含版本号，因此 CI 将在下次发布时自动重新下载和重新构建

## 文件参考

| 文件 | 角色 |
|------|------|
| `package/build.ts` | `CEF_VERSION`/`CHROMIUM_VERSION` 常量，`vendorCEF()`，`copyToDist()` |
| `package/src/cli/index.ts` | `CEF_HELPER_*` 路径常量，`downloadAndExtractCustomCEF()`，`ensureCEFDependencies()` |
| `package/scripts/package-release.js` | 从 `dist/` 和 `bin/` 创建 3 个 tarball |
| `package/scripts/check-latest-cef.ts` | 查询 Spotify CDN 获取最新的稳定 CEF 版本 |
| `.github/workflows/release.yml` | 带有 CEF 和 process_helper 缓存的构建 + 发布工作流 |
| `.github/workflows/cef-check.yml` | 每周 CEF 版本检查 |
| `package/src/native/macos/cef_process_helper_mac.cc` | macOS process_helper 源代码 |
| `package/src/native/win/cef_process_helper_win.cpp` | Windows process_helper 源代码 |
| `package/src/native/linux/cef_process_helper_linux.cpp` | Linux process_helper 源代码 |
| `package/src/native/linux/cef_loader.{h,cpp}` | Linux 基于 dlopen 的 CEF 加载 |