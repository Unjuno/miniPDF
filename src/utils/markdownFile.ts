const MARKDOWN_EXTENSIONS = new Set(['.md', '.markdown', '.mdown']);

export const isMarkdownFilePath = (filePath: string) => {
  const normalizedPath = filePath.trim().toLowerCase();
  const lastDot = normalizedPath.lastIndexOf('.');
  if (lastDot === -1) {
    return false;
  }
  const extension = normalizedPath.slice(lastDot);
  return MARKDOWN_EXTENSIONS.has(extension);
};
