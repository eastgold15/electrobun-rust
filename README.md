<p align="center">
  <a href="https://electrobun.dev"><img src="https://github.com/blackboardsh/electrobun/assets/75102186/8799b522-0507-45e9-86e3-c3cfded1aa7c" alt="Logo" height=170></a>
</p>

<h1 align="center">Electrobun</h1>

<div align="center">
  使用模板快速开始 <br />
  <code><strong>npx electrobun init</strong></code>   
</div>



## 什么是 Electrobun？

Electrobun 旨在成为一个完整的**开箱即用解决方案**，用于构建、更新和发布超快速、小巧且跨平台的桌面应用程序（使用 TypeScript 编写）。
底层使用 <a href="https://bun.sh">bun</a> 来执行主进程并打包 webview 的 TypeScript 代码，同时拥有用 ObjC、C++ 编写的原生绑定，以及多个用 <a href="https://ziglang.org/">zig</a> 编写的核心部分。

访问 <a href="https://docs.electrobunny.ai/electrobun/">https://docs.electrobunny.ai/electrobun/</a> 查看 API 文档、指南等更多内容。

通过 npm 使用它。

不要错过我们的：
- 使用 ZSTD 压缩的自解压包，使分发包更小，最小可达 16MB
- Zig 优化的 BSDIFF 实现，让您能够发布小到 4KB 的应用更新
- `bundleCEF` 标志，用于捆绑和固定 Chromium，适合那些希望以文件大小换取一致性的用户
- `bundleWGPU` 让您能够使用 Bun TypeScript -> WGPU 来控制原生 GPU 表面，无需 webview
- 我们的 Three.js 和 Babylon.js 适配器，可直接在 Bun 中运行
- 我们的 `<electrobun-webview>` 和 `<electrobun-wpgu>` HTML 元素，让您能够将真正的 OOPIF 和原生 GPU 表面合成到 UI 中
- 还有更多功能。

**项目目标**

- 为主进程和 webview 编写 TypeScript，无需担心其他问题。
- 主进程和 webview 进程之间的隔离，它们之间具有快速、类型化、易于实现的 RPC。
- 小型自解压应用包约 14MB（使用系统 webview 时，大部分是 bun 运行时）
- 更小的应用更新，最小可达 4KB（使用 bsdiff，只下载版本间的微小补丁）
- 在一个紧密集成的工作流中提供您所需的一切，让您能够在 5 分钟内开始编写代码，10 分钟内发布应用。

