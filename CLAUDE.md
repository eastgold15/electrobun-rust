Electrobun 开发规范（Claude 版）
编译与运行 Electrobun
重要：编译命令
禁止直接在 bin 目录或 node_modules 目录下运行 Electrobun。正确的编译、运行方式如下：
1. 进入 package 目录执行命令：
  - bun dev：以开发模式编译并运行示例应用
  - bun dev:canary：以测试版模式编译示例应用
2. 编译执行流程
  - 所有编译命令必须在 package 目录下执行
  - 编译流程会自动完成以下操作：
    - 编译原生封装层
    - 编译 TypeScript 代码
    - 编译命令行工具（CLI）
    - 切换至 kitchen 目录，完成应用编译并启动运行
项目目录结构
- /package：Electrobun 主包源码
- /kitchen：测试示例应用（综合演示项目）
- /package/src/cli：命令行工具源码
- /package/src/extractor：自解压程序源码（基于 Zig 语言开发）
- /package/src/native：各平台原生封装代码