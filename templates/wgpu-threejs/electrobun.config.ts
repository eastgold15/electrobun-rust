import type { ElectrobunConfig } from "@pori15/electrobun-rust";

export default {
	app: {
		name: "three-physics",
		identifier: "three-physics.electrobun.dev",
		version: "0.0.1",
	},
	build: {
		useAsar: false,
		bun: {
			entrypoint: "src/bun/index.ts",
		},
		views: {
			mainview: {
				entrypoint: "src/mainview/index.ts",
			},
		},
		copy: {
			"src/mainview/index.html": "views/mainview/index.html",
			"src/mainview/index.css": "views/mainview/index.css",
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
