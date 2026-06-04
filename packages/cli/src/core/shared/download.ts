/**
 * 共享下载工具 — 统一所有模块的文件下载/解压逻辑
 *
 * 将 SDK 构建（scripts/build-sdk.ts）和应用构建（src/cli/downloads/*）
 * 中的重复下载逻辑收敛到此处。
 */

import { createWriteStream, existsSync, mkdirSync, unlinkSync, statSync } from "fs";
import { execSync } from "child_process";
import { join } from "path";

// ── 类型 ────────────────────────────────────────────────────────────

export interface DownloadOptions {
    /** 下载标签（用于日志输出） */
    label?: string;
    /** 最大重试次数（默认 3） */
    maxRetries?: number;
    /** 重试延迟基数（ms，指数退避，默认 2000） */
    retryDelayBase?: number;
    /** 进度回调（percent: 0-100） */
    onProgress?: (percent: number, downloadedMb: number, totalMb: number) => void;
}

// ── 下载 ────────────────────────────────────────────────────────────

/**
 * 下载文件，带进度报告和自动重试
 */
export async function downloadFile(
    url: string,
    destPath: string,
    options?: DownloadOptions,
): Promise<number /* downloaded bytes */> {
    const label = options?.label ?? "Download";
    const maxRetries = options?.maxRetries ?? 3;
    const retryDelayBase = options?.retryDelayBase ?? 2000;

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
        try {
            console.log(`${label} (attempt ${attempt}/${maxRetries}): ${url}`);
            const controller = new AbortController();
            const timeout = setTimeout(() => controller.abort(), 30000);
            const response = await fetch(url, { signal: controller.signal });
            clearTimeout(timeout);
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const contentLength = response.headers.get("content-length");
            const totalSize = contentLength ? parseInt(contentLength, 10) : 0;
            const fileStream = createWriteStream(destPath);
            let downloadedSize = 0;
            let lastReportedPercent = -1;

            if (response.body) {
                const reader = response.body.getReader();
                while (true) {
                    const { done, value } = await reader.read();
                    if (done) break;
                    const chunk = Buffer.from(value);
                    fileStream.write(chunk);
                    downloadedSize += chunk.length;
                    if (totalSize > 0) {
                        const percent = Math.round((downloadedSize / totalSize) * 100);
                        const percentTier = Math.floor(percent / 10) * 10;
                        if (percentTier > lastReportedPercent && percentTier <= 100) {
                            const downloadedMb = Math.round(downloadedSize / 1024 / 1024);
                            const totalMb = Math.round(totalSize / 1024 / 1024);
                            options?.onProgress?.(percentTier, downloadedMb, totalMb);
                            console.log(`  Progress: ${percentTier}% (${downloadedMb}MB/${totalMb}MB)`);
                            lastReportedPercent = percentTier;
                        }
                    }
                }
            }

            await new Promise<void>((resolve, reject) => {
                fileStream.end((error: any) => {
                    if (error) reject(error);
                    else resolve();
                });
            });

            // 校验下载完整性
            if (totalSize > 0) {
                const actualSize = statSync(destPath).size;
                if (actualSize !== totalSize) {
                    throw new Error(
                        `Downloaded file size mismatch: expected ${totalSize}, got ${actualSize}`,
                    );
                }
            }

            if (!existsSync(destPath) || statSync(destPath).size === 0) {
                throw new Error("Downloaded file is empty or missing");
            }

            console.log(`✓ ${label} completed (${Math.round(downloadedSize / 1024 / 1024)}MB)`);
            return downloadedSize;
        } catch (error: any) {
            console.error(`${label} attempt ${attempt} failed:`, error.message);
            // 清理损坏文件
            try {
                if (existsSync(destPath)) unlinkSync(destPath);
            } catch { /* ignore */ }

            if (attempt === maxRetries) {
                throw new Error(`${label} failed after ${maxRetries} attempts: ${error.message}`);
            }

            const delay = Math.min(Math.pow(2, attempt) * retryDelayBase, 30000);
            console.log(`Retrying in ${Math.round(delay / 1000)}s...`);
            await new Promise((resolve) => setTimeout(resolve, delay));
        }
    }

    throw new Error("Unreachable");
}

// ── 解压 ────────────────────────────────────────────────────────────

/**
 * 使用 Bun.Archive 解压 .tar.gz
 */
export async function extractTarGz(source: string, destDir: string): Promise<void> {
    mkdirSync(destDir, { recursive: true });
    const tarBytes = await Bun.file(source).arrayBuffer();
    const archive = new Bun.Archive(tarBytes);
    await archive.extract(destDir);
}

/**
 * 使用 tar 命令解压 .tar.bz2
 */
export async function extractTarBz2(source: string, destDir: string): Promise<void> {
    mkdirSync(destDir, { recursive: true });
    execSync(`tar -xjf "${source}" --strip-components=1 -C "${destDir}"`, { stdio: "inherit" });
}

/**
 * 跨平台解压 .zip
 */
export async function extractZip(source: string, destDir: string): Promise<void> {
    mkdirSync(destDir, { recursive: true });
    if (process.platform === "win32") {
        execSync(
            `powershell -command "Expand-Archive -Path '${source}' -DestinationPath '${destDir}' -Force"`,
            { stdio: "inherit" },
        );
    } else {
        execSync(`unzip -o "${source}" -d "${destDir}"`, { stdio: "inherit" });
    }
}

// ── 复合操作 ─────────────────────────────────────────────────────────

/**
 * 下载 .tar.gz 并解压
 */
export async function downloadAndExtractTarGz(
    url: string,
    destDir: string,
    options?: DownloadOptions,
): Promise<void> {
    const label = options?.label ?? "Download";
    const tempFile = join(destDir, `..`, `.temp-${Date.now()}.tar.gz`);
    try {
        await downloadFile(url, tempFile, { ...options, label });
        await extractTarGz(tempFile, destDir);
    } finally {
        try {
            if (existsSync(tempFile)) unlinkSync(tempFile);
        } catch { /* ignore */ }
    }
}

/**
 * 下载 .zip 并解压
 */
export async function downloadAndExtractZip(
    url: string,
    destDir: string,
    options?: DownloadOptions,
): Promise<void> {
    const label = options?.label ?? "Download";
    const tempFile = join(destDir, `..`, `.temp-${Date.now()}.zip`);
    try {
        await downloadFile(url, tempFile, { ...options, label });
        await extractZip(tempFile, destDir);
    } finally {
        try {
            if (existsSync(tempFile)) unlinkSync(tempFile);
        } catch { /* ignore */ }
    }
}
