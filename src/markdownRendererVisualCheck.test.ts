import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('markdown renderer visual check fixture', () => {
  it('contains the requested coverage areas', () => {
    const fixturePath = resolve(process.cwd(), 'fixtures', 'markdown-renderer-visual-check.md');
    const text = readFileSync(fixturePath, 'utf8');

    expect(text).toContain('Markdown Renderer Visual Check');
    expect(text).toContain('H1 見出し');
    expect(text).toContain('- [x] 完了タスク');
    expect(text).toContain('```mermaid');
    expect(text).toContain('$E = mc^2$');
    expect(text).toContain('\\|\\mathbf{x}\\|_2');
    expect(text).toContain('🧪');
    expect(text).toContain('Unicode / 日本語 / 絵文字');
    expect(text).toContain('終端確認');
    expect(text).toContain('引用内コードブロック');
    expect(text).toContain('<script>');
    expect(text).toContain('[^note1]');
  });
});
