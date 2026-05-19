# Superpowers SessionStart / Agent panel flicker (Cursor, Windows)

- **Date:** 2026-05-19
- **Type:** Bug / environment
- **Status:** mitigated (plugin cache patch v2)
- **Target:** Cursor + Superpowers plugin v5.0.7

## Symptom

- `sessionStart` ran repeatedly; many `bash.exe` processes; Cursor sluggish.
- After partial fixes, SessionStart spam stopped but **Agent panel opened then immediately closed**.
- `run-hook.cmd session-start` cold start ~32s; warm ~250–500ms.

## Root cause

1. Windows: `hooks-cursor.json` must use `run-hook.cmd`, not raw `session-start`.
2. Disabling only `hooks-cursor.json` `sessionStart: []` was **not enough**: `hooks/hooks.json` still registered Claude-format `SessionStart` with `run-hook.cmd`.
3. Plugin manifest `"hooks": "./hooks/hooks-cursor.json"` still registered hooks with Cursor until removed.

## Fix v2 (local plugin cache)

Run from repo:

```powershell
.\scripts\patch-superpowers-cursor-hooks.ps1
```

Patches:

1. Remove `"hooks"` from Superpowers `.cursor-plugin/plugin.json`
2. Empty `hooks/hooks.json` and `hooks/hooks-cursor.json`
3. `session-start` → instant `{}` when `CURSOR_PLUGIN_ROOT` is set
4. Runlayer: `sessionStart: []` if installed

Project static rules: `.cursor/rules/superpowers-lite.mdc`

## Verification

```powershell
$env:CURSOR_PLUGIN_ROOT = "$env:USERPROFILE\.cursor\plugins\cache\cursor-public\superpowers\b7a8f76985f1e93e75dd2f2a3b424dc731bd9d37"
Measure-Command { & "$env:CURSOR_PLUGIN_ROOT\hooks\run-hook.cmd" session-start }
```

Expect: `{}`, exit 0, **&lt; 1s**.

Then: quit all Cursor windows → reopen miniPDF → Agent panel stays open.

## If still broken

1. **Cursor Settings → Plugins → disable Superpowers** (isolates plugin vs Cursor bug)
2. **Settings → Hooks**: clear any user-defined `sessionStart` entries
3. **Settings → Features**: toggle “Third-party skills” off, restart (stops Claude-format hook merge)

## Residual risk

Plugin cache overwrites on update; re-run patch script.
