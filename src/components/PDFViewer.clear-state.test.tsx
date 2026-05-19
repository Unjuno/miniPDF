import { render, waitFor, cleanup, screen } from '@testing-library/react';
import React from 'react';
import { PDFViewer } from './PDFViewer';
import { usePdfStore } from '../stores/pdfStore';
import { readFile } from '@tauri-apps/plugin-fs';

const clearMock = vi.fn();

vi.mock('../hooks/useRenderCache', () => {
  const noop = () => {};
  return {
    useRenderCache: () => ({
      get: noop,
      set: noop,
      clear: clearMock,
      size: noop,
    }),
  };
});

vi.mock('@tauri-apps/plugin-fs', () => ({
  readFile: vi.fn(),
}));

vi.mock('pdfjs-dist', () => ({
  GlobalWorkerOptions: {
    workerSrc: '',
  },
  getDocument: vi.fn(() => ({
    promise: Promise.resolve({
      numPages: 0,
      getPage: vi.fn(),
    }),
    destroy: vi.fn(() => Promise.resolve()),
  })),
}));

describe('PDFViewer clears state when pdfStructure is cleared', () => {
  afterEach(() => {
    clearMock.mockClear();
    cleanup();
    vi.clearAllMocks();
    usePdfStore.setState({
      currentPage: 1,
    });
  });

  it('clears render cache and resets current page when pdfStructure is null', async () => {
    usePdfStore.setState({ currentPage: 3 });

    render(<PDFViewer pdfStructure={null} zoomLevel={1} previewOnly />);

    expect(screen.getByText('Markdownを入力すると印刷プレビューが表示されます')).toBeTruthy();

    await waitFor(() => {
      expect(usePdfStore.getState().currentPage).toBe(1);
      expect(clearMock).toHaveBeenCalled();
    });
  });

  it('starts in continuous mode for print-preview style browsing', async () => {
    vi.mocked(readFile).mockResolvedValue(new Uint8Array([1, 2, 3]));
    const loadingTaskDestroyMock = vi.fn(() => Promise.resolve());

    const getViewport = vi.fn(({ scale = 1 }) => ({
      width: 595 * scale,
      height: 842 * scale,
    }));
    const renderMock = vi.fn(() => ({ promise: Promise.resolve() }));
    const getPage = vi.fn(async () => ({
      getViewport,
      render: renderMock,
    }));

    vi.mocked((await import('pdfjs-dist')).getDocument).mockReturnValue({
      promise: Promise.resolve({
        numPages: 1,
        getPage,
        destroy: vi.fn(),
      }),
      destroy: loadingTaskDestroyMock,
    } as unknown as ReturnType<typeof import('pdfjs-dist')['getDocument']>);

    HTMLCanvasElement.prototype.getContext = vi.fn(() => null);

    const pdfStructure = {
      filePath: 'preview.pdf',
      metadata: {},
      pages: [
        {
          pageNumber: 1,
          sourcePageNumber: 1,
          width: 595,
          height: 842,
          images: [],
          textBlocks: [],
        },
      ],
    };

    const { container, unmount } = render(<PDFViewer pdfStructure={pdfStructure} zoomLevel={1} previewOnly />);

    await waitFor(() => {
      expect(container.querySelector('[data-view-mode="continuous"]')).not.toBeNull();
    });

    const pdfjs = await import('pdfjs-dist');
    await waitFor(() => {
      expect(vi.mocked(pdfjs.getDocument)).toHaveBeenCalled();
    });

    unmount();
    await waitFor(() => {
      expect(loadingTaskDestroyMock).toHaveBeenCalled();
    });
  });

  it('preserves the current page when loading a refreshed preview PDF', async () => {
    vi.mocked(readFile).mockResolvedValue(new Uint8Array([1, 2, 3]));
    const loadingTaskDestroyMock = vi.fn(() => Promise.resolve());

    const getViewport = vi.fn(({ scale = 1 }) => ({
      width: 595 * scale,
      height: 842 * scale,
    }));
    const renderMock = vi.fn(() => ({ promise: Promise.resolve() }));
    const getPage = vi.fn(async () => ({
      getViewport,
      render: renderMock,
    }));

    vi.mocked((await import('pdfjs-dist')).getDocument).mockReturnValue({
      promise: Promise.resolve({
        numPages: 3,
        getPage,
        destroy: vi.fn(),
      }),
      destroy: loadingTaskDestroyMock,
    } as unknown as ReturnType<typeof import('pdfjs-dist')['getDocument']>);

    HTMLCanvasElement.prototype.getContext = vi.fn(() => null);
    HTMLElement.prototype.scrollTo = vi.fn();

    usePdfStore.setState({ currentPage: 3 });

    const pdfStructure = {
      filePath: 'preview.pdf',
      metadata: {},
      pages: [
        {
          pageNumber: 1,
          sourcePageNumber: 1,
          width: 595,
          height: 842,
          images: [],
          textBlocks: [],
        },
        {
          pageNumber: 2,
          sourcePageNumber: 2,
          width: 595,
          height: 842,
          images: [],
          textBlocks: [],
        },
        {
          pageNumber: 3,
          sourcePageNumber: 3,
          width: 595,
          height: 842,
          images: [],
          textBlocks: [],
        },
      ],
    };

    render(<PDFViewer pdfStructure={pdfStructure} zoomLevel={1} previewOnly />);

    const pdfjs = await import('pdfjs-dist');
    await waitFor(() => {
      expect(vi.mocked(pdfjs.getDocument)).toHaveBeenCalled();
      expect(usePdfStore.getState().currentPage).toBe(3);
    });
  });

  it('previewOnly hides layout overlays and edit toolbar actions', async () => {
    vi.mocked(readFile).mockResolvedValue(new Uint8Array([1, 2, 3]));

    const getViewport = vi.fn(({ scale = 1 }) => ({
      width: 595 * scale,
      height: 842 * scale,
    }));
    const renderMock = vi.fn(() => ({ promise: Promise.resolve() }));
    const getPage = vi.fn(async () => ({
      getViewport,
      render: renderMock,
    }));

    vi.mocked((await import('pdfjs-dist')).getDocument).mockReturnValue({
      promise: Promise.resolve({
        numPages: 1,
        getPage,
        destroy: vi.fn(),
      }),
      destroy: vi.fn(() => Promise.resolve()),
    } as unknown as ReturnType<typeof import('pdfjs-dist')['getDocument']>);

    HTMLCanvasElement.prototype.getContext = vi.fn(() => null);

    const pdfStructure = {
      filePath: 'preview.pdf',
      metadata: {},
      pages: [
        {
          pageNumber: 1,
          sourcePageNumber: 1,
          width: 595,
          height: 842,
          images: [],
          textBlocks: [
            {
              id: 'tb1',
              x: 72,
              y: 72,
              width: 200,
              height: 20,
              text: 'Hello',
              fontSize: 12,
              lineHeight: 1.2,
              fontFamily: 'Helvetica',
            },
          ],
        },
      ],
    };

    const { container } = render(<PDFViewer pdfStructure={pdfStructure} zoomLevel={1} previewOnly />);

    await waitFor(() => {
      expect(container.querySelector('[data-view-mode="continuous"]')).not.toBeNull();
    });

    expect(container.querySelector('.pdf-structure-overlay')).toBeNull();
    expect(screen.queryByRole('button', { name: '新しいテキストブロックを追加' })).toBeNull();
    expect(screen.queryByRole('button', { name: '新しい画像を挿入' })).toBeNull();
    expect(screen.queryByText(/テキスト:/)).toBeNull();
  });

  it('scrolls the single-page preview when the cursor line changes', async () => {
    vi.mocked(readFile).mockResolvedValue(new Uint8Array([1, 2, 3]));

    const getViewport = vi.fn(({ scale = 1 }) => ({
      width: 595 * scale,
      height: 842 * scale,
    }));
    const renderMock = vi.fn(() => ({ promise: Promise.resolve() }));
    const getPage = vi.fn(async () => ({
      getViewport,
      render: renderMock,
    }));

    vi.mocked((await import('pdfjs-dist')).getDocument).mockReturnValue({
      promise: Promise.resolve({
        numPages: 1,
        getPage,
        destroy: vi.fn(),
      }),
      destroy: vi.fn(() => Promise.resolve()),
    } as unknown as ReturnType<typeof import('pdfjs-dist')['getDocument']>);

    HTMLCanvasElement.prototype.getContext = vi.fn(() => null);

    const pdfStructure = {
      filePath: 'preview.pdf',
      metadata: {},
      pages: [
        {
          pageNumber: 1,
          sourcePageNumber: 1,
          width: 595,
          height: 842,
          images: [],
          textBlocks: [],
        },
      ],
    };

    usePdfStore.setState({
      markdownText: [
        'afaga',
        'adada',
        'afasa',
        'adaf',
        '# fsga',
        '---',
        '',
        'adada',
      ].join('\n'),
    });

    const { container, rerender } = render(
      <PDFViewer pdfStructure={pdfStructure} zoomLevel={1} previewOnly editorCursorLine={1} />
    );

    const canvasContainer = container.querySelector('.pdf-viewer-canvas-container') as HTMLElement;
    Object.defineProperty(canvasContainer, 'clientHeight', { configurable: true, value: 800 });
    Object.defineProperty(canvasContainer, 'scrollHeight', { configurable: true, value: 1600 });
    Object.defineProperty(canvasContainer, 'scrollTop', { configurable: true, writable: true, value: 0 });

    const pdfjs = await import('pdfjs-dist');
    await waitFor(() => {
      expect(vi.mocked(pdfjs.getDocument)).toHaveBeenCalled();
    });

    rerender(<PDFViewer pdfStructure={pdfStructure} zoomLevel={1} previewOnly editorCursorLine={12} />);

    await waitFor(() => {
      expect(canvasContainer.scrollTop).toBeGreaterThan(0);
    });
  });
});
