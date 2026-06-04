# 集成测试

本目录包含 electrobun 的集成测试，需要完整的 SDK 构建产物（DLL 文件）才能运行。

## 目录结构

- `integration/native/` — Native FFI DLL 加载与符号兼容性测试
  - `native-ffi-loading.test.ts` — 验证 `electrobun_core.dll` 能被 `bun:ffi` 的 `dlopen` 加载

## 前置条件

运行集成测试前需要先构建 SDK：

```bash
cd packages/cli
bun run build:dev
```

## 运行测试

```bash
# 运行所有单元测试（推荐，无需构建）
bun test

# 运行集成测试（需要先构建）
bun test __tests__/integration
```

## 迁移说明

原先本目录包含大量 Zig 时代的测试文件，已在项目重构中清理。
集成测试已迁移到 `apps/kitchen/` 中，使用 `TestExecutor` 框架运行。
纯函数单元测试迁移到 `packages/cli/src/` 中各模块旁，使用 `bun:test` 运行。
