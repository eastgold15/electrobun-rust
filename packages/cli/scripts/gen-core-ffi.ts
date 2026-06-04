/**
 * 从 Rust lib.rs 自动生成 core dlopen 的 FFI 符号定义
 *
 * 用法: cd package && bun scripts/gen-core-ffi.ts
 *
 * 输出到 stdout，可手动替换 native.ts 中的 core dlopen 块
 */
import { readFileSync } from "fs";
import { join } from "path";

const RUST_SRC = join(import.meta.dir, "..", "src", "core", "src", "lib.rs");

const RUST_TO_FFI: Record<string, string> = {
  "*const c_char": "FFIType.cstring",
  "*mut c_char": "FFIType.cstring",
  "c_char": "FFIType.cstring",
  "f64": "FFIType.f64",
  "f32": "FFIType.f32",
  "u32": "FFIType.u32",
  "i32": "FFIType.i32",
  "u64": "FFIType.u64",
  "i64": "FFIType.i64",
  "u8": "FFIType.u8",
  "i8": "FFIType.i8",
  "u16": "FFIType.u16",
  "i16": "FFIType.i16",
  "usize": "FFIType.u64",
  "isize": "FFIType.i64",
  "bool": "FFIType.bool",
  "*mut std::ffi::c_void": "FFIType.ptr",
  "*const std::ffi::c_void": "FFIType.ptr",
  "std::os::raw::c_int": "FFIType.i32",
  "*mut u32": "FFIType.ptr",
  "*mut f64": "FFIType.ptr",
  "*mut f32": "FFIType.ptr",
  "*mut u64": "FFIType.ptr",
  "*mut i32": "FFIType.ptr",
  "*const std::os::raw::c_char": "FFIType.cstring",
  "*mut std::os::raw::c_char": "FFIType.cstring",
};

function rustTypeToFFI(rustType: string): string {
  // Remove trailing whitespace and normalize
  const t = rustType.trim();

  // Direct match
  if (RUST_TO_FFI[t]) return RUST_TO_FFI[t];

  // Function pointer: Option<unsafe extern "C" fn(...)>
  if (t.includes("Option<unsafe extern") || t.includes("unsafe extern")) {
    return "FFIType.function";
  }

  // Pointer to function
  if (t.startsWith("*mut ") && t.includes("fn")) {
    return "FFIType.function";
  }

  console.warn(`  未知类型: ${t}`);
  return "FFIType.ptr";
}

function rustReturnToFFI(rustReturn: string): string {
  const t = rustReturn.trim();
  if (!t || t === "()" || t === "void") return "FFIType.void";

  if (t.startsWith("*const c_char") || t.startsWith("*mut c_char")) {
    return "FFIType.cstring";
  }
  if (t.startsWith("*mut std::ffi::c_void")) {
    return "FFIType.ptr";
  }
  if (t.includes("*mut c_char")) return "FFIType.cstring";

  return rustTypeToFFI(t);
}

interface RustFunc {
  name: string;
  args: { name: string; type: string }[];
  returns: string;
}

function parseRustFunctions(source: string): RustFunc[] {
  const funcs: RustFunc[] = [];

  // Find each #[no_mangle] then manually parse balanced parens
  const noMangleRe = /#\[no_mangle\]\s*\n\s*pub\s+extern\s+"C"\s+fn\s+(\w+)\s*\(/g;
  let m;
  while ((m = noMangleRe.exec(source)) !== null) {
    const name = m[1];
    if (!name.startsWith("electrobun_")) continue;

    // Parse balanced parentheses for args
    let depth = 1;
    let argsStart = noMangleRe.lastIndex;
    let i = argsStart;
    while (i < source.length && depth > 0) {
      if (source[i] === "(") depth++;
      else if (source[i] === ")") depth--;
      i++;
    }
    const argsStr = source.substring(argsStart, i - 1).trim();

    // Parse return type (optional, before {)
    let returns = "void";
    let j = i;
    // Skip whitespace
    while (j < source.length && (source[j] === " " || source[j] === "\t")) j++;
    if (source[j] === "-" && source[j + 1] === ">") {
      j += 2;
      // Find { - but skip balanced < > and nested parens
      let retStart = j;
      while (j < source.length && source[j] !== "{") j++;
      returns = source.substring(retStart, j).trim();
    } else {
      // No return type, find {
      while (j < source.length && source[j] !== "{") j++;
    }

    // Parse args
    const args: { name: string; type: string }[] = [];
    if (argsStr) {
      // Split by comma, but respect nested <> and fn()
      const parts = splitArgsTopLevel(argsStr);
      for (const part of parts) {
        const argMatch = part.match(/^(_?\w+)\s*:\s*(.+)$/);
        if (argMatch) {
          args.push({ name: argMatch[1].replace(/^_/, ""), type: argMatch[2].trim() });
        } else if (part) {
          args.push({ name: "arg", type: part.trim() });
        }
      }
    }

    funcs.push({ name, args, returns });
  }

  return funcs;
}

/** Split comma-separated top-level args, respecting nested <> and () */
function splitArgsTopLevel(str: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < str.length; i++) {
    const ch = str[i];
    if (ch === "<" || ch === "(") depth++;
    else if (ch === ">" || ch === ")") depth--;
    else if (ch === "," && depth === 0) {
      parts.push(str.substring(start, i).trim());
      start = i + 1;
    }
  }
  parts.push(str.substring(start).trim());
  return parts.filter(Boolean);
}

// ── 主逻辑 ────────────────────────────────────────────────
const source = readFileSync(RUST_SRC, "utf8");
const funcs = parseRustFunctions(source);

console.log("// 从 lib.rs 自动生成 — 与 Rust DLL 签名完全匹配");
console.log("// 使用方式: 替换 native.ts 中 return dlopen(corePath, { ... }) 的符号对象");
console.log();
console.log(`// 共 ${funcs.length} 个符号`);
console.log();

for (const func of funcs) {
  const args = func.args.map(a => rustTypeToFFI(a.type)).join(", ");
  const returns = rustReturnToFFI(func.returns);

  const hasComplexArgs = func.args.length > 3;
  if (hasComplexArgs) {
    // 多行格式
    console.log(`\t\t\t\t${func.name}: {`);
    console.log(`\t\t\t\t\targs: [`);
    for (const arg of func.args) {
      console.log(`\t\t\t\t\t\t${rustTypeToFFI(arg.type)}, // ${arg.name}`);
    }
    console.log(`\t\t\t\t\t],`);
    console.log(`\t\t\t\t\treturns: ${returns},`);
    console.log(`\t\t\t\t},`);
  } else if (func.args.length === 0) {
    console.log(`\t\t\t\t${func.name}: {`);
    console.log(`\t\t\t\t\targs: [],`);
    console.log(`\t\t\t\t\treturns: ${returns},`);
    console.log(`\t\t\t\t},`);
  } else {
    const argList = func.args.map(a => rustTypeToFFI(a.type)).join(", ");
    console.log(`\t\t\t\t${func.name}: { args: [${argList}], returns: ${returns} },`);
  }
}
