
# 注释使用中文

## Shell 规范

**强制使用 Bash（Git Bash）语法，禁止使用 PowerShell/cmd.exe 语法。**

原因：
- Bash 命令简短，节省 token
- Git Bash 已安装在 `C:\Program Files\Git\bin\bash.exe`
- 本项目构建脚本（cargo build、bun dev）均为 POSIX 风格

规则：
- 所有终端命令使用 POSIX 语法：`ls`、`rm`、`cat`、`cd`、`&&`、`|`、`$HOME`
- 路径使用 MSYS 风格：`/c/Users/...` 或 `L:/Documents/...`
- 禁止使用 PowerShell 内置命令：`Get-ChildItem`、`Select-String`、`$env:VAR`
- 禁止使用 cmd 语法：`dir`、`type`、`%VAR%`
- 环境变量引用用 `$VAR` 而非 `$env:VAR`

## 代码重构规范

**代码搜索与批量重构优先使用 ast-grep（CLI 命令 `ast-grep`，已安装 v0.43.0）。**

原因：
- ast-grep 基于 AST（抽象语法树）匹配，不理解语法的纯文本替换（sed/Python）容易误伤
- 例如 `println!` → `info!` 的文本替换会把 `eprintln!` 误改成 `einfo!`，而 ast-grep 不会

用法：
```bash
# 搜索
ast-grep run -p 'println!($$$)' -l rust <路径>

# 批量替换（-U 表示不确认直接应用）
ast-grep run -p 'println!($$$)' -r 'info!($$$)' -l rust -U <路径>
```

## AI 工具使用规则

AI 助手（Claude/Hermes）在操作本项目时允许使用的工具和命令：

### 允许的操作
- 读写文件（write_file / read_file）
- 搜索文件内容（search_files）
- 运行 Rust 构建命令：`cargo build --workspace`、`cargo fmt`、`cargo clippy`
- 运行 TypeScript/Bun 构建命令：`bun dev`、`bun run build`
- 安装/更新依赖（cargo add / bun add）
- 编辑 Cargo.toml、package.json、tsconfig.json 等配置文件
- Git 操作（commit、push、pull、status）
- 运行测试（cargo test、bun test）

### 禁止的行为
- 不可删除未被明确要求删除的文件
- 不可修改 `.git/config`、SSH 密钥等安全敏感文件
- 不可运行未经确认的 `rm -rf` 等破坏性命令
- 不可修改 `/package/vendors/`、`node_modules`、`target/` 内的文件
- 不可在未告知的情况下切换 git 分支或 force push

### 执行原则
- 执行完每个操作后必须报告实际结果（成功/失败/输出），不得伪造输出
- 遇到报错先尝试修复，无法修复时如实报告阻塞原因
- 完成一个任务前必须实际运行代码验证，不能只写代码不运行

## 日志规范

**强制使用 `tracing` crate 进行日志输出，禁止直接使用 `println!` / `eprintln!`。**

原因：
- `tracing` 输出带时间戳、级别、颜色，可筛选
- clippy lint `print_stdout` / `print_stderr` 会 warn 裸输出
- 生产环境和开发环境可通过环境变量 `RUST_LOG` 控制日志级别

### 用法

**程序入口 main 函数第一行初始化：**
```rust
tracing_subscriber::fmt::init();
```

**导入宏：**
```rust
use tracing::{error, info, warn, debug};
```

**日志级别对应关系：**
| 原写法 | 替换为 | 说明 |
|--------|--------|------|
| `println!(...)` | `info!(...)` | 普通信息 |
| `eprintln!(...)` | `error!(...)` | 错误信息 |
| `eprintln!("警告: ...")` | `warn!(...)` | 警告 |
| `println!("调试: ...")` | `debug!(...)` | 调试信息 |

### 依赖配置

