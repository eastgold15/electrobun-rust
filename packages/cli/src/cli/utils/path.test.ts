import { describe, expect, it } from "bun:test";
import { escapePathForTerminal } from "./path";
import { OS } from "../../core/shared/platform";

describe("escapePathForTerminal", () => {
  it("returns the path wrapped in quotes", () => {
    const result = escapePathForTerminal("/some/path");
    expect(result.length).toBeGreaterThan("/some/path".length);
  });

  it("starts and ends with a quote character", () => {
    const result = escapePathForTerminal("/some/path");
    expect(result[0]).toBe(OS === "win" ? '"' : "'");
    expect(result[result.length - 1]).toBe(OS === "win" ? '"' : "'");
  });

  it("escapes embedded quotes on Unix", () => {
    // On Unix, single quotes inside get escaped as: '\''
    const result = escapePathForTerminal("/path/with'quote");
    if (OS !== "win") {
      expect(result).toContain("'\\''");
    }
  });

  it("escapes embedded double-quotes on Windows", () => {
    const result = escapePathForTerminal('C:\\path\\with"quote');
    if (OS === "win") {
      expect(result).toBe('"C:\\path\\with""quote"');
    }
  });

  it("handles paths with spaces", () => {
    const result = escapePathForTerminal("/path/with spaces/file");
    if (OS !== "win") {
      expect(result).toBe("'/path/with spaces/file'");
    } else {
      expect(result).toBe('"/path/with spaces/file"');
    }
  });

  it("handles simple paths without special chars", () => {
    const result = escapePathForTerminal("/usr/local/bin/app");
    if (OS !== "win") {
      expect(result).toBe("'/usr/local/bin/app'");
    } else {
      expect(result).toBe('"/usr/local/bin/app"');
    }
  });
});
