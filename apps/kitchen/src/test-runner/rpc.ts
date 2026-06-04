import type { RPCSchema } from "@pori15/electrobun-rust";
import type {
  UpdateStatusDetails,
  UpdateStatusEntry,
  UpdateStatusType,
} from "@pori15/electrobun-rust/bun";
import type { TestResult } from "../test-framework/types";

export interface TestInfo {
  category: string;
  description?: string;
  id: string;
  interactive: boolean;
  name: string;
}

export type UpdateStatus =
  | "checking"
  | "update-available"
  | "downloading"
  | "update-ready"
  | "no-update"
  | "error";

export type { UpdateStatusDetails, UpdateStatusEntry, UpdateStatusType };

export interface UpdateInfo {
  currentVersion: string;
  error?: string;
  newVersion?: string;
  status: UpdateStatus;
}

export interface TestRunnerPreferences {
  searchQuery: string;
}

export type TestRunnerRPC = {
  bun: RPCSchema<{
    requests: {
      getTests: {
        params: {};
        response: TestInfo[];
      };
      runTest: {
        params: { testId: string };
        response: TestResult;
      };
      runAllAutomated: {
        params: {};
        response: TestResult[];
      };
      runInteractiveTests: {
        params: {};
        response: TestResult[];
      };
      submitInteractiveResult: {
        params: { testId: string; passed: boolean; notes?: string };
        response: void;
      };
      submitReady: {
        params: { testId: string };
        response: void;
      };
      submitVerification: {
        params: {
          testId: string;
          action: "pass" | "fail" | "retest";
          notes?: string;
        };
        response: void;
      };
      applyUpdate: {
        params: {};
        response: void;
      };
      getUpdateStatusHistory: {
        params: {};
        response: UpdateStatusEntry[];
      };
      clearUpdateStatusHistory: {
        params: {};
        response: void;
      };
      getTestRunnerPreferences: {
        params: {};
        response: TestRunnerPreferences;
      };
      setTestRunnerPreferences: {
        params: TestRunnerPreferences;
        response: void;
      };
    };
    messages: {
      logToBun: {
        msg: string;
      };
    };
  }>;
  webview: RPCSchema<{
    requests: {};
    messages: {
      testStarted: {
        testId: string;
        name: string;
      };
      testCompleted: {
        testId: string;
        result: TestResult;
      };
      testLog: {
        testId: string;
        message: string;
      };
      allCompleted: {
        results: TestResult[];
      };
      interactiveWaiting: {
        testId: string;
        instructions: string[];
      };
      interactiveReady: {
        testId: string;
        instructions: string[];
      };
      interactiveVerify: {
        testId: string;
      };
      buildConfig: {
        defaultRenderer: "native" | "cef";
        availableRenderers: ("native" | "cef")[];
        mainProcess?: "bun" | "zig";
        cefVersion?: string;
        bunVersion?: string;
        zigVersion?: string;
      };
      updateStatus: UpdateInfo;
      updateStatusEntry: UpdateStatusEntry;
    };
  }>;
};
