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
  // Close button
  document.getElementById("closeBtn")?.addEventListener("click", () => {
    (electrobun.rpc as any)?.request.closeWindow({});
  });

  // Make the floating cards draggable
  const cards = document.querySelectorAll(".floating-card");
  cards.forEach((card) => {
    card.classList.add("electrobun-webkit-app-region-drag");
  });
});
