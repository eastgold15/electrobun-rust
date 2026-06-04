#!/usr/bin/env bun

/**
 * Electrobun CLI — 轻量入口
 *
 * 实际实现在 src/cli/ 目录下按职责分模块管理：
 *   - src/cli/index.ts         命令路由
 *   - src/cli/config.ts        配置管理
 *   - src/cli/platform-paths.ts 平台路径
 *   - src/cli/build.ts         构建逻辑
 *   - src/cli/dev.ts           开发/运行
 *   - src/cli/commands/init.ts 初始化命令
 *   - src/cli/downloads/       Bun/CEF/WGPU/Core 下载管理
 *   - src/cli/utils/           工具函数
 */

/**
 * Electrobun CLI — main entry point
 *
 * Routes CLI commands to the appropriate handler modules.
 */


// CLI argument parsing — 找第一个已知命令词
// 兼容：electrobun build、bun index.ts build、node index.ts build
const COMMANDS = new Set(["init", "build", "dev", "run"]);
let commandArg = process.argv.find((arg) => COMMANDS.has(arg)) || "build";

/**
 * Find and load electrobun config from the project root
 */
async function getConfig(): Promise<import("./config").DefaultConfig> {
  const { findConfigFile, defaultConfig } = await import("./config");
  const configFile = findConfigFile();

  if (!configFile) {
    console.log("No electrobun.config.ts found, using defaults");
    return defaultConfig;
  }

  try {
    const userConfig = await import(configFile);
    // Merge user config with defaults
    const merged = {
      ...defaultConfig,
      ...userConfig.default,
      build: {
        ...defaultConfig.build,
        ...(userConfig.default?.build || {}),
        mac: {
          ...defaultConfig.build.mac,
          ...(userConfig.default?.build?.mac || {}),
        },
        win: {
          ...defaultConfig.build.win,
          ...(userConfig.default?.build?.win || {}),
        },
        linux: {
          ...defaultConfig.build.linux,
          ...(userConfig.default?.build?.linux || {}),
        },
      },
    };
    return merged;
  } catch (error) {
    console.error(`Failed to load config from ${configFile}:`, error);
    return defaultConfig;
  }
}

// ── Command dispatch ──────────────────────────────────────────
(async () => {
  if (commandArg === "init") {
    const { initCommand } = await import("./commands/init");
    await initCommand();
  } else if (commandArg === "build") {
    const config = await getConfig();
    const envArg =
      process.argv.find((arg) => arg.startsWith("--env="))?.split("=")[1] || "";
    const buildEnvironment: "dev" | "canary" | "stable" = ["dev", "canary", "stable"].includes(envArg)
      ? (envArg as "dev" | "canary" | "stable")
      : "dev";
    try {
      const { runBuild } = await import("./build");
      await runBuild(config, buildEnvironment);
    } catch (error) {
      console.error("Build failed:", error);
      process.exit(1);
    }
  } else if (commandArg === "run") {
    const config = await getConfig();
    const { runAppWithSignalHandling } = await import("./dev");
    await runAppWithSignalHandling(config);
  } else if (commandArg === "dev") {
    const config = await getConfig();
    const watchMode = process.argv.includes("--watch");

    if (watchMode) {
      const { runDevWatch } = await import("./dev");
      await runDevWatch(config);
    } else {
      try {
        const { runBuild } = await import("./build");
        await runBuild(config, "dev");
      } catch (error) {
        console.error("Build failed:", error);
        process.exit(1);
      }
      const { runAppWithSignalHandling } = await import("./dev");
      await runAppWithSignalHandling(config);
    }
  }
})();
