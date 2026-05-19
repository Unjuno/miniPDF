import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { usePdfStore } from './stores/pdfStore';

vi.mock('./components/PDFViewer', async () => {
  const { usePdfStore } = await import('./stores/pdfStore');

  const PDFViewer = () => {
    const currentPage = usePdfStore((state) => state.currentPage);
    const setCurrentPage = usePdfStore((state) => state.setCurrentPage);

    return (
      <div>
        <div data-testid="preview-page">Page {currentPage}</div>
        <button onClick={() => setCurrentPage(2)}>Go to page 2</button>
      </div>
    );
  };

  return { PDFViewer: PDFViewer };
});

vi.mock('./components/ErrorDisplay', () => ({
  ErrorDisplay: () => null,
}));

vi.mock('./components/Toast', () => ({
  ToastContainer: () => null,
}));

vi.mock('./components/KeyboardShortcutsHelp', () => ({
  KeyboardShortcutsHelp: () => null,
}));

vi.mock('./hooks/useKeyboardShortcuts', () => ({
  useKeyboardShortcuts: () => undefined,
}));

vi.mock('./hooks/useToast', () => ({
  useToast: () => ({
    toasts: [],
    dismissToast: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

vi.mock('./hooks/useDebounce', () => ({
  useDebounce: <T,>(value: T) => value,
}));

import App from './App';

describe('App preview page sync', () => {
  beforeEach(() => {
    const markdownText = Array.from({ length: 40 }, (_, index) => `line ${index + 1}`).join('\n');
    usePdfStore.setState({
      pdfStructure: {
        filePath: 'preview.pdf',
        metadata: {},
        pages: [
          {
            pageNumber: 1,
            width: 595,
            height: 842,
            images: [],
            textBlocks: [],
          },
          {
            pageNumber: 2,
            width: 595,
            height: 842,
            images: [],
            textBlocks: [],
          },
        ],
      },
      currentPage: 1,
      markdownText,
      zoomLevel: 1,
      error: null,
      previewError: null,
      previewHtml: null,
      previewPdfPath: null,
      previewRequestId: 0,
      isPreviewBuilding: false,
      isLoading: false,
      isEditing: false,
      selectedImageId: null,
      selectedTextBlockId: null,
    });
  });

  afterEach(() => {
    usePdfStore.setState({
      pdfStructure: null,
      currentPage: 1,
      markdownText: '',
      zoomLevel: 1,
      error: null,
      previewError: null,
      previewHtml: null,
      previewPdfPath: null,
      previewRequestId: 0,
      isPreviewBuilding: false,
      isLoading: false,
      isEditing: false,
      selectedImageId: null,
      selectedTextBlockId: null,
    });
  });

  it('keeps a manually selected preview page instead of snapping back to the cursor page', async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByTestId('preview-page').textContent).toContain('Page 1');
    });

    fireEvent.click(screen.getByRole('button', { name: 'Go to page 2' }));

    await waitFor(() => {
      expect(screen.getByTestId('preview-page').textContent).toContain('Page 2');
      expect(usePdfStore.getState().currentPage).toBe(2);
    });
  });

  it('updates the preview page when Enter inserts a hard break across a page boundary', async () => {
    const longMarkdown = Array.from({ length: 100 }, (_, index) => `line ${index + 1}`).join('\n');
    // One extra slot so the map still matches after Enter adds a line (101 lines).
    const linePageMap = Array.from({ length: 101 }, (_, index) => (index < 50 ? 1 : 2));
    usePdfStore.setState({
      markdownText: longMarkdown,
      currentPage: 1,
      previewLinePageMap: linePageMap,
    });

    render(<App />);

    const editor = screen.getByPlaceholderText('# Markdownを入力してください') as HTMLTextAreaElement;
    const lines = longMarkdown.split('\n');
    const offsetToLine50End = lines.slice(0, 50).reduce((sum, line) => sum + line.length + 1, 0) - 1;
    editor.setSelectionRange(offsetToLine50End, offsetToLine50End);

    fireEvent.keyDown(editor, { key: 'Enter', code: 'Enter', keyCode: 13, charCode: 13 });

    await waitFor(() => {
      expect(usePdfStore.getState().markdownText.split('\n').length).toBe(101);
      expect(screen.getByTestId('preview-page').textContent).toContain('Page 2');
    });
  });
});