所有 crate 统一使用 workspace 依赖：
```toml
# Cargo.toml (workspace 根)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 各 crate Cargo.toml
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

## 依赖管理

### Rust 依赖
- 使用 workspace 级 `[workspace.dependencies]` 统一管理版本
- 新增依赖优先加在 workspace 根 Cargo.toml，各 crate 引用 `workspace = true`
- 更新依赖用 `cargo upgrade` 或手动改根 Cargo.toml 版本号
- 定期运行 `cargo audit` 检查安全漏洞

### TypeScript 依赖
- 使用 Bun workspace 管理：`bun add <pkg> -d`（dev 依赖加 `-d`）
- devDependencies 和 dependencies 要区分清楚
- 避免锁定过细版本，用 `^` 范围



## Git 提交规范

可选建议，推荐但不强制：

```
feat(core): 添加窗口置顶功能
fix(launcher): 修复子进程退出时资源泄漏
chore(deps): 更新 wry 到 0.55.1
refactor(extractor): 提取 ZSTD 解压为独立函数
docs(readme): 更新编译说明
```

类型：`feat` | `fix` | `chore` | `refactor` | `docs` | `test` | `style`
范围：对应 crate 或模块名

## 命名规范

### Rust 命名规范（遵循 Rust API Guidelines）

| 类型 | 规范 | 当前项目情况 |
|------|------|-------------|
| crate 名 | snake_case | ✅ `electrobun-launcher` 正确 |
| 文件名 | snake_case | ✅ `main.rs`, `asar.rs` 正确 |
| 枚举/结构体 | UpperCamelCase | ✅ |
| 函数/方法 | snake_case | ✅ |
| 常量 | SCREAMING_SNAKE | ✅ |
| 类型变量 | 短 UpperCamelCase（T, E, Item） | ✅ |

### TypeScript 命名规范（参考 Google TS Style Guide）

| 标识符 | 规范 |
|--------|------|
| 变量/函数 | camelCase |
| 类型/接口 | PascalCase |
| 枚举值 | PascalCase 或 UPPER_CASE（保持项目一致） |
| 私有属性 | #privateField（真正私有）或 _privateField（约定） |
| 文件命名 | kebab-case（如 webview-manager.ts） |
| 测试文件 | *.test.ts 或 *.spec.ts |

## 跨语言交互规范

### Rust → TS（通过 CLI/进程通信）
- 序列化统一用 serde_json
- 错误返回格式统一：`{ "ok": false, "error": "message" }`
- 成功返回格式：`{ "ok": true, "data": ... }`

### Bun FFI 调用 Rust 原生层
- Rust 公开接口用 `#[no_mangle] extern "C" fn` 导出
- 命名前缀统一：`electrobun_`（如 `electrobun_create_window`）
- 字符串传递用 `*const c_char` + 长度，避免 CString 内存管理问题
- 错误码统一用 enum 映射到 i32

## Karpathy Guidelines

