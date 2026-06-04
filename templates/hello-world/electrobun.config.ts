import type { ElectrobunConfig } from "@pori15/electrobun-rust";

export default {
	app: {
		name: "hello-world",
		identifier: "helloworld.electrobun.dev",
		version: "0.0.1",
	},
	build: {
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
