import type { ElectrobunConfig } from "@pori15/electrobun-rust";

export default {
  app: {
    name: "webgpu-babylon",
    identifier: "webgpu-babylon.electrobun.dev",
    version: "0.0.1",
  },
  build: {
    useAsar: false,
    bun: {
      entrypoint: "src/bun/index.ts",
    },
    copy: {
      "src/assets": "assets",
    },
    mac: {
      bundleWGPU: true,
    },
    linux: {
      bundleWGPU: true,
    },
    win: {
      bundleWGPU: true,
    },
  },
} satisfies ElectrobunConfig;
