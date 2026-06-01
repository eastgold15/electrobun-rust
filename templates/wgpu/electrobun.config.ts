import type { ElectrobunConfig } from "@pori15/electrobun-rust";

export default {
	app: {
		name: "wgpu",
		identifier: "wgpu.electrobun.dev",
		version: "0.0.1",
	},
	build: {
		bun: {
			entrypoint: "src/bun/index.ts",
		},
		mac: {
			bundleCEF: false,
			bundleWGPU: true,
		},
		linux: {
			bundleCEF: false,
			bundleWGPU: true,
		},
		win: {
			bundleCEF: false,
			bundleWGPU: true,
		},
	},
} satisfies ElectrobunConfig;
