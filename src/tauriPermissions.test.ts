// @vitest-environment node
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import path from 'node:path';

describe('tauri permissions', () => {
  it('allows the markdown preview command through the custom command ACL', () => {
    const permissionsPath = path.resolve(__dirname, '../src-tauri/permissions/custom-commands.toml');
    const permissionsToml = readFileSync(permissionsPath, 'utf8');

    expect(permissionsToml).toContain('"render_markdown_to_pdf_preview"');
  });

  it('applies the custom command ACL to the main window capability', () => {
    const capabilityPath = path.resolve(__dirname, '../src-tauri/capabilities/main-capability.json');
    const capability = JSON.parse(readFileSync(capabilityPath, 'utf8')) as { permissions?: string[] };

    expect(capability.permissions).toContain('custom-commands');
  });
});
