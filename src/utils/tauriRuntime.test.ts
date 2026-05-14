import { afterEach, describe, expect, it } from 'vitest';
import { isTauriRuntimeAvailable } from './tauriRuntime';

type TauriWindow = Window & { __TAURI__?: unknown; __TAURI_INTERNALS__?: unknown };

const tauriWindow = globalThis.window as TauriWindow;
const originalTauri = tauriWindow.__TAURI__;
const originalTauriInternals = tauriWindow.__TAURI_INTERNALS__;

afterEach(() => {
  tauriWindow.__TAURI__ = originalTauri;
  tauriWindow.__TAURI_INTERNALS__ = originalTauriInternals;
});

describe('isTauriRuntimeAvailable', () => {
  it('Tauri関連オブジェクトがない場合はfalseを返す', () => {
    delete tauriWindow.__TAURI__;
    delete tauriWindow.__TAURI_INTERNALS__;

    expect(isTauriRuntimeAvailable()).toBe(false);
  });

  it('__TAURI__ が存在する場合はtrueを返す', () => {
    tauriWindow.__TAURI__ = {};
    delete tauriWindow.__TAURI_INTERNALS__;

    expect(isTauriRuntimeAvailable()).toBe(true);
  });

  it('__TAURI_INTERNALS__ が存在する場合はtrueを返す', () => {
    delete tauriWindow.__TAURI__;
    tauriWindow.__TAURI_INTERNALS__ = {};

    expect(isTauriRuntimeAvailable()).toBe(true);
  });
});
