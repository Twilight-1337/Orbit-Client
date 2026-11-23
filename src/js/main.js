import { el } from "./dom.js";
import { invoke, IpcError } from "./ipc.js";
import { state, setState, subscribe } from "./state.js";
import { renderAccounts } from "./views/accounts.js";

const viewRoot = document.getElementById("view");
const tabsRoot = document.getElementById("tabs");
const versionRoot = document.getElementById("app-version");

const PLACEHOLDERS = {
  instances: "Instance model, loader selection and Modrinth integration arrive in steps 5–9.",
  mods: "Modrinth search, mod installation and SHA-512 verification arrive in steps 7–8.",
  logs: "Live log streaming is wired in step 10.",
};

function card(title, body) {
  return el("section", { class: "card" }, [el("h2", { text: title }), body]);
}

function errorBanner() {
  if (!state.error) return null;
  return el("section", { class: "card error-card" }, [
    el("h2", { text: "Error" }),
    el("p", {}, [
      el("span", { class: "badge error", text: state.error.code ?? "error" }),
      el("span", { text: ` ${state.error.message}` }),
    ]),
  ]);
}

function backendCard() {
  if (!state.appInfo) {
    return card("Backend", el("p", { class: "muted", text: "Connecting to the Rust backend…" }));
  }

  const info = state.appInfo;
  const facts = el("dl", { class: "facts" }, [
    el("dt", { text: "Backend" }),
    el("dd", { text: `${info.name} v${info.version}` }),
    el("dt", { text: "Tauri" }),
    el("dd", { text: info.tauriVersion }),
    el("dt", { text: "Platform" }),
    el("dd", { text: `${info.platform} / ${info.arch}` }),
    el("dt", { text: "Data directory" }),
    el("dd", { text: info.dataDir }),
  ]);

  return card("Backend", facts);
}

function placeholderCard() {
  return card(
    state.view[0].toUpperCase() + state.view.slice(1),
    el("p", { class: "muted", text: PLACEHOLDERS[state.view] }),
  );
}

function render() {
  for (const tab of tabsRoot.querySelectorAll(".tab")) {
    tab.classList.toggle("is-active", tab.dataset.view === state.view);
  }

  const body =
    state.view === "accounts"
      ? [backendCard(), ...renderAccounts(state)]
      : [placeholderCard()];

  viewRoot.replaceChildren(...[errorBanner(), ...body].filter(Boolean));
}

async function loadAccountState() {
  try {
    const snapshot = await invoke("account_state");
    setState({ accounts: snapshot });
  } catch (err) {
    const message = err instanceof IpcError ? err.message : String(err);
    setState({ error: { code: err.code ?? "unknown", message } });
  }
}

async function boot() {
  tabsRoot.addEventListener("click", (event) => {
    const tab = event.target.closest(".tab");
    if (tab) setState({ view: tab.dataset.view });
  });

  subscribe(render);

  try {
    const info = await invoke("app_info");
    versionRoot.textContent = `v${info.version}`;
    setState({ appInfo: info });
  } catch (err) {
    const message = err instanceof IpcError ? err.message : String(err);
    setState({ error: { code: err.code ?? "unknown", message } });
  }

  await loadAccountState();
}

boot();
