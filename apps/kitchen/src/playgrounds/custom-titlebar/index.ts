import Electrobun, { Electroview } from "@pori15/electrobun-rust/view";

const rpc = Electroview.defineRPC<any>({
  maxRequestTime: 600_000,
  handlers: {
    requests: {},
    messages: {},
  },
});

const electrobun = new Electrobun.Electroview({ rpc });

document.addEventListener("DOMContentLoaded", () => {
  // Custom window control buttons
  document.getElementById("closeBtn")?.addEventListener("click", () => {
    (electrobun.rpc as any)?.request.closeWindow({});
  });

  document.getElementById("minimizeBtn")?.addEventListener("click", () => {
    (electrobun.rpc as any)?.request.minimizeWindow({});
  });

  document.getElementById("maximizeBtn")?.addEventListener("click", () => {
    (electrobun.rpc as any)?.request.maximizeWindow({});
  });

  // Done button
  document.getElementById("doneBtn")?.addEventListener("click", () => {
    (electrobun.rpc as any)?.request.closeWindow({});
  });
});
