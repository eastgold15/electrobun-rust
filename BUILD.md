# 构建系统

本文档描述了 Electrobun 的构建系统和跨平台编译方法。

## 概述

Electrobun 使用自定义构建系统（`build.ts`）来处理：
- 依赖打包（Bun、rust、CEF、WebView2）
- 为每个平台构建原生包装器
- 创建分发包

## 平台特定的原生包装器

### macOS
- 单个 `libNativeWrapper.dylib`，弱链接到 CEF 框架
- 使用 `-weak_framework 'Chromium Embedded Framework'` 实现可选的 CEF 支持
- 当 CEF 未捆绑时优雅地回退到 WebKit

### Windows  
- 单个 `libNativeWrapper.dll`，运行时检测 CEF
- 在构建时同时链接 WebView2 和 CEF 库
- 使用运行时检查来确定使用哪个 webview 引擎

### Linux
**双二进制文件方法** - Linux 构建创建两个独立的原生包装器二进制文件：

#### `libNativeWrapper.so`（仅 GTK）
- 大小：约 1.46MB
- 依赖：仅 WebKitGTK、GTK+3、AppIndicator
- 没有链接 CEF 依赖
- 当 electrobun.config 中 `bundleCEF: false` 时使用

#### `libNativeWrapper_cef.so`（启用 CEF）  
- 大小：约 3.47MB
- 依赖：WebKitGTK、GTK+3、AppIndicator + CEF 库
- 提供完整的 CEF 功能
- 当 electrobun.config 中 `bundleCEF: true` 时使用

#### 为什么使用双二进制文件？

与 macOS 和 Windows 不同，Linux 对于共享库没有可靠的弱链接机制。硬链接 CEF 库会导致在未捆绑 CEF 时出现 `dlopen` 失败。双二进制文件方法提供了：

1. **更小的包体积** - 开发者可以发布轻量级应用，无需 CEF 开销
2. **灵活性** - 同一代码库同时支持系统 WebKitGTK 和 CEF 渲染
3. **可靠性** - 没有运行时链接失败或未定义符号问题

#### CLI 二进制文件选择

Electrobun CLI 根据 `bundleCEF` 设置自动复制适当的二进制文件：

```typescript
const useCEF = config.build.linux?.bundleCEF;
const nativeWrapperSource = useCEF 
  ? PATHS.NATIVE_WRAPPER_LINUX_CEF 
  : PATHS.NATIVE_WRAPPER_LINUX;
```

两个二进制文件都包含在分发的 `electrobun` npm 包中，确保开发者可以在不重新编译的情况下切换 CEF 支持。

## 构建命令

所有命令都在 `/package` 目录下运行：

```bash
cd electrobun/package

# 完整构建所有平台
bun build.ts

# 开发构建，附带 kitchen sink 测试应用
bun dev

# 发布构建
bun build.ts --release

# CI 构建
bun build.ts --ci
```

## 架构支持

- **macOS**：ARM64（Apple Silicon）、x64（Intel） 
- **Windows**：仅 x64（ARM Windows 用户通过自动仿真运行）
- **Linux**：x64、ARM64

### Windows 架构说明

Windows 构建在 ARM VM 上创建，但目标是 x64 架构。x64 和 ARM Windows 用户使用相同的 x64 二进制文件：
- **x64 Windows**：原生运行
- **ARM Windows**：通过自动 Windows 仿真层运行

这种方法简化了分发，同时保持了跨 Windows 架构的兼容性。

构建系统自动检测主机架构并下载相应的依赖项。