## 使用 Electrobun 构建的应用
- [24agents](https://github.com/jhsu/24agents) - Hyperprompter
- [act-track-ai](https://github.com/IrdanGu/act-track-ai) - 个人桌面生产力追踪器
- [Agents Council](https://github.com/MrLesk/agents-council) - 用于反馈请求的代理间 MCP 通信工具
- [ai-wrapped](https://github.com/gulivan/ai-wrapped) - Wrapped 风格的桌面仪表板，展示您的 AI 编码代理活动
- [Audio TTS](https://github.com/blackboardsh/audio-tts) - 使用 Qwen3-TTS 进行语音设计、克隆和生成的桌面文本转语音应用
- [aueio-player-desktop](https://github.com/tuomashatakka/aueio-player-desktop) - 美观、简约的跨平台音频播放器
- [bestdiff](https://github.com/tesmond/bestdiff) - 带有曲线连接器的 git diff 检查器
- [BuddyWriter](https://github.com/OxFrancesco/BuddyWriter) - BuddyWriter 桌面和移动应用
- [burns](https://github.com/l3wi/burns) - Smithers 管理器
- [cbx-tool](https://github.com/jebin2/cbx-tool) - 用于阅读和编辑漫画档案（.cbz/.cbr）的桌面应用
- [Co(lab)](https://blackboard.sh/colab/) - 混合 Web 浏览器 + 代码编辑器，专为深度工作设计
- [codlogs](https://github.com/tobitege/codlogs) - 通过 CLI 或桌面应用搜索和导出本地 Codex 会话
- [Codex Agents Composer](https://github.com/MrLesk/codex-agents-composer) - 管理您的 Codex 代理及其技能的桌面应用
- [codex-devtools](https://github.com/gulivan/codex-devtools) - Codex 会话数据的桌面检查器；浏览对话、搜索消息和分析代理活动
- [Deskdown](https://github.com/guarana-studio/deskdown) - 在 20 秒内将任何网页地址转换为桌面应用
- [Dictate](https://github.com/siddhantparadox/dictate) - Windows 听写应用，支持本地和 BYOK 云转录
- [dev-3.0](https://github.com/h0x91b/dev-3.0) - 帮助您在跨项目管理多个 AI 代理时不会迷失方向
- [DOOM](https://github.com/blackboardsh/electrobun-doom) - DOOM 的两种实现方式：bun -> (c doom -> 捆绑 wgpu) 和 (完整 ts 移植 bun -> 捆绑 wgpu)
- [dotlock](https://github.com/tsconfigdotjson/dotlock) - macOS 桌面应用，用于管理您各个项目中的 `.env` 文件
- [electrobun-pdf](https://github.com/GijungKim/electrobun-pdf) - 本地优先的 PDF 和 DOCX 编辑器，用于打开、注释和导出文档，无需离开您的机器
- [electrobun-rms](https://github.com/khanhthanhdev/electrobun-rms) - 快速的 Electrobun 桌面应用模板，包含 React、Tailwind CSS 和 Vite
- [golb](https://github.com/chrisdadev13/golb) - 使用 React、Vite 和 Tailwind 构建的桌面 AI 编码工作空间
- [GOG Achievements GUI](https://github.com/timendum/gog-achievements-gui) - 管理 GOG 成就的桌面应用
- [groov](https://github.com/laurenzcodes/groov) - 桌面音频设备监控器
- [Guerilla Glass](https://github.com/okikeSolutions/guerillaglass) - 开源跨平台创作者工作室，实现快速录制 -> 编辑 -> 交付工作流
- [Marginalia](https://github.com/lars-hoeijmans/Marginalia) - 一个简单的笔记应用
- [MarkBun](https://github.com/xiaochong/markbun) - 快速、美观、类似 Typora 的 Markdown 桌面编辑器
- [md-browse](https://github.com/needle-tools/md-browse) - Markdown 优先的浏览器，将网页转换为干净的 Markdown
- [moop](https://github.com/zrubinrattet/moop/) - 批量图像优化桌面应用，专为网络优化
- [Patchline](https://github.com/adwaithks/Patchline) - 轻量级桌面 Git 客户端，用于读取补丁和行差异，然后暂存和提交更改
- [peekachu](https://github.com/needle-tools/peekachu) - AI 密码管理器；将秘密存储在操作系统密钥链中并清理输出，使 AI 助手永远看不到实际值
- [PiBun](https://github.com/khairold/pibun) - Pi 编码代理的桌面 GUI，具有聊天、终端、Git 集成和插件系统
- [PLEXI](https://github.com/ianjamesburke/PLEXI) - 代理时代的多维终端复用器
- [Prometheus](https://github.com/opensourcectl/prometheus) - 桌面实用工具箱，用于文件清理、文档操作和图像处理
- [Quiver](https://ataraxy-labs.github.io/quiver/) - 桌面应用，用于 GitHub PR 审查、合并冲突解决和 AI 提交消息
- [remotecode.io](https://github.com/samuelfaj/remotecode.io) - 从您的移动设备继续本地 AI 编码会话（Claude Code 或 Codex）
- [sirene](https://github.com/KevinBonnoron/sirene) - 自托管多后端文本转语音平台，支持语音克隆
- [StoryForge](https://github.com/vrrdnt/StoryForge) - Vintage Story 玩家的桌面应用，用于在游戏版本、模组包、服务器和账户之间切换
- [Tensamin Client](https://github.com/Tensamin/Client) - 访问 Tensamin 的 Web、桌面和移动应用
- [tokenpass-desktop](https://github.com/b-open-io/tokenpass-desktop) - 在本地运行 Sigma Identity 栈以实现比特币支持的身份验证的桌面应用
- [typsmthng-desktop](https://github.com/aaditagrawal/typsmthng-desktop) - 实验性桌面打字应用
- [VibesOS](https://github.com/popmechanic/VibesOS) - Claude Code 的 GUI，让轻松 vibe 编码简单、不可破解的应用变得容易
- [VoiceVault](https://github.com/PJH720/VoiceVault) - AI 驱动的录音机，具有转录、摘要和 RAG 搜索功能
- [warren](https://github.com/Loa212/warren) - 开源点对点终端网格，让您能够从任何设备访问您的机器，无需 SSH 密钥或配置文件
- [whatsapp-reminder](https://github.com/FatahChan/whatsapp-reminder) - 管理的定时 WhatsApp 消息

### 视频演示

[![Audio TTS Demo](https://img.youtube.com/vi/Z4dNK1d6l6E/maxresdefault.jpg)](https://www.youtube.com/watch?v=Z4dNK1d6l6E)

[![Co(lab) Demo](https://img.youtube.com/vi/WWTCqGmE86w/maxresdefault.jpg)](https://www.youtube.com/watch?v=WWTCqGmE86w)

[![DOOM Demo](https://github.com/user-attachments/assets/6cc5f04a-6d97-4010-b65f-3f282d32590c)](https://x.com/YoavCodes/status/2028499038148903239?s=20)

## Star 历史

[![Star History Chart](https://api.star-history.com/svg?repos=blackboardsh/electrobun&type=date&legend=top-left&cache=3)](https://www.star-history.com/#blackboardsh/electrobun&type=date&legend=top-left)

## 贡献
Electrobun 是我正在构建的愿景的一部分。我专注于专注和执行。Issues 和 PR 可以用来分享想法，但不要期望我会审查、回复或合并它们。

参与方式：

- 阅读[贡献指南](./CONTRIBUTING.md)
- 在 X 上关注我们以获取更新 <a href="https://twitter.com/BlackboardTech">@BlackboardTech</a> 和 <a href="https://twitter.com/YoavCodes">@YoavCodes</a>，或在 Bluesky 上关注 <a href="https://bsky.app/profile/yoav.codes">@yoav.codes</a>
- 加入 <a href="https://discord.gg/ueKE4tjaCE">Discord</a> 上的讨论
- 创建并参与 Github issues 和讨论
- 告诉我您正在用 Electrobun 构建什么

## 开发设置
使用 Electrobun 构建应用就像用 `npm add electrobun` 更新 package.json 依赖项一样简单，或者通过 `npx electrobun init` 尝试我们的模板之一。

**本节适用于从本地源代码构建 Electrobun 以贡献修复。**

### 前置条件

**macOS：**
- Xcode 命令行工具
- cmake（通过 homebrew 安装：`brew install cmake`）

**Windows：**
- Visual Studio Build Tools 或带有 C++ 开发工具的 Visual Studio
- cmake

**Linux：**
- build-essential 包
- cmake
- webkit2gtk 和 GTK 开发包

在基于 Ubuntu/Debian 的发行版上：`sudo apt install build-essential cmake pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev`

### 首次设置

```bash
git clone --recurse-submodules https://github.com/blackboardsh/electrobun.git
cd electrobun/package
bun install
bun dev:clean
```

> `bun install` 会自动触发 **postinstall** 脚本，下载当前平台的核心二进制文件（~48MB）
> 到 `dist-{OS}-{ARCH}/` 目录。下载完成后，后续构建无需联网。
>
> 如果使用 `file:../../package` 本地链接开发，postinstall 不会运行，CLI 会在首次
> 构建时自动下载并缓存这些文件。

### 开发工作流

```bash
# 所有命令都在 /package 目录下运行
cd electrobun/package

# 修改源代码后
bun dev

# 如果您只更改了 kitchen sink 代码（而不是 electrobun 源代码）
bun dev:rerun

# 如果需要完全重新开始
bun dev:clean
```

### 其他命令

所有命令都在 `/package` 目录下运行：

- `bun dev:canary` - 以 canary 模式构建并运行 kitchen sink
- `bun build:dev` - 以开发模式构建 electrobun
- `bun build:release` - 以发布模式构建 electrobun

### 调试

**macOS：** 使用 `lldb <path-to-bundle>/Contents/MacOS/launcher` 然后 `run` 来调试发布版本

## 平台支持

| 操作系统 | 状态 |
|---|---|
| macOS 14+ | 官方支持 |
| Windows 11+ | 官方支持 |
| Ubuntu 22.04+ | 官方支持 |
| 其他 Linux 发行版（gtk3, webkit2gtk-4.1） | 社区支持 |
| Raspberry Pi | 非官方分支：[kortexa-ai/electrobun (linux-wpe)](https://github.com/kortexa-ai/electrobun/tree/kortexa/linux-wpe) — 关注作者 [@francip](https://x.com/francip/status/2050149256053539059?s=20) |
