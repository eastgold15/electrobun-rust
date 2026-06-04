import { readFileSync, writeFileSync, existsSync, readdirSync } from "fs";
import { resolve, join, dirname } from "path";
import { fileURLToPath } from "url";
import { WGPU_LIB_FILENAMES } from "./wgpu-shared";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");
function resolveHeaderPath() {
	const base = resolve(root, "vendors", "wgpu");
	if (!existsSync(base)) return null;

	const entries = readdirSync(base, { withFileTypes: true });
	for (const entry of entries) {
		if (!entry.isDirectory()) continue;
		const candidate = join(base, entry.name, "include", "dawn", "webgpu.h");
		if (existsSync(candidate)) return candidate;
	}

	return null;
}

const headerPath = resolveHeaderPath();
if (!headerPath) {
	throw new Error("Could not find vendors/wgpu/*/include/dawn/webgpu.h");
}

const header = readFileSync(headerPath, "utf8");

const callbackTypes = new Set();
const callbackRegex = /typedef\s+[^;]*\(\s*\*\s*(WGPU[A-Za-z0-9_]+)\s*\)\s*\(/g;
let cbMatch;
while ((cbMatch = callbackRegex.exec(header))) {
	callbackTypes.add(cbMatch[1]);
}

const typedefMap = new Map();
const scalarTypedefRegex = /typedef\s+(uint32_t|uint64_t|int32_t|int64_t|size_t|intptr_t|uintptr_t|float|double|int|unsigned int)\s+(WGPU[A-Za-z0-9_]+)\s*;/g;
let tdMatch;
while ((tdMatch = scalarTypedefRegex.exec(header))) {
	typedefMap.set(tdMatch[2], tdMatch[1]);
}

const enumRegex = /typedef\s+enum\s+WGPU[A-Za-z0-9_]+\s*\{[\s\S]*?\}\s*(WGPU[A-Za-z0-9_]+)(?:\s+WGPU_[A-Z_]+)*\s*;/g;
let enumMatch;
while ((enumMatch = enumRegex.exec(header))) {
	if (!typedefMap.has(enumMatch[1])) {
		typedefMap.set(enumMatch[1], "uint32_t");
	}
}

const aliasTypedefRegex = /typedef\s+(WGPU[A-Za-z0-9_]+)\s+(WGPU[A-Za-z0-9_]+)\s*;/g;
let aliasMatch;
while ((aliasMatch = aliasTypedefRegex.exec(header))) {
	const from = aliasMatch[1];
	const to = aliasMatch[2];
	if (typedefMap.has(from) && !typedefMap.has(to)) {
		typedefMap.set(to, typedefMap.get(from));
	}
}

const structValueMap = new Map([
	["WGPUFuture", "uint64_t"],
	["WGPUStringView", "ptr"],
]);

const typeToFFI = (type, isReturn = false) => {
	let t = type
		.replace(/\bconst\b/g, "")
		.replace(/\bstruct\b/g, "")
		.replace(/\bWGPU_NULLABLE\b/g, "")
		.replace(/\bWGPU_NONNULL\b/g, "")
		.replace(/\s+/g, " ")
		.trim();

	if (t === "void") return "FFIType.void";

	const isPtr = t.includes("*");
	const base = t.replace(/\*/g, "").trim();

	if (callbackTypes.has(base)) {
		return isReturn ? "FFIType.ptr" : "FFIType.function";
	}

	if (structValueMap.has(base)) {
		const mapped = structValueMap.get(base);
		if (mapped === "ptr") return "FFIType.ptr";
		if (mapped === "uint64_t") return "FFIType.u64";
	}

	if (isPtr) {
		if (base === "char") return "FFIType.cstring";
		return "FFIType.ptr";
	}

	const scalar = typedefMap.get(base) || base;
	switch (scalar) {
		case "uint32_t":
			return "FFIType.u32";
		case "uint64_t":
			return "FFIType.u64";
		case "int32_t":
			return "FFIType.i32";
		case "int64_t":
			return "FFIType.i64";
		case "size_t":
		case "uintptr_t":
		case "intptr_t":
			return "FFIType.u64";
		case "float":
			return "FFIType.f32";
		case "double":
			return "FFIType.f64";
		case "int":
			return "FFIType.i32";
		case "unsigned int":
			return "FFIType.u32";
		default:
			if (base.startsWith("WGPU")) return "FFIType.ptr";
			if (base === "char") return "FFIType.u8";
			return "FFIType.ptr";
	}
};

const funcRegex = /WGPU_EXPORT\s+([\s\S]*?)\s*\(([^;]*?)\)\s*(?:WGPU_FUNCTION_ATTRIBUTE\s*)?;/g;
const symbols = [];
let funcMatch;
while ((funcMatch = funcRegex.exec(header))) {
	const beforeParen = funcMatch[1].trim().replace(/\s+/g, " ");
	const argsRaw = funcMatch[2].trim();

	const nameMatch = beforeParen.match(/^(.*)\s+([A-Za-z_][A-Za-z0-9_]*)$/);
	if (!nameMatch) continue;

	const returnType = nameMatch[1].trim();
	const funcName = nameMatch[2].trim();

	if (!funcName.startsWith("wgpu")) {
		continue;
	}

	const args = [];
	if (argsRaw && argsRaw !== "void") {
		const parts = argsRaw.split(",");
		for (const partRaw of parts) {
			const part = partRaw.trim();
			if (!part) continue;
			const paramMatch = part.match(/^(.*?)([A-Za-z_][A-Za-z0-9_]*)$/);
			const typePart = paramMatch ? paramMatch[1].trim() : part;
			args.push(typeToFFI(typePart, false));
		}
	}

	const returns = typeToFFI(returnType, true);
	const argsList = args.length ? args.join(", ") : "";
	symbols.push(`\t${funcName}: { args: [${argsList}], returns: ${returns} },`);
}

// 从共享常量构建 WGPU_LIB_NAMES 类型注解 + 值（使用 \t 对齐模板风格）
function buildWgpuLibNamesCode(): string {
	const lines: string[] = [];
	for (const [platform, names] of Object.entries(WGPU_LIB_FILENAMES)) {
		const quoted = names.map((n) => `"${n}"`).join(", ");
		lines.push(`\t${platform}: [${quoted}],`);
	}
	return `const WGPU_LIB_NAMES: Record<string, string[]> = {\n${lines.join("\n")}\n};`;
}

const output = `import { existsSync } from "fs";\nimport { join, dirname } from "path";\nimport { dlopen, suffix, FFIType } from "bun:ffi";\n\n// NOTE: WGPUStringView is passed by value in the C API. Bun FFI does not support\n// by-value structs, so WGPUStringView parameters are exposed as pointers for now.\n// If you need these calls, add a small C shim that accepts a pointer and\n// forwards by value. WGPUFuture is a single u64 and is mapped to FFIType.u64.\nconst WGPU_SYMBOLS = {\n${symbols.join("\n")}\n} as const;\n\n${buildWgpuLibNamesCode()}\n\nfunction findWgpuLibraryPath(): string | null {\n\tconst envPath = process.env.ELECTROBUN_WGPU_PATH;\n\tif (envPath && existsSync(envPath)) return envPath;\n\n\tconst names = WGPU_LIB_NAMES[process.platform] ?? ["libwebgpu_dawn." + suffix];\n\tfor (const name of names) {\n\t\tconst cwdCandidate = join(process.cwd(), name);\n\t\tif (existsSync(cwdCandidate)) return cwdCandidate;\n\t\tconst execDir = dirname(process.execPath);\n\t\tconst macCandidate = join(execDir, "..", "MacOS", name);\n\t\tif (existsSync(macCandidate)) return macCandidate;\n\t\tconst resCandidate = join(execDir, "..", "Resources", name);\n\t\tif (existsSync(resCandidate)) return resCandidate;\n\t\tconst execCandidate = join(execDir, name);\n\t\tif (existsSync(execCandidate)) return execCandidate;\n\t}\n\n\treturn null;\n}\n\nexport const native = (() => {\n\tconst libPath = findWgpuLibraryPath();\n\tif (!libPath) {\n\t\treturn {\n\t\t\tavailable: false,\n\t\t\tpath: null as string | null,\n\t\t\tsymbols: {} as Record<string, never>,\n\t\t\tclose: () => {},\n\t\t};\n\t}\n\n\ttry {\n\t\tconst lib = dlopen(libPath, WGPU_SYMBOLS);\n\t\treturn {\n\t\t\tavailable: true,\n\t\t\tpath: libPath,\n\t\t\tsymbols: lib.symbols,\n\t\t\tclose: lib.close,\n\t\t};\n\t} catch {\n\t\treturn {\n\t\t\tavailable: false,\n\t\t\tpath: libPath,\n\t\t\tsymbols: {} as Record<string, never>,\n\t\t\tclose: () => {},\n\t\t};\n\t}\n})();\n\nconst WGPU = {\n\tnative,\n};\n\nexport default WGPU;\n`;

const outPath = resolve(root, "src/core/bun/webGPU.ts");

// 内容未变时跳过写入，避免 mtime 变更触发不必要的重建
let skip = false;
try {
  const existing = readFileSync(outPath, "utf8");
  if (existing === output) {
    skip = true;
  }
} catch {
  // 文件不存在则正常写入
}
if (!skip) {
  writeFileSync(outPath, output, "utf8");
  console.log(`Generated ${outPath}`);
} else {
  console.log(`${outPath} 未变更，跳过写入`);
}
