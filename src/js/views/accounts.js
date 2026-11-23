// Вкладка аккаунтов: отображение шлюза лицензии и управление профилями.
// Никаких правил доступа здесь нет — только вызов команд и рендер ответа.

import { el } from "../dom.js";
import { invoke } from "../ipc.js";
import { setState } from "../state.js";

let draftUsername = "";
let busy = false;

async function run(command, args) {
  if (busy) return;
  busy = true;
  try {
    const snapshot = await invoke(command, args);
    setState({ accounts: snapshot, error: null });
  } catch (err) {
    setState({ error: { code: err.code, message: err.message } });
  } finally {
    busy = false;
  }
}

function badge(text, tone) {
  return el("span", { class: `badge ${tone}`, text });
}

function accountRow(account, activeId) {
  const isActive = account.id === activeId;

  const status = el("span", { class: "row-status" }, [
    isActive ? badge("active", "ok") : null,
    account.blocked ? badge("blocked", "error") : null,
    account.licenseVerified ? badge("licensed", "ok") : null,
  ]);

  const actions = el("div", { class: "row-actions" }, [
    isActive
      ? null
      : el("button", {
          type: "button",
          class: "ghost",
          text: "Use",
          on: { click: () => run("set_active_account", { accountId: account.id }) },
        }),
    el("button", {
      type: "button",
      class: "ghost danger",
      text: "Remove",
      on: { click: () => confirmRemoval(account) },
    }),
  ]);

  return el("li", { class: "account-row" }, [
    el("div", { class: "row-main" }, [
      el("div", { class: "row-title" }, [
        el("span", { class: "kind", text: account.kind === "microsoft" ? "MS" : "OFF" }),
        el("strong", { text: account.username }),
        status,
      ]),
      el("div", { class: "row-sub" }, [
        el("code", { text: account.mcUuid }),
        el("span", { text: account.kind === "microsoft" ? "Microsoft account" : "Offline profile" }),
      ]),
    ]),
    actions,
  ]);
}

async function confirmRemoval(account) {
  let confirmed = true;
  try {
    confirmed = window.confirm(`Remove account "${account.username}"?`);
  } catch {
    confirmed = true;
  }
  if (confirmed) await run("remove_account", { accountId: account.id });
}

function licenseCard(snapshot) {
  const gate = snapshot.hasLicense
    ? badge("license verified", "ok")
    : badge("no license", "error");

  const note = snapshot.canCreateOffline
    ? "Offline profiles are available."
    : "Offline profiles are locked until a verified Microsoft account is added.";

  return el("section", { class: "card" }, [
    el("h2", { text: "License gate" }),
    el("p", {}, [gate, el("span", { text: ` ${note}` })]),
  ]);
}

function accountsCard(snapshot) {
  if (snapshot.accounts.length === 0) {
    return el("section", { class: "card" }, [
      el("h2", { text: "Accounts" }),
      el("p", { class: "muted", text: "No accounts yet." }),
    ]);
  }

  return el("section", { class: "card" }, [
    el("h2", { text: "Accounts" }),
    el(
      "ul",
      { class: "account-list" },
      snapshot.accounts.map((account) => accountRow(account, snapshot.activeAccountId)),
    ),
  ]);
}

function offlineForm(snapshot) {
  const input = el("input", {
    type: "text",
    id: "offline-username",
    placeholder: "3–16 chars, A–Z 0–9 _",
    maxlength: "16",
    autocomplete: "off",
    value: draftUsername,
    on: {
      input: (event) => {
        draftUsername = event.target.value;
      },
    },
  });

  const submit = el("button", {
    type: "submit",
    text: "Add offline profile",
    disabled: !snapshot.canCreateOffline,
  });

  const form = el(
    "form",
    {
      class: "inline-form",
      on: {
        submit: (event) => {
          event.preventDefault();
          const username = input.value.trim();
          if (!username) return;
          draftUsername = "";
          run("create_offline_account", { username });
        },
      },
    },
    [input, submit],
  );

  return el("section", { class: "card" }, [
    el("h2", { text: "Offline profile" }),
    form,
    snapshot.canCreateOffline
      ? el("p", { class: "muted", text: "Uses the Java-compatible offline UUID (v3)." })
      : el("p", { class: "muted", text: "Blocked: add a verified Microsoft account first." }),
  ]);
}

function debugCard() {
  return el("section", { class: "card" }, [
    el("h2", { text: "Debug" }),
    el("p", { class: "muted", text: "Seeds a fake licensed account to test the gate without OAuth. Debug builds only." }),
    el("div", { class: "inline-form" }, [
      el("button", {
        type: "button",
        text: "Seed licensed account",
        on: { click: () => run("debug_seed_licensed_account", {}) },
      }),
    ]),
  ]);
}

export function renderAccounts(state) {
  const snapshot = state.accounts;

  if (!snapshot) {
    return [
      el("section", { class: "card" }, [
        el("h2", { text: "Accounts" }),
        el("p", { class: "muted", text: "Loading account state…" }),
      ]),
    ];
  }

  return [
    licenseCard(snapshot),
    accountsCard(snapshot),
    offlineForm(snapshot),
    debugCard(),
  ];
}
