// Test framework types

export type TestStatus =
  | "pending"
  | "running"
  | "passed"
  | "failed"
  | "skipped";

export interface TestResult {
  duration?: number;
  error?: string;
  logs?: string[];
  name: string;
  status: TestStatus;
  testId: string;
}

export interface TestDefinition {
  category: string;
  description?: string;
  id: string;
  interactive: boolean;
  name: string;
  run: (context: TestContext) => Promise<void>;
  timeout?: number;
}

export interface TestSuiteDefinition {
  category: string;
  name: string;
  setup?: () => Promise<any>;
  teardown?: (fixture: any) => Promise<void>;
  tests: Omit<TestDefinition, "category">[];
}

export type InteractiveResult = {
  action: "pass" | "fail" | "retest";
  notes?: string;
};

export interface TestContext {
  // Window creation helpers
  createWindow: (options: WindowOptions) => Promise<TestWindow>;
  // Log to test output
  log: (message: string) => void;
  // For interactive tests - show instructions and wait for user to be ready
  showInstructions: (instructions: string[]) => Promise<void>;
  // Legacy - wait for user action (combines show + verify)
  waitForUserAction: (
    instructions: string[]
  ) => Promise<{ passed: boolean; notes?: string }>;
  // For interactive tests - wait for user to verify (pass/fail/retest)
  waitForUserVerification: () => Promise<InteractiveResult>;
}

export type TitleBarStyle = "default" | "hiddenInset" | "hidden";

export interface WindowOptions {
  activate?: boolean;
  height?: number;
  hidden?: boolean;
  html?: string;
  preload?: string;
  renderer?: "cef" | "native";
  rpc?: any;
  sandbox?: boolean; // When true, disables RPC and only allows event emission
  title?: string;
  titleBarStyle?: TitleBarStyle;
  trafficLightOffset?: { x: number; y: number };
  url?: string;
  width?: number;
  x?: number;
  y?: number;
}

export interface TestWindow {
  close: () => void;
  id: number;
  webview: any; // BrowserView
  webviewId: number;
  window: any; // BrowserWindow
}

// Assertion error
export class AssertionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "AssertionError";
  }
}

// Simple expect assertions
export function expect<T>(actual: T, label?: string) {
  const prefix = label ? `[${label}] ` : "";

  return {
    toBe(expected: T) {
      if (actual !== expected) {
        throw new AssertionError(
          `${prefix}Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`
        );
      }
    },
    toEqual(expected: T) {
      if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        throw new AssertionError(
          `${prefix}Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`
        );
      }
    },
    toBeGreaterThan(n: number) {
      if (typeof actual !== "number" || actual <= n) {
        throw new AssertionError(`${prefix}Expected ${actual} > ${n}`);
      }
    },
    toBeGreaterThanOrEqual(n: number) {
      if (typeof actual !== "number" || actual < n) {
        throw new AssertionError(`${prefix}Expected ${actual} >= ${n}`);
      }
    },
    toBeLessThan(n: number) {
      if (typeof actual !== "number" || actual >= n) {
        throw new AssertionError(`${prefix}Expected ${actual} < ${n}`);
      }
    },
    toBeLessThanOrEqual(n: number) {
      if (typeof actual !== "number" || actual > n) {
        throw new AssertionError(`${prefix}Expected ${actual} <= ${n}`);
      }
    },
    toBeTruthy() {
      if (!actual) {
        throw new AssertionError(
          `${prefix}Expected truthy, got ${JSON.stringify(actual)}`
        );
      }
    },
    toBeFalsy() {
      if (actual) {
        throw new AssertionError(
          `${prefix}Expected falsy, got ${JSON.stringify(actual)}`
        );
      }
    },
    toBeNull() {
      if (actual !== null) {
        throw new AssertionError(
          `${prefix}Expected null, got ${JSON.stringify(actual)}`
        );
      }
    },
    toBeUndefined() {
      if (actual !== undefined) {
        throw new AssertionError(
          `${prefix}Expected undefined, got ${JSON.stringify(actual)}`
        );
      }
    },
    toBeDefined() {
      if (actual === undefined) {
        throw new AssertionError(`${prefix}Expected defined, got undefined`);
      }
    },
    toContain(item: any) {
      if (typeof actual === "string") {
        if (!actual.includes(item)) {
          throw new AssertionError(
            `${prefix}Expected string to contain "${item}"`
          );
        }
      } else if (Array.isArray(actual)) {
        if (!actual.includes(item)) {
          throw new AssertionError(
            `${prefix}Expected array to contain ${JSON.stringify(item)}`
          );
        }
      } else {
        throw new AssertionError(
          `${prefix}toContain only works with strings and arrays`
        );
      }
    },
    toHaveLength(length: number) {
      if (!Array.isArray(actual) && typeof actual !== "string") {
        throw new AssertionError(
          `${prefix}toHaveLength only works with strings and arrays`
        );
      }
      if ((actual as any).length !== length) {
        throw new AssertionError(
          `${prefix}Expected length ${length}, got ${(actual as any).length}`
        );
      }
    },
    toBeInstanceOf(constructor: any) {
      if (!(actual instanceof constructor)) {
        throw new AssertionError(
          `${prefix}Expected instance of ${constructor.name}`
        );
      }
    },
    toMatch(regex: RegExp) {
      if (typeof actual !== "string" || !regex.test(actual)) {
        throw new AssertionError(
          `${prefix}Expected "${actual}" to match ${regex}`
        );
      }
    },
    toThrow() {
      if (typeof actual !== "function") {
        throw new AssertionError(`${prefix}toThrow expects a function`);
      }
      try {
        (actual as any)();
        throw new AssertionError(`${prefix}Expected function to throw`);
      } catch (e) {
        if (e instanceof AssertionError) {
          throw e;
        }
        // Function threw as expected
      }
    },
  };
}

// Helper to define a test
let testIdCounter = 0;
export function defineTest(config: {
  name: string;
  category: string;
  description?: string;
  interactive?: boolean;
  timeout?: number;
  run: (context: TestContext) => Promise<void>;
}): TestDefinition {
  return {
    id: `test-${++testIdCounter}-${config.name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`,
    name: config.name,
    category: config.category,
    description: config.description,
    interactive: config.interactive ?? false,
    timeout: config.timeout ?? 10_000,
    run: config.run,
  };
}

// Helper to define a test suite with shared setup/teardown
export function defineTestSuite(config: TestSuiteDefinition): TestDefinition[] {
  return config.tests.map((test, index) => ({
    id: `test-${++testIdCounter}-${config.name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}-${index}`,
    name: test.name,
    category: config.category,
    description: test.description,
    interactive: test.interactive,
    timeout: test.timeout ?? 10_000,
    run: test.run,
  }));
}
