import { OS } from "../../core/shared/platform";

/**
 * Escape a file path for safe use in terminal commands
 */
export function escapePathForTerminal(path: string): string {
  if (OS === "win") {
    return `"${path.replace(/"/g, '""')}"`;
  } else {
    return `'${path.replace(/'/g, "'\\''")}'`;
  }
}
