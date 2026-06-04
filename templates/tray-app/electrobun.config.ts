import type { ElectrobunConfig } from "@pori15/electrobun-rust";

export default {
	app: {
		name: "tray-app",
		identifier: "trayapp.electrobun.dev",
		version: "0.0.1",
	},
	runtime: {
		exitOnLastWindowClosed: false,
	},
	build: {
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
		},
		mac: {
		},
		linux: {
		},
		win: {
		},
	},
} satisfies ElectrobunConfig;
