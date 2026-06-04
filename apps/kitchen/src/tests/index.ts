// Test Index - Exports all test definitions

import type { TestDefinition } from "../test-framework/types";
import { babylonAdapterTests } from "./babylon-adapter.test";
import { eventsTests } from "./events.test";
import { chromelessTests } from "./interactive/chromeless.test";
import { clipboardInteractiveTests } from "./interactive/clipboard.test";
// Interactive tests
import { dialogTests } from "./interactive/dialogs.test";
import { fullsizeFrameReproTests } from "./interactive/fullsize-frame-repro.test";
import { menuTests } from "./interactive/menus.test";
import { multiwindowCefTests } from "./interactive/multiwindow-cef.test";
import { permissionTests } from "./interactive/permissions.test";
import { quitTests } from "./interactive/quit-test.test";
import { shortcutTests } from "./interactive/shortcuts.test";
import { trayTests } from "./interactive/tray.test";
import { webviewCleanupTests } from "./interactive/webview-cleanup.test";
import { webviewSettingsTests } from "./interactive/webview-settings.test";
import { webviewTagTests } from "./interactive/webview-tag.test";
import { wgpuTagTests } from "./interactive/wgpu-tag.test";
import { wgpuViewTests } from "./interactive/wgpu-view.test";
import { windowEventTests } from "./interactive/window-events.test";
import { navigationTests } from "./navigation.test";
import { preloadTests } from "./preload.test";
import { rpcTests } from "./rpc.test";
import { sandboxTests } from "./sandbox.test";
import { screenTests } from "./screen.test";
import { sessionTests } from "./session.test";
import { trayApiTests } from "./tray-api.test";
import { updaterTests } from "./updater.test";
import { utilsTests } from "./utils.test";
import { wgpuAdapterTests } from "./wgpu-adapter.test";
import { wgpuAdapterExtendedTests } from "./wgpu-adapter-extended.test";
// Automated tests
import { wgpuFfiTests } from "./wgpu-ffi.test";
import { windowTests } from "./window.test";

// Collect all tests
export const allTests: TestDefinition[] = [
  // Automated tests (run in parallel)
  ...rpcTests,
  ...windowTests,
  ...navigationTests,
  ...utilsTests,
  ...screenTests,
  ...sessionTests,
  ...eventsTests,
  ...preloadTests,
  ...updaterTests,
  ...sandboxTests,
  ...trayApiTests,
  ...wgpuFfiTests,
  ...wgpuAdapterTests,
  ...babylonAdapterTests,
  ...wgpuAdapterExtendedTests,

  // Interactive tests (run sequentially, require user)
  ...dialogTests,
  ...trayTests,
  ...shortcutTests,
  ...webviewTagTests,
  ...clipboardInteractiveTests,
  ...menuTests,
  ...windowEventTests,
  ...chromelessTests,
  ...multiwindowCefTests,
  ...quitTests,
  ...webviewSettingsTests,
  ...webviewCleanupTests,
  ...wgpuViewTests,
  ...wgpuTagTests,
  ...fullsizeFrameReproTests,
  ...permissionTests,
];

// Export by category for selective running
export const automatedTests: TestDefinition[] = allTests.filter(
  (t) => !t.interactive
);
export const interactiveTests: TestDefinition[] = allTests.filter(
  (t) => t.interactive
);

// Export individual test suites for reference
export {
  babylonAdapterTests,
  chromelessTests,
  clipboardInteractiveTests,
  dialogTests,
  eventsTests,
  fullsizeFrameReproTests,
  menuTests,
  multiwindowCefTests,
  navigationTests,
  permissionTests,
  preloadTests,
  quitTests,
  rpcTests,
  sandboxTests,
  screenTests,
  sessionTests,
  shortcutTests,
  trayApiTests,
  trayTests,
  updaterTests,
  utilsTests,
  webviewCleanupTests,
  webviewSettingsTests,
  webviewTagTests,
  wgpuAdapterExtendedTests,
  wgpuAdapterTests,
  wgpuFfiTests,
  wgpuTagTests,
  wgpuViewTests,
  windowEventTests,
  windowTests,
};
