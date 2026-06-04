// Interactive Webview Settings Tests - Playground for webview tag initial settings

import { BrowserView, BrowserWindow } from "@pori15/electrobun-rust/bun";
import { defineTest } from "../../test-framework/types";

export const webviewSettingsTests = [
  defineTest({
    name: "Webview Settings playground",
    category: "Webview Tag (Interactive)",
    description:
      "Add webview tags with transparent/passthrough/sandbox settings and toggle them at runtime",
    interactive: true,
    timeout: 600_000,
    async run({ log, showInstructions }) {
      await showInstructions([
        "A webview settings playground will open",
        "Toggle transparent, passthrough, sandbox, renderer, then click '+ Add Webview'",
        "Use the toggle buttons on each card to change settings at runtime",
        "Click the red '-' button to remove a webview",
        "Close the window when done to pass the test",
      ]);

      log("Opening webview settings playground window");

      await new Promise<void>((resolve) => {
        let winRef: BrowserWindow<any> | null = null;

        const rpc = BrowserView.defineRPC<any>({
          maxRequestTime: 600_000,
          handlers: {
            requests: {
              closeWindow: () => {
                winRef?.close();
                return { success: true };
              },
            },
            messages: {},
          },
        });

        winRef = new BrowserWindow({
          title: "Webview Settings Playground",
          url: "views://playgrounds/webview-settings/index.html",
          renderer: "cef",
          frame: { width: 700, height: 800, x: 150, y: 50 },
          rpc,
        });

        winRef.setAlwaysOnTop(true);
        const win = winRef;

        win.on("close", () => {
          log("Playground closed - test complete");
          resolve();
        });
      });
    },
  }),
];
