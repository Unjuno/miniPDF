export const isTauriRuntimeAvailable = () => {
  if (globalThis.window === undefined) {
    return false;
  }
  const tauriWindow = globalThis.window as Window & { __TAURI__?: unknown; __TAURI_INTERNALS__?: unknown };
  return Boolean(tauriWindow.__TAURI__ || tauriWindow.__TAURI_INTERNALS__);
};
