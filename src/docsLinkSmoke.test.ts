import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const repoRoot = process.cwd();
const readmeLinkTargets = [
  './docs/FEATURES.md',
  './docs/HTML_PREVIEW.md',
  './docs/markdown-editor-preview-sync.md',
  './docs/markdown-katex-rendering.md',
  './docs/SPECIFICATION.md',
  './docs/CURRENT_ISSUES_SUMMARY.md',
  './docs/INDEX.md',
  './docs/RELEASING.md',
  './LICENSE',
];

const indexLinkTargets = [
  './FEATURES.md',
  './HTML_PREVIEW.md',
  './markdown-editor-preview-sync.md',
  './markdown-katex-rendering.md',
  './SPECIFICATION.md',
  './CURRENT_ISSUES_SUMMARY.md',
  './dev/repo-cleanup-shim-inventory.md',
  './RELEASING.md',
  './CONCEPT.md',
  './DO_NOT.md',
  './FONT_MANAGEMENT.md',
  './FILE_LIST.md',
  './COMPLETE_FILE_LIST.md',
  './archive/POTENTIAL_ISSUES.md',
  './archive/POTENTIAL_ISSUES_ANALYSIS.md',
  './archive/LATEST_ISSUES_ANALYSIS.md',
  './archive/FINAL_ISSUES_ANALYSIS.md',
];

function assertTargetsExist(markdownPath: string, targets: readonly string[]) {
  const markdown = readFileSync(markdownPath, 'utf8');

  for (const target of targets) {
    expect(markdown).toContain(`](${target})`);
    expect(readFileSync(resolve(markdownPath, '..', target), 'utf8')).toBeDefined();
  }
}

describe('docs link smoke check', () => {
  it('keeps README links pointed at tracked docs', () => {
    assertTargetsExist(resolve(repoRoot, 'README.md'), readmeLinkTargets);
  });

  it('keeps docs index links pointed at tracked docs', () => {
    assertTargetsExist(resolve(repoRoot, 'docs', 'INDEX.md'), indexLinkTargets);
  });
});
