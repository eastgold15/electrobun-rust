import { Electroview } from "@pori15/electrobun-rust/view";

const rpc = Electroview.defineRPC<any>({
  maxRequestTime: 600_000,
  handlers: {
    requests: {},
    messages: {},
  },
});

const electrobun = new Electroview({ rpc });

// Done button handler
document.addEventListener("DOMContentLoaded", () => {
  document.getElementById("doneBtn")?.addEventListener("click", () => {
    (electrobun.rpc as any)?.request.closeWindow({});
  });
});
