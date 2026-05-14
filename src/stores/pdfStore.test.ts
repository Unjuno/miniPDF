import { describe, it, expect, beforeEach, vi, Mock } from 'vitest';
import { usePdfStore } from './pdfStore';
import { PdfStructure } from '../types/pdf';

// Tauri APIのモック
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readFile: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';

const mockPdf = {
  numPages: 1,
  getPage: vi.fn(async () => ({
    getViewport: () => ({ width: 800, height: 1000 }),
    getTextContent: vi.fn(),
    getOperatorList: vi.fn(async () => ({ fnArray: [], argsArray: [] })),
    objs: new Map(),
  })),
  destroy: vi.fn(),
};

vi.mock('pdfjs-dist', () => ({
  GlobalWorkerOptions: {
    workerSrc: '',
  },
  getDocument: vi.fn(() => ({
    promise: Promise.resolve(mockPdf),
  })),
  OPS: { save: 'save', restore: 'restore', transform: 'transform', paintXObject: 'paintXObject' },
}));

const mockExtractTextWithPDFjs = vi.fn();
const mockExtractImageAreasWithPDFjs = vi.fn();

vi.mock('../utils/pdfTextExtractor', () => ({
  extractTextWithPDFjs: mockExtractTextWithPDFjs,
}));

vi.mock('../utils/pdfImageExtractor', () => ({
  extractImageAreasWithPDFjs: mockExtractImageAreasWithPDFjs,
}));

