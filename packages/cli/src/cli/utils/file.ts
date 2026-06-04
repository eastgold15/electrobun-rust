import { copyFileSync, existsSync, unlinkSync } from "fs";
import { execSync } from "child_process";
import * as path from "path";

/**
 * safeCopyFile: workaround for Bun's cpSync EPERM bug on Windows
 * Bun's fs.cpSync sometimes throws EPERM (errno -1) on Windows for single
 * files.  Node's copyFileSync doesn't have this issue.
 */
export function safeCopyFile(src: string, dest: string, maxRetries = 3) {
  let lastError: Error | null = null;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      copyFileSync(src, dest);
      return; // Success
    } catch (err: any) {
      lastError = err;

      // Only retry on EPERM or EBUSY errors (file locked/in use)
      if ((err.code !== 'EPERM' && err.code !== 'EBUSY') || attempt === maxRetries) {
        throw err;
      }

      // Delete the destination file if it exists and retry
      if (existsSync(dest)) {
        try {
          unlinkSync(dest);
        } catch (unlinkErr) {
          // If we can't delete it, wait and retry
        }
      }

      // Wait before retrying (exponential backoff: 100ms, 200ms, 400ms)
      const delay = Math.pow(2, attempt - 1) * 100;
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, delay);
    }
  }

  // Should never reach here, but just in case
  if (lastError) throw lastError;
}

/**
 * Create a tar file using system tar command (preserves file permissions unlike Bun.Archive)
 */
export function createTar(tarPath: string, cwd: string, entries: string[]) {
  // Use a relative path for the tar output on Windows to avoid bsdtar
  // interpreting the "C:" drive letter as a remote host specifier.
  const resolvedTarPath =
    process.platform === "win32" ? path.relative(cwd, tarPath) : tarPath;
  execSync(
    `tar -cf "${resolvedTarPath}" ${entries.map((e) => `"${e}"`).join(" ")}`,
    {
      cwd,
      stdio: "pipe",
      // Prevent macOS tar from including Apple Double (._*) files. No-op on other platforms.
      env: { ...process.env, COPYFILE_DISABLE: "1" },
    },
  );
}
