import { join } from "path";
import { existsSync } from "fs";

export type FileAssociation = {
  ext: string[];
  name: string;
  role?: "Editor" | "Viewer" | "Shell" | "None";
  icon?: string;
};

export const _commandDefaults = {
  init: {
    projectRoot: process.cwd(),
    config: "electrobun.config",
  },
  build: {
    projectRoot: process.cwd(),
    config: "electrobun.config",
  },
  dev: {
    projectRoot: process.cwd(),
    config: "electrobun.config",
  },
};

export interface DefaultConfig {
  app: {
    name: string;
    identifier: string;
    version: string;
    description?: string;
    urlSchemes?: string[];
    fileAssociations?: FileAssociation[];
  };
  build: {
    buildFolder: string;
    artifactFolder: string;
    mainProcess: string;
    useAsar: boolean;
    asarUnpack?: string[];
    cefVersion?: string;
    wgpuVersion?: string;
    bunVersion?: string;
    bunnyBun?: string;
    locales?: string[] | "*";
    mac: {
      codesign: boolean;
      createDmg: boolean;
      notarize: boolean;
      bundleWGPU: boolean;
      entitlements: Record<string, boolean | string>;
      icons?: string;
      defaultRenderer?: "native";
      chromiumFlags?: Record<string, string | boolean>;
    };
    win: {
      bundleWGPU: boolean;
      icon?: string;
      defaultRenderer?: "native";
      chromiumFlags?: Record<string, string | boolean>;
    };
    linux: {
      bundleWGPU: boolean;
      icon?: string;
      defaultRenderer?: "native";
      chromiumFlags?: Record<string, string | boolean>;
    };
    bun: {
      entrypoint: string;
    };
    views?: Record<string, { entrypoint: string; [key: string]: unknown }>;
    copy?: Record<string, string>;
    watch?: string[];
    watchIgnore?: string[];
  };
  runtime: Record<string, unknown>;
  scripts: {
    preBuild: string;
    postBuild: string;
    postWrap: string;
    postPackage: string;
  };
  release: {
    baseUrl: string;
    generatePatch: boolean;
  };
}

// Default values merged with user's electrobun.config.ts
export const defaultConfig: DefaultConfig = {
  app: {
    name: "MyApp",
    identifier: "com.example.myapp",
    version: "0.1.0",
    description: "",
    urlSchemes: undefined,
    fileAssociations: undefined,
  },
  build: {
    buildFolder: "build",
    artifactFolder: "artifacts",
    mainProcess: "bun",
    useAsar: false,
    asarUnpack: undefined,
    cefVersion: undefined,
    wgpuVersion: undefined,
    bunVersion: undefined,
    bunnyBun: undefined,
    locales: undefined,
    mac: {
      codesign: false,
      createDmg: true,
      notarize: false,
      bundleWGPU: false,
      entitlements: {
        "com.apple.security.cs.allow-jit": true,
        "com.apple.security.cs.allow-unsigned-executable-memory": true,
        "com.apple.security.cs.disable-library-validation": true,
      },
      icons: "icon.iconset",
      defaultRenderer: undefined,
      chromiumFlags: undefined,
    },
    win: {
      bundleWGPU: false,
      icon: undefined,
      defaultRenderer: undefined,
      chromiumFlags: undefined,
    },
    linux: {
      bundleWGPU: false,
      icon: undefined,
      defaultRenderer: undefined,
      chromiumFlags: undefined,
    },
    bun: {
      entrypoint: "src/bun/index.ts",
    },
    views: undefined,
    copy: undefined,
    watch: undefined,
    watchIgnore: undefined,
  },
  runtime: {},
  scripts: {
    preBuild: "",
    postBuild: "",
    postWrap: "",
    postPackage: "",
  },
  release: {
    baseUrl: "",
    generatePatch: true,
  },
};

/**
 * Find TypeScript ESM config file
 */
export function findConfigFile(): string | null {
  const projectRoot = process.cwd();
  const configFile = join(projectRoot, "electrobun.config.ts");
  return existsSync(configFile) ? configFile : null;
}
