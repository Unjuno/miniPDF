import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('preview consistency fixture', () => {
  it('contains the Markdown structures needed to compare preview and PDF output', () => {
    const fixturePath = resolve(process.cwd(), 'fixtures', 'preview-consistency-test.md');
    const text = readFileSync(fixturePath, 'utf8');

    expect(text).toContain('```mermaid');
    expect(text).toContain('| 項目 | 値 |');
    expect(text).toContain('> この引用ブロックは');
    expect(text).toContain('- 子項目その1');
    expect(text).toContain('### 小見出し');
  });
});