LLM 编程行为准则，源自 [Andrej Karpathy 的观察](https://x.com/karpathy/status/2015883857489522876)。

**权衡：** 这些准则偏向谨慎而非速度。琐碎任务可凭判断灵活处理。

### 1. 先思考再写代码

**不要猜测。不要隐藏困惑。把权衡摆到台面上。**

实现之前：
- 明确说出你的假设。不确定就问。
- 如果有多种解读，提出来——别闷声选一个。
- 如果有更简单的方案，说出来。必要时提出反对。
- 有不清楚的地方，停下来。指出哪里困惑。问。

### 2. 简单至上

**解决问题的最小代码量。不做投机性预留。**

- 不加需求之外的功能。
- 不为只用一次的东西做抽象。
- 不加没被要求的"灵活性"或"可配置性"。
- 不为不可能发生的场景写错误处理。
- 如果写了 200 行但 50 行就能搞定，重写它。

扪心自问：*"资深工程师会觉得这写复杂了吗？"* 如果会，就简化。

### 3. 外科手术式改动

**只碰必须碰的。只清理自己造成的乱摊子。**

编辑现有代码时：
- 不要顺手"改进"旁边的代码、注释或格式。
- 不要重构没坏的东西。
- 匹配已有的代码风格（即使你个人更喜欢另一种）。
- 如果发现无关的死代码——提一句，不要删。

你的改动造成遗留时：
- 删掉被你改动的代码弄成未使用的 import/变量/函数。
- 不要删本来就存在的死代码（除非被要求）。

验证标准：每行改动的代码都应该能直接追溯到用户的请求。

### 4. 目标驱动执行

**定义成功标准。循环直到验证通过。**

把任务转化为可验证的目标：
- "加验证" → "先写无效输入的测试，再让它们通过"
- "修 Bug" → "先写复现 Bug 的测试，再让测试通过"
- "重构 X" → "确保重构前后测试都通过"

多步骤任务给出简要计划：
```
1. [步骤] → 验证：[检查点]
2. [步骤] → 验证：[检查点]
3. [步骤] → 验证：[检查点]
```

强的成功标准让你能独立循环验证。弱的标准（"让它能跑"）需要不断确认。

---

## 项目架构（由 codegraph 分析生成）

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    应用层 (apps/ + templates/)                │
│   kitchen(演示) · 19 个项目模板 (React/Vue/Svelte/Solid...)   │
├─────────────────────────────────────────────────────────────┤
│                   TypeScript SDK (packages/electrobun-rust)  │
│  ┌──────────────────┐  ┌──────────────────┐                 │
│  │  Bun 端 (core/bun/) │ │浏览器端 (core/browser/)│              │
│  │ BrowserWindow     │  │ Electroview RPC   │                │
│  │ Tray · WebGPU     │  │ <electrobun-webview>              │
│  │ ApplicationMenu   │  │ <electrobun-wgpu>                 │
│  └────────┬─────────┘  └──────────────────┘                 │
│           │ FFI (bun:ffi)                                   │
├───────────┼─────────────────────────────────────────────────┤
│           ▼                                                  │
│       Rust 原生层 (crates/)                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              electrobun-core                          │   │
│  │  window · webview · tray · dialog · clipboard         │   │
│  │  session · shortcuts · display · wgpu · notifications │   │
│  │  crypto · file_ops · transport                        │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌───────────┐ ┌──────────────┐ ┌─────────────┐            │
│  │  launcher  │ │  extractor   │ │ build-tools │            │
│  │ (启动器)    │ │ (自解压)      │ │ (构建工具)    │            │
│  └───────────┘ └──────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│                CEF (Chromium Embedded Framework)             │
│                    (WebView 渲染引擎)                         │
└─────────────────────────────────────────────────────────────┘
```

### 分层详解

#### 1. Rust 原生层 — `crates/`

| Crate | 职责 | 关键模块 |
|-------|------|---------|
| **electrobun-core** | 核心原生库，导出 FFI 接口给 Bun 调用 | `window.rs`(84符号), `webview.rs`(43), `tray.rs`(24), `api/mod.rs`, `transport.rs`(26), `clipboard.rs`, `dialog.rs`, `display.rs`, `shortcuts.rs`, `session.rs`, `notifications.rs`, `wgpu.rs`(37), `crypto.rs`, `file_ops.rs`, `types.rs`, `error.rs` |
| **electrobun-launcher** | 应用启动器（Rust + TS 混合） | `main.rs`(30符号), `main.ts`(10) |
| **electrobun-extractor** | 自解压程序，解压 app bundle 并安装 | `main.rs`(41符号) |
| **electrobun-build-tools** | 构建工具合集 | `asar.rs`, `bsdiff.rs`, `bspatch.rs`, `zstd.rs` |
| **electrobun-macros** | 过程宏，自动生成 FFI 绑定和 TS 客户端代码 | `ffi_gen.rs`, `ipc.rs`(22符号), `ts_gen.rs`(11), `stream.rs` |

#### 2. TypeScript SDK — `packages/electrobun-rust/cli/src/core/`

**Bun 端 (`core/bun/`)** — 在 Bun 运行时中运行，管理窗口和应用生命周期：

| 模块 | 文件 | 说明 |
|------|------|------|
| 核心类 | `BrowserWindow.ts` (80 符号) | 窗口管理：创建/关闭/大小/位置/全屏/置顶 |
| | `BrowserView.ts` (72 符号) | WebView 管理 |
| | `GpuWindow.ts` (54 符号) | GPU 窗口（WebGPU） |
| | `WGPUView.ts` (33 符号) | WGPU 视图 |
| | `Tray.ts` (36 符号) | 系统托盘 |
| | `Updater.ts` (19 符号) | 应用更新 |
| | `Utils.ts` (67 符号) | 工具函数 |
| | `ApplicationMenu.ts` (20 符号) | 应用菜单（macOS） |
| | `ContextMenu.ts` (19 符号) | 右键菜单 |
| FFI 通信 | `proc/` | 模块化 FFI 调用（`core-lib.ts`、`ffi-impl.ts`、`ffi-symbols.ts`、`platform.ts`、`wgpu.ts`） |
| 事件系统 | `events/` | ApplicationEvents、windowEvents、webviewEvents、trayEvents |
| 预加载脚本 | `preload/webviewTag.ts` (56 符号) | 自定义 HTML 元素 `<electrobun-webview>` |
| | `preload/wgpuTag.ts` (30 符号) | 自定义 HTML 元素 `<electrobun-wgpu>` |
| | `preload/dragRegions.ts` | 拖拽区域支持 |
| | `preload/encryption.ts` | 加密模块 |
| 入口 | `index.ts` (55 符号) | 导出所有模块，`Electrobun` 命名空间 |

**浏览器端 (`core/browser/`)** — 在前端页面中运行，提供 RPC 通信：

| 模块 | 说明 |
|------|------|
| `Electroview` | 浏览器端核心，通过 RPC 与 Bun 端通信 |
| `<electrobun-webview>` | 自定义 HTML 元素，嵌入子 WebView |
| `<electrobun-wgpu>` | 自定义 HTML 元素，嵌入 WGPU 渲染 |
| `builtinrpcSchema.ts` | 内建 RPC Schema |

**API 层 (`core/api/`)** — 提供给前端应用的统一 API：

```
ElectrobunAPI
├── app          — 应用信息、退出、打开外部链接
├── clipboard    — 剪贴板读写
├── dialog       — 文件对话框、消息弹窗
├── notification — 系统通知
├── shell        — 执行命令、打开文件
├── tray         — 系统托盘
├── webview      — WebView 控制
└── window       — 窗口控制
```

**生成代码 (`core/generated/`)** — 由 `electrobun-macros` 自动生成：
- 每个 API 模块对应一个 `*Client.ts`（如 `WindowAPIClient.ts`、`TrayAPIClient.ts`）
- 每个客户端通过 `bun:ffi` 的 `dlopen` 加载 `electrobun_core` 动态库
- 通信方式：参数 JSON 序列化 → C 字符串传递 → Rust 反序列化 → 执行 → JSON 序列化返回

#### 3. 跨语言通信机制

```
Bun (TS)                  Rust (electrobun-core)
───────                   ─────────────────────
bun:ffi dlopen()  ─────→  #[no_mangle] extern "C"
  │                            │
  ├─ JSON.stringify(params)    │
  ├─ → CString → *const c_char │
  │                        ←───┘
  │                   serde_json::from_slice
  │                        │
  │                    [执行业务逻辑]
  │                        │
  └─ ← CString ←──── serde_json::to_string
     JSON.parse(result)
```

- 所有 Rust 导出函数命名前缀 `electrobun_`（如 `electrobun_create_window`）
- 参数和返回值统一 JSON 序列化
- 错误类型统一 `{ "ok": false, "error": "message" }` 格式
- `electrobun-macros` 的 `ffi_gen` 和 `ts_gen` 宏自动生成 FFI 绑定和 TS 客户端

#### 4. 构建流程

```
bun dev
  ↓
编译 Rust crate (cargo build)
  ├─ electrobun-core → electobun_core.dll/.dylib/.so
  ├─ electrobun-launcher → launcher 可执行文件
  ├─ electrobun-extractor → 自解压程序
  └─ electrobun-build-tools → asar/bsdiff/bspatch/zstd 工具
  ↓
编译 TypeScript (Bun build)
  ↓
编译 CLI
  ↓
切换到 apps/kitchen → 完成应用编译并启动运行
```

#### 5. 项目模板 — `templates/`

19 个模板，覆盖主流前端框架和使用场景：

| 类别 | 模板 |
|------|------|
| 基础 | `hello-world`, `vanilla-vite`, `tailwind-vanilla` |
| 前端框架 | `react-tailwind-vite`, `vue`, `svelte`, `solid`, `angular` |
| 多窗口 | `multi-window`, `multitab-browser` |
| WebGPU | `wgpu`, `wgpu-babylon`, `wgpu-mlp`, `wgpu-threejs` |
| 应用 | `notes-app`, `sqlite-crud`, `photo-booth`, `tray-app`, `bunny` |

### 关键架构决策

1. **双端架构**：Bun 端管理原生窗口/系统能力，浏览器端通过 RPC 通信，类似 Electron 的 main/renderer 进程模型
2. **CEF 渲染**：使用 Chromium Embedded Framework 而非系统 WebView，确保跨平台一致性
3. **FFI 通信**：通过 `bun:ffi` 直接调用 Rust 动态库，无需额外的 IPC 层
4. **代码生成**：`electrobun-macros` 过程宏自动生成 Rust ↔ TypeScript 的 FFI 胶水代码


