import { BrowserWindow } from "./core/BrowserWindow";

try {
  const win = new BrowserWindow({
    title: "Wry Test",
    html: "<h1 style='color:blue;font-family:sans-serif'>Hello Wry!</h1><p>如果看到这行字，wry 渲染正常</p>",
    frame: { width: 600, height: 400, x: 300, y: 200 },
  });
  console.log("Wry test window created:", win.id);
  win.show();
} catch (e) {
  console.error("Failed:", e);
}
