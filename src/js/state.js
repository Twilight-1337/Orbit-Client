// Примитивный store без фреймворка: одно состояние + подписчики.
// Вся бизнес-логика остаётся в Rust, здесь только отображение.

export const state = {
  view: "accounts",
  appInfo: null,
  accounts: null,
  error: null,
};

const subscribers = new Set();

export function setState(patch) {
  Object.assign(state, patch);
  for (const notify of subscribers) {
    notify(state);
  }
}

export function subscribe(notify) {
  subscribers.add(notify);
  notify(state);
  return () => subscribers.delete(notify);
}
