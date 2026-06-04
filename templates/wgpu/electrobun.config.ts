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
