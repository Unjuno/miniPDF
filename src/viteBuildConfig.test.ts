// @vitest-environment node
import { describe, expect, it } from 'vitest';
import viteConfig from '../vite.config';

describe('vite production config', () => {
  it('keeps module side effects enabled for the production bundle', () => {
    const treeshake = viteConfig.build?.rollupOptions?.treeshake;

    if (treeshake && typeof treeshake === 'object' && 'moduleSideEffects' in treeshake) {
      const { moduleSideEffects } = treeshake as { moduleSideEffects?: unknown };
      expect(moduleSideEffects).not.toBe(false);
      return;
    }

    expect(treeshake).toBeUndefined();
  });
});
