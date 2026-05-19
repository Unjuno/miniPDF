export const getLineNumberFromOffset = (text: string, offset: number): number => {
  if (!text) return 1;
  const clampedOffset = Math.max(0, Math.min(offset, text.length));
  let line = 1;
  for (let i = 0; i < clampedOffset; i += 1) {
    if (text[i] === '\n') {
      line += 1;
    }
  }
  return line;
};

export const resolvePreviewPageFromMarkdown = (
  markdown: string,
  lineNumber: number,
  pageCount: number,
  linePageMap?: number[] | null
): number => {
  if (pageCount <= 1) return 1;

  if (linePageMap && linePageMap.length > 0) {
    const index = Math.max(0, lineNumber - 1);
    const clampedIndex = Math.min(index, linePageMap.length - 1);
    const mappedPage = linePageMap[clampedIndex] ?? linePageMap[linePageMap.length - 1] ?? 1;
    return Math.max(1, Math.min(pageCount, mappedPage));
  }

  return estimatePreviewPageFromMarkdown(markdown, lineNumber, pageCount);
};

export const estimatePreviewPageFromMarkdown = (
  markdown: string,
  lineNumber: number,
  pageCount: number
): number => {
  if (pageCount <= 1) return 1;

  const lines = markdown.split(/\r?\n/);
  const totalWeight = Math.max(1, lines.reduce((sum, line) => sum + getLineVisualWeight(line), 0));
  const clampedLine = Math.max(1, Math.min(lineNumber, lines.length || 1));

  if (lines.length === 1) return 1;

  const currentWeight = lines
    .slice(0, clampedLine - 1)
    .reduce((sum, line) => sum + getLineVisualWeight(line), 0)
    + Math.max(0.1, getLineVisualWeight(lines[clampedLine - 1] ?? ''));

  const normalized = Math.max(0, Math.min(1, currentWeight / totalWeight));
  const page = Math.ceil(normalized * pageCount);
  return Math.max(1, Math.min(pageCount, page));
};

export const estimatePreviewScrollTopFromMarkdown = (
  markdown: string,
  lineNumber: number,
  viewportHeight: number,
  contentHeight: number,
  linePageMap?: number[] | null
): number => {
  if (viewportHeight <= 0 || contentHeight <= viewportHeight) {
    return 0;
  }

  const lines = markdown.split(/\r?\n/);
  const clampedLine = Math.max(1, Math.min(lineNumber, lines.length || 1));
  const maxScrollTop = Math.max(0, contentHeight - viewportHeight);

  if (linePageMap && linePageMap.length > 0) {
    const mapIndex = Math.min(clampedLine - 1, linePageMap.length - 1);
    const pageAtLine = linePageMap[mapIndex] ?? linePageMap[linePageMap.length - 1] ?? 1;
    const maxPage = linePageMap[linePageMap.length - 1] ?? pageAtLine;
    if (maxPage <= 1) {
      return 0;
    }
    const normalized = Math.max(0, Math.min(1, (pageAtLine - 1) / (maxPage - 1)));
    return Math.max(0, Math.min(maxScrollTop, Math.round(normalized * maxScrollTop)));
  }

  const totalWeight = Math.max(1, lines.reduce((sum, line) => sum + getLineVisualWeight(line), 0));
  const currentWeight = lines
    .slice(0, clampedLine - 1)
    .reduce((sum, line) => sum + getLineVisualWeight(line), 0);

  const normalized = Math.max(0, Math.min(1, currentWeight / totalWeight));
  return Math.max(0, Math.min(maxScrollTop, Math.round(normalized * maxScrollTop)));
};

const getLineVisualWeight = (line: string): number => {
  const trimmed = line.trim();
  if (!trimmed) return 0.85;
  const headingMatch = trimmed.match(/^(#{1,6})\s+/);
  if (headingMatch) {
    const level = headingMatch[1].length;
    return [3.2, 2.8, 2.4, 2.1, 1.9, 1.7][level - 1] ?? 1.7;
  }
  if (/^>\s?/.test(trimmed)) return 1.3;
  if (/^(\*{3,}|-{3,}|_{3,})\s*$/.test(trimmed)) return 1.5;
  if (/^```/.test(trimmed) || /^~~~/.test(trimmed)) return 0.8;
  if (/^(\s*)([-+*]|\d+\.)\s+/.test(line)) return 1.15;
  if (/^\|.*\|$/.test(trimmed)) return 0.95;
  return 1.0;
};
