// Тонкая обёртка над Tauri IPC.
// Никакой бизнес-логики: только вызов команд и нормализация ошибок.

const tauri = window.__TAURI__;

export class IpcError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "IpcError";
    this.code = code;
  }
}

export function isAvailable() {
  return Boolean(tauri?.core?.invoke);
}

export async function invoke(command, args) {
  if (!isAvailable()) {
    throw new IpcError("ipc_unavailable", "Tauri IPC is not available in this context");
  }

  try {
    return await tauri.core.invoke(command, args);
  } catch (err) {
    if (err && typeof err === "object" && typeof err.code === "string") {
      throw new IpcError(err.code, err.message ?? "Unknown backend error");
    }
    throw new IpcError("unknown", String(err?.message ?? err));
  }
}

export function listen(event, handler) {
  if (!tauri?.event?.listen) {
    return () => {};
  }

  const pending = tauri.event.listen(event, (message) => handler(message.payload));
  return () => {
    pending.then((unlisten) => unlisten()).catch(() => {});
  };
}
