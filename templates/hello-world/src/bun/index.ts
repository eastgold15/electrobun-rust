import { BrowserWindow } from "@pori15/electrobun-rust/bun";

try {
    const mainWindow = new BrowserWindow({
        title: "Hello Electrobun!",
        url: "views://mainview/index.html",
        frame: { width: 800, height: 800, x: 200, y: 200 },
    });
    console.log("Window created:", mainWindow);
} catch (e) {
    console.error("创建窗口失败:", e);
}
