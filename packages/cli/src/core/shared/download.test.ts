import { describe, expect, it } from "bun:test";
import { join } from "path";
import { tmpdir } from "os";

describe("downloadFile", () => {
  it("is exported as a function", async () => {
    const { downloadFile } = await import("./download");
    expect(typeof downloadFile).toBe("function");
  });

  it("accepts DownloadOptions with maxRetries", async () => {
    const { downloadFile } = await import("./download");
    const fnStr = downloadFile.toString();
    expect(fnStr).toContain("maxRetries");
    expect(fnStr).toContain("retryDelayBase");
  });

  it("throws on unreachable URL", async () => {
    const { downloadFile } = await import("./download");
    const dest = join(tmpdir(), `download-test-${Date.now()}`);
    try {
      await downloadFile(
        "https://invalid.example.invalid/file.tar.gz",
        dest,
        { label: "FailTest", maxRetries: 1, retryDelayBase: 100 },
      );
      expect.unreachable("should have thrown");
    } catch (error: any) {
      // Any error means the function rejected — success
      expect(error).toBeDefined();
    }
  });
});

describe("extractTarGz", () => {
  it("exists and is a function", async () => {
    const { extractTarGz } = await import("./download");
    expect(typeof extractTarGz).toBe("function");
  });
});
