// @vitest-environment node
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('package scripts', () => {
  it('runs vitest in one-shot mode by default', () => {
    const packageJsonPath = path.resolve(fileURLToPath(new URL('.', import.meta.url)), '..', 'package.json');
    const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf8')) as {
      scripts?: { test?: string; 'markdown:preview'?: string };
    };

    expect(packageJson.scripts?.test).toBe('vitest --run');
    expect(packageJson.scripts?.['markdown:preview']).toBe(
      'cargo run --manifest-path src-tauri/Cargo.toml --bin markdown_preview_cli --'
    );
  });
});
