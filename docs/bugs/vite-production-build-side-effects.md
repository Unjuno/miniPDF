# Bug Note: vite-production-build-side-effects

## Symptom
- `npm run build` completed, but the generated entry bundle only contained the Vite modulepreload polyfill and the production app rendered blank.

## Root Cause
- `vite.config.ts` forced `rollupOptions.treeshake.moduleSideEffects = false`, which allowed Rollup to strip the app bootstrap side effects from the production entry bundle.

## Fix
- Removed the custom `treeshake` override and restored Rollup's default side-effect handling for production builds.

## Prevent Regression
- Added a config test that fails if production bundling disables module side effects again (`src/viteBuildConfig.test.ts`).