describe('pdfStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    usePdfStore.setState({
      pdfStructure: null,
      selectedImageId: null,
      selectedTextBlockId: null,
      zoomLevel: 1,
      error: null,
      currentPage: 1,
      isLoading: false,
      isEditing: false,
    });
    mockExtractTextWithPDFjs.mockReset();
    mockExtractImageAreasWithPDFjs.mockReset();
  });

  it('初期状態が正しく設定されている', () => {
    const store = usePdfStore.getState();
    expect(store.pdfStructure).toBeNull();
    expect(store.selectedImageId).toBeNull();
    expect(store.selectedTextBlockId).toBeNull();
    expect(store.zoomLevel).toBe(1);
    expect(store.error).toBeNull();
  });

  it('画像を選択できる', () => {
    const store = usePdfStore.getState();
    store.selectImage('img-1');
    expect(usePdfStore.getState().selectedImageId).toBe('img-1');
  });

  it('画像選択を解除できる', () => {
    const store = usePdfStore.getState();
    store.selectImage('img-1');
    store.selectImage(null);
    expect(usePdfStore.getState().selectedImageId).toBeNull();
  });

  it('テキストブロックを選択できる', () => {
    const store = usePdfStore.getState();
    store.selectTextBlock('text-1');
    expect(usePdfStore.getState().selectedTextBlockId).toBe('text-1');
  });

  it('ズームレベルを設定できる', () => {
    const store = usePdfStore.getState();
    store.setZoomLevel(1.5);
    expect(usePdfStore.getState().zoomLevel).toBe(1.5);
  });

  it('ズームレベルが範囲内に制限される', () => {
    const store = usePdfStore.getState();
    store.setZoomLevel(3.0);
    expect(usePdfStore.getState().zoomLevel).toBe(2.0);
    
    store.setZoomLevel(0.1);
    expect(usePdfStore.getState().zoomLevel).toBe(0.5);
  });

  it('エラーを設定できる', () => {
    const store = usePdfStore.getState();
    store.setError('テストエラー');
    expect(usePdfStore.getState().error).toBe('テストエラー');
  });

  it('エラーをクリアできる', () => {
    const store = usePdfStore.getState();
    store.setError('テストエラー');
    store.clearError();
    expect(usePdfStore.getState().error).toBeNull();
  });

  it('pageNumberが欠落していてもデフォルトのページ番号で読み込める', async () => {
    const mockedInvoke = invoke as unknown as Mock;
    const mockedReadFile = readFile as unknown as Mock;

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'load_pdf') {
        const structure: PdfStructure = {
          filePath: 'dummy.pdf',
          metadata: {},
          pages: [
            {
              // @ts-expect-error intentionally undefined to simulate backend omission
              pageNumber: undefined,
              width: 800,
              height: 1000,
              images: [],
              textBlocks: [],
            },
          ],
        };
        return Promise.resolve(structure);
      }
      return Promise.resolve([]);
    });
    mockedReadFile.mockResolvedValue(new Uint8Array([1, 2, 3]));
    mockExtractTextWithPDFjs.mockResolvedValue([
      { id: 'text_1_0', x: 0, y: 0, width: 10, height: 10, text: 'hi', fontSize: 12, lineHeight: 1.2, fontFamily: 'Arial' },
    ]);
    mockExtractImageAreasWithPDFjs.mockResolvedValue([]);

    await usePdfStore.getState().loadPdf('dummy.pdf');

    const pdfStructure = usePdfStore.getState().pdfStructure;
    expect(pdfStructure?.pages[0].pageNumber).toBe(1);
    expect(mockExtractTextWithPDFjs).toHaveBeenCalledWith(
      expect.anything(),
      mockPdf,
      1,
      800,
      1000,
      []
    );
  });

  it('PDF読み込み失敗時にエラーメッセージを設定する', async () => {
    const mockedInvoke = invoke as unknown as Mock;
    mockedInvoke.mockRejectedValue(new Error('load failed'));

    await expect(usePdfStore.getState().loadPdf('dummy.pdf')).rejects.toThrow('load failed');
    expect(usePdfStore.getState().error).toBe('load failed');
    expect(usePdfStore.getState().pdfStructure).toBeNull();
    expect(usePdfStore.getState().selectedImageId).toBeNull();
    expect(usePdfStore.getState().selectedTextBlockId).toBeNull();
    expect(usePdfStore.getState().currentPage).toBe(1);
  });

  it('PDF読み込み失敗時に前回のPDFをクリアする', async () => {
    const mockedInvoke = invoke as unknown as Mock;
    mockedInvoke.mockRejectedValue(new Error('load failed'));

    usePdfStore.setState({
      pdfStructure: {
        filePath: 'prev.pdf',
        metadata: {},
        pages: [],
      } as unknown as PdfStructure,
    });

    await expect(usePdfStore.getState().loadPdf('dummy.pdf')).rejects.toThrow('load failed');
    expect(usePdfStore.getState().pdfStructure).toBeNull();
    expect(usePdfStore.getState().selectedImageId).toBeNull();
    expect(usePdfStore.getState().selectedTextBlockId).toBeNull();
    expect(usePdfStore.getState().currentPage).toBe(1);
  });

  it('PDF読み込み成功時に選択状態とカレントページをリセットする', async () => {
    const mockedInvoke = invoke as unknown as Mock;
    const mockedReadFile = readFile as unknown as Mock;

    usePdfStore.setState({
      selectedImageId: 'old-img',
      selectedTextBlockId: 'old-text',
      currentPage: 3,
    });

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'load_pdf') {
        const structure: PdfStructure = {
          filePath: 'dummy.pdf',
          metadata: {},
          pages: [
            {
              pageNumber: 1,
              width: 800,
              height: 1000,
              images: [],
              textBlocks: [],
            },
          ],
        };
        return Promise.resolve(structure);
      }
      return Promise.resolve([]);
    });
    mockedReadFile.mockResolvedValue(new Uint8Array([1, 2, 3]));
    mockExtractTextWithPDFjs.mockResolvedValue([
      { id: 'text_1_0', x: 0, y: 0, width: 10, height: 10, text: 'hi', fontSize: 12, lineHeight: 1.2, fontFamily: 'Arial' },
    ]);
    mockExtractImageAreasWithPDFjs.mockResolvedValue([]);

    await usePdfStore.getState().loadPdf('dummy.pdf');

    const state = usePdfStore.getState();
    expect(state.selectedImageId).toBeNull();
    expect(state.selectedTextBlockId).toBeNull();
    expect(state.currentPage).toBe(1);
  });

  it('ページ削除後にcurrentPageが存在するページ内にクランプされる', async () => {
    const mockedInvoke = invoke as unknown as Mock;

    const structure: PdfStructure = {
      filePath: 'dummy.pdf',
      metadata: {},
      pages: [
        { pageNumber: 1, width: 800, height: 1000, images: [], textBlocks: [] },
        { pageNumber: 2, width: 800, height: 1000, images: [], textBlocks: [] },
      ],
    };

    mockedInvoke.mockImplementation((cmd: string, args: any) => {
      if (cmd === 'delete_page') {
        return Promise.resolve({
          ...structure,
          pages: structure.pages.slice(0, 1),
        });
      }
      return Promise.resolve(structure);
    });

    usePdfStore.setState({
      pdfStructure: structure,
      currentPage: 2,
    });

    await usePdfStore.getState().deletePage(2);

    const state = usePdfStore.getState();
    expect(state.pdfStructure?.pages).toHaveLength(1);
    expect(state.currentPage).toBe(1);
    expect(state.selectedImageId).toBeNull();
    expect(state.selectedTextBlockId).toBeNull();
  });

  it('最後のページを削除したらpdfStructureをクリアする', async () => {
    const mockedInvoke = invoke as unknown as Mock;

    const structure: PdfStructure = {
      filePath: 'dummy.pdf',
      metadata: {},
      pages: [
        { pageNumber: 1, width: 800, height: 1000, images: [], textBlocks: [] },
      ],
    };

    mockedInvoke.mockImplementation((cmd: string, args: any) => {
      if (cmd === 'delete_page') {
        return Promise.resolve({
          ...structure,
          pages: [],
        });
      }
      return Promise.resolve(structure);
    });

    usePdfStore.setState({
      pdfStructure: structure,
      currentPage: 1,
    });

    await usePdfStore.getState().deletePage(1);

    const state = usePdfStore.getState();
    expect(state.pdfStructure).toBeNull();
    expect(state.currentPage).toBe(1);
    expect(state.selectedImageId).toBeNull();
    expect(state.selectedTextBlockId).toBeNull();
  });

  it('最後のページを削除したときにズームをリセットする', async () => {
    const mockedInvoke = invoke as unknown as Mock;

    const structure: PdfStructure = {
      filePath: 'dummy.pdf',
      metadata: {},
      pages: [
        { pageNumber: 1, width: 800, height: 1000, images: [], textBlocks: [] },
      ],
    };

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'delete_page') {
        return Promise.resolve({ ...structure, pages: [] });
      }
      return Promise.resolve(structure);
    });

    usePdfStore.setState({
      pdfStructure: structure,
      currentPage: 1,
      zoomLevel: 1.5,
    });

    await usePdfStore.getState().deletePage(1);

    const state = usePdfStore.getState();
    expect(state.pdfStructure).toBeNull();
    expect(state.zoomLevel).toBe(1);
    expect(state.currentPage).toBe(1);
  });

  it('ページ並び替え後に選択状態をクリアし、currentPageをクランプする', async () => {
    const mockedInvoke = invoke as unknown as Mock;

    const structure: PdfStructure = {
      filePath: 'dummy.pdf',
      metadata: {},
      pages: [
        { pageNumber: 1, width: 800, height: 1000, images: [], textBlocks: [] },
        { pageNumber: 2, width: 800, height: 1000, images: [], textBlocks: [] },
      ],
    };

    mockedInvoke.mockImplementation((cmd: string, args: any) => {
      if (cmd === 'reorder_pages') {
        return Promise.resolve(structure);
      }
      return Promise.resolve(structure);
    });

    usePdfStore.setState({
      pdfStructure: structure,
      selectedImageId: 'img-1',
      selectedTextBlockId: 'text-1',
      currentPage: 3,
    });

    await usePdfStore.getState().reorderPages(1, 2);

    const state = usePdfStore.getState();
    expect(state.selectedImageId).toBeNull();
    expect(state.selectedTextBlockId).toBeNull();
    expect(state.currentPage).toBe(2);
  });

  it('ページ並び替えでtoIndex=0の場合はエラーになる', async () => {
    const structure: PdfStructure = {
      filePath: 'dummy.pdf',
      metadata: {},
      pages: [
        { pageNumber: 1, width: 800, height: 1000, images: [], textBlocks: [] },
        { pageNumber: 2, width: 800, height: 1000, images: [], textBlocks: [] },
      ],
    };

    usePdfStore.setState({
      pdfStructure: structure,
    });

    await expect(usePdfStore.getState().reorderPages(1, 0)).rejects.toThrow('無効な終了インデックスです');
  });

  it('画像を移動するとその画像を再選択しテキスト選択を解除する', async () => {
    const mockedInvoke = invoke as unknown as Mock;

    const structure: PdfStructure = {
      filePath: 'dummy.pdf',
      metadata: {},
      pages: [
        {
          pageNumber: 1,
          width: 800,
          height: 1000,
          images: [{ id: 'img-1', x: 0, y: 0, width: 10, height: 10, originalWidth: 10, originalHeight: 10, data: '', format: 'png' }],
          textBlocks: [],
        },
      ],
    };

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'move_image') {
        return Promise.resolve(structure);
      }
      return Promise.resolve(structure);
    });

    usePdfStore.setState({
      pdfStructure: structure,
      selectedTextBlockId: 'text-1',
      selectedImageId: null,
    });

    await usePdfStore.getState().moveImage('img-1', 5, 5);

    const state = usePdfStore.getState();
    expect(state.selectedImageId).toBe('img-1');
    expect(state.selectedTextBlockId).toBeNull();
  });

  it('テキストブロックを移動するとそのテキストを再選択し画像選択を解除する', async () => {
    const mockedInvoke = invoke as unknown as Mock;

    const structure: PdfStructure = {
      filePath: 'dummy.pdf',
      metadata: {},
      pages: [
        {
          pageNumber: 1,
          width: 800,
          height: 1000,
          images: [],
          textBlocks: [{ id: 'text-1', x: 0, y: 0, width: 10, height: 10, text: 'hi', fontSize: 12, lineHeight: 1.2, fontFamily: 'Arial' }],
        },
      ],
    };

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'move_text_block') {
        return Promise.resolve(structure);
      }
      return Promise.resolve(structure);
    });

    usePdfStore.setState({
      pdfStructure: structure,
      selectedTextBlockId: null,
      selectedImageId: 'img-1',
    });

    await usePdfStore.getState().moveTextBlock('text-1', 5, 5);

    const state = usePdfStore.getState();
    expect(state.selectedTextBlockId).toBe('text-1');
    expect(state.selectedImageId).toBeNull();
  });

  it('setCurrentPage clamps to valid range based on pdfStructure length', () => {
    usePdfStore.setState({
      pdfStructure: {
        filePath: 'dummy.pdf',
        metadata: {},
        pages: [
          { pageNumber: 1, width: 800, height: 1000, images: [], textBlocks: [] },
          { pageNumber: 2, width: 800, height: 1000, images: [], textBlocks: [] },
        ],
      } as PdfStructure,
      currentPage: 1,
    });

    usePdfStore.getState().setCurrentPage(0);
    expect(usePdfStore.getState().currentPage).toBe(1);

    usePdfStore.getState().setCurrentPage(5);
    expect(usePdfStore.getState().currentPage).toBe(2);
  });

  it('setCurrentPage clamps using page numbers when they are not contiguous', () => {
    usePdfStore.setState({
      pdfStructure: {
        filePath: 'dummy.pdf',
        metadata: {},
        pages: [
          { pageNumber: 1, width: 800, height: 1000, images: [], textBlocks: [] },
          { pageNumber: 3, width: 800, height: 1000, images: [], textBlocks: [] },
        ],
      } as PdfStructure,
      currentPage: 1,
    });

    usePdfStore.getState().setCurrentPage(5);
    expect(usePdfStore.getState().currentPage).toBe(3);
    usePdfStore.getState().setCurrentPage(0);
    expect(usePdfStore.getState().currentPage).toBe(1);
    usePdfStore.getState().setCurrentPage(2);
    // nearest existing page is the closest lower page when exact match is missing
    expect(usePdfStore.getState().currentPage).toBe(1);
    usePdfStore.getState().setCurrentPage(4);
    // chooses closest existing page <= requested, so falls back to page 3
    expect(usePdfStore.getState().currentPage).toBe(3);
  });

  it('setCurrentPage clears selections not on the target page', () => {
    usePdfStore.setState({
      pdfStructure: {
        filePath: 'dummy.pdf',
        metadata: {},
        pages: [
          { pageNumber: 1, width: 800, height: 1000, images: [{ id: 'img-1', x: 0, y: 0, width: 10, height: 10, originalWidth: 10, originalHeight: 10, data: '', format: 'png' }], textBlocks: [] },
          { pageNumber: 2, width: 800, height: 1000, images: [], textBlocks: [{ id: 'text-1', x: 0, y: 0, width: 10, height: 10, text: 'hi', fontSize: 12, lineHeight: 1.2, fontFamily: 'Arial' }] },
        ],
      } as PdfStructure,
      currentPage: 1,
      selectedImageId: 'img-1',
      selectedTextBlockId: 'text-1',
    });

    usePdfStore.getState().setCurrentPage(2);

    expect(usePdfStore.getState().currentPage).toBe(2);
    expect(usePdfStore.getState().selectedImageId).toBeNull();
    expect(usePdfStore.getState().selectedTextBlockId).toBe('text-1');
  });

  it('loadPdf resets zoom level to 1 on start, success, and failure', async () => {
    const mockedInvoke = invoke as unknown as Mock;
    const mockedReadFile = readFile as unknown as Mock;

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'load_pdf') {
        const structure: PdfStructure = {
          filePath: 'dummy.pdf',
          metadata: {},
          pages: [
            {
              pageNumber: 1,
              width: 800,
              height: 1000,
              images: [],
              textBlocks: [],
            },
          ],
        };
        return Promise.resolve(structure);
      }
      return Promise.resolve([]);
    });
    mockedReadFile.mockResolvedValue(new Uint8Array([1, 2, 3]));
    mockExtractTextWithPDFjs.mockResolvedValue([]);
    mockExtractImageAreasWithPDFjs.mockResolvedValue([]);

    usePdfStore.setState({ zoomLevel: 1.8 });
    await usePdfStore.getState().loadPdf('dummy.pdf');
    expect(usePdfStore.getState().zoomLevel).toBe(1);

    mockedInvoke.mockRejectedValueOnce(new Error('fail'));
    usePdfStore.setState({ zoomLevel: 1.5 });
    await expect(usePdfStore.getState().loadPdf('dummy.pdf')).rejects.toThrow();
    expect(usePdfStore.getState().zoomLevel).toBe(1);
  });

  it('ブラウザモードではMarkdownプレビュー生成をスキップする', async () => {
    const mockedInvoke = invoke as unknown as Mock;
    const tauriWindow = globalThis.window as Window & { __TAURI__?: unknown; __TAURI_INTERNALS__?: unknown };
    delete tauriWindow.__TAURI__;
    delete tauriWindow.__TAURI_INTERNALS__;

    const result = await usePdfStore.getState().requestMarkdownPreview('# test', 1);

    expect(result).toBeNull();
    expect(mockedInvoke).not.toHaveBeenCalled();
    expect(usePdfStore.getState().previewError).toContain('ブラウザモードではPDFライブプレビューは利用できません');
  });

  it('TauriランタイムではMarkdownプレビュー生成コマンドを呼び出す', async () => {
    const mockedInvoke = invoke as unknown as Mock;
    const tauriWindow = globalThis.window as Window & { __TAURI__?: unknown; __TAURI_INTERNALS__?: unknown };
    tauriWindow.__TAURI__ = {};
    mockedInvoke.mockResolvedValue('C:/temp/preview.pdf');

    const result = await usePdfStore.getState().requestMarkdownPreview('# ok', 2);

    expect(result).toBe('C:/temp/preview.pdf');
    expect(mockedInvoke).toHaveBeenCalledWith('render_markdown_to_pdf_preview', {
      markdown: '# ok',
    });
    expect(usePdfStore.getState().previewPdfPath).toBe('C:/temp/preview.pdf');
    expect(usePdfStore.getState().previewError).toBeNull();

    delete tauriWindow.__TAURI__;
  });
});

