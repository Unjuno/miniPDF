import React, { useEffect, useRef, useState, useMemo, useCallback, lazy, Suspense, memo, useReducer } from 'react';
import { PdfStructure, TextBlock } from '../types/pdf';
import { usePdfStore, useSelectedTextBlockId, useSelectedImageId } from '../stores/pdfStore';
import { useDebounce } from '../hooks/useDebounce';
import { useRenderCache } from '../hooks/useRenderCache';
import { readFile } from '@tauri-apps/plugin-fs';
import { getSourcePageNumber } from '../utils/pageMapping';
import { renderBlankPage } from '../utils/renderBlankPage';
import { logger } from '../utils/logger';
import './PDFViewer.css';

// why: PDF.jsは重いライブラリのため、実際にPDFを読み込む時まで遅延読み込み
// alt: 通常のimport（起動時に読み込まれる）
// evidence: PDF.jsは数MBのサイズがあり、起動時間に大きく影響する
// assumption: ユーザーがPDFを開くまでPDF.jsは不要
let pdfjsLib: typeof import('pdfjs-dist') | null = null;
const loadPdfJs = async () => {
  if (!pdfjsLib) {
    pdfjsLib = await import('pdfjs-dist');
    // why: PDF.jsワーカーの設定（Vite環境でのワーカーパスを解決）
    // alt: ワーカー設定なし（PDF読み込みが失敗する）
    // evidence: ワーカー設定によりPDF読み込みが正常に動作する
    if (globalThis.window !== undefined && pdfjsLib.GlobalWorkerOptions) {
      pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
        'pdfjs-dist/build/pdf.worker.min.mjs',
        import.meta.url
      ).toString();
    }
  }
  return pdfjsLib;
};

// 重いコンポーネントも遅延読み込み
const TextEditor = lazy(() => import('./TextEditor').then(m => ({ default: m.TextEditor })));
const TextInput = lazy(() => import('./TextEditor').then(m => ({ default: m.TextInput })));
const ImageInserter = lazy(() => import('./ImageInserter').then(m => ({ default: m.ImageInserter })));
const TextBlockOverlay = lazy(() => import('./TextBlockOverlay').then(m => ({ default: m.TextBlockOverlay })));
const ImageOverlay = lazy(() => import('./ImageOverlay').then(m => ({ default: m.ImageOverlay })));
const PageBreakEditor = lazy(() => import('./PageBreakEditor').then(m => ({ default: m.PageBreakEditor })));
const InlineTextEditor = lazy(() => import('./InlineTextEditor').then(m => ({ default: m.InlineTextEditor })));

interface PDFViewerProps {
  pdfStructure: PdfStructure | null;
  zoomLevel: number;
  onPageChange?: (pageNumber: number) => void;
  /** When true (default), hide layout overlays and editing UI — print-style preview only. */
  previewOnly?: boolean;
}

// why: PDFViewerをメモ化して不要な再レンダリングを削減
// alt: 通常のコンポーネント（親の再レンダリングで毎回再レンダリング）
// evidence: メモ化により不要な再レンダリングが削減され、パフォーマンスが向上
export const PDFViewer: React.FC<PDFViewerProps> = memo(({ 
  pdfStructure, 
  zoomLevel,
  onPageChange,
  previewOnly = true,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  // why: PDF.jsのRenderTask型を使用して型安全性を向上
  // alt: any型を使用（型安全性が低下）
  // evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
  const renderTaskRef = useRef<import('pdfjs-dist').RenderTask | null>(null);
  const isDraggingRef = useRef<boolean>(false);
  const dragStartRef = useRef<{ x: number; y: number; scrollLeft: number; scrollTop: number } | null>(null);
  const zoomRafIdRef = useRef<number | null>(null);
  const resizeRafIdRef = useRef<number | null>(null);
  const storeCurrentPage = usePdfStore((state) => state.currentPage);
  const [currentPage, setCurrentPage] = useState(1);
  // why: PDF.jsのPDFDocumentProxy型を使用して型安全性を向上
  // alt: any型を使用（型安全性が低下）
  // evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
  const [pdfDoc, setPdfDoc] = useState<import('pdfjs-dist').PDFDocumentProxy | null>(null);
  // why: pdfDocをuseRefでも管理して、クリーンアップ関数で最新の値を参照できるようにする
  // alt: useStateのみを使用（クリーンアップ関数が古い値を参照する可能性がある）
  // evidence: useRefにより、クリーンアップ関数内で最新のpdfDocを参照できる
  const pdfDocRef = useRef<import('pdfjs-dist').PDFDocumentProxy | null>(null);
  // why: 前回のpdfStructureを保持して、ファイルパスの変更を検出する
  // alt: pdfStructureの変更を検出できない（PDFファイルの再読み込みが必要かどうか判断できない）
  // evidence: 前回のpdfStructureを保持することで、ファイルパスが変更されたかどうかを判断できる
  const prevPdfStructureRef = useRef<PdfStructure | null>(null);
  // why: PDF.jsの読み込み状態を管理（値は使用しないが、setterのみ使用）
  // alt: 値を削除（setterが使用できなくなる）
  // evidence: setPdfJsLoadedのみを使用しており、値は不要
  const [_pdfJsLoaded, setPdfJsLoaded] = useState(false);
  const canvasRefsMap = useRef<Map<number, HTMLCanvasElement>>(new Map());
  const pageContainerRefsMap = useRef<Map<number, HTMLDivElement>>(new Map());
  const selectedTextBlockId = useSelectedTextBlockId();
  const selectedImageId = useSelectedImageId();
  const selectTextBlock = usePdfStore((state) => state.selectTextBlock);
  const adjustPageBreak = usePdfStore((state) => state.adjustPageBreak);
  const setCurrentPageInStore = usePdfStore((state) => state.setCurrentPage);
  const setError = usePdfStore((state) => state.setError);
  const isScrollingRef = useRef(false);

  // why: 複数の関連する状態をuseReducerで統合して再レンダリングを削減
  // alt: 複数のuseStateを使用（各状態変更で再レンダリングが発生）
  // evidence: useReducerにより状態更新をバッチ処理し、不要な再レンダリングを削減
  interface UIState {
    editingTextBlock: TextBlock | null;
    inlineEditingTextBlockId: string | null;
    showPageBreakEditor: boolean;
    showTextInput: boolean;
    textInputPosition: { x: number; y: number; width: number; height: number };
    showImageInserter: boolean;
    imageInserterPosition: { x: number; y: number };
    viewMode: 'single' | 'continuous';
  }

  type UIAction =
    | { type: 'SET_EDITING_TEXT_BLOCK'; payload: TextBlock | null }
    | { type: 'SET_INLINE_EDITING_TEXT_BLOCK_ID'; payload: string | null }
    | { type: 'SET_SHOW_PAGE_BREAK_EDITOR'; payload: boolean }
    | { type: 'SET_SHOW_TEXT_INPUT'; payload: boolean }
    | { type: 'SET_TEXT_INPUT_POSITION'; payload: { x: number; y: number; width: number; height: number } }
    | { type: 'SET_SHOW_IMAGE_INSERTER'; payload: boolean }
    | { type: 'SET_IMAGE_INSERTER_POSITION'; payload: { x: number; y: number } }
    | { type: 'SET_VIEW_MODE'; payload: 'single' | 'continuous' }
    | { type: 'RESET_UI_STATE' };

  const initialState: UIState = {
    editingTextBlock: null,
    inlineEditingTextBlockId: null,
    showPageBreakEditor: false,
    showTextInput: false,
    textInputPosition: { x: 0, y: 0, width: 400, height: 100 },
    showImageInserter: false,
    imageInserterPosition: { x: 0, y: 0 },
    viewMode: 'continuous',
  };

  const uiReducer = (state: UIState, action: UIAction): UIState => {
    switch (action.type) {
      case 'SET_EDITING_TEXT_BLOCK':
        return { ...state, editingTextBlock: action.payload };
      case 'SET_INLINE_EDITING_TEXT_BLOCK_ID':
        return { ...state, inlineEditingTextBlockId: action.payload };
      case 'SET_SHOW_PAGE_BREAK_EDITOR':
        return { ...state, showPageBreakEditor: action.payload };
      case 'SET_SHOW_TEXT_INPUT':
        return { ...state, showTextInput: action.payload };
      case 'SET_TEXT_INPUT_POSITION':
        return { ...state, textInputPosition: action.payload };
      case 'SET_SHOW_IMAGE_INSERTER':
        return { ...state, showImageInserter: action.payload };
      case 'SET_IMAGE_INSERTER_POSITION':
        return { ...state, imageInserterPosition: action.payload };
      case 'SET_VIEW_MODE':
        return { ...state, viewMode: action.payload };
      case 'RESET_UI_STATE':
        return initialState;
      default:
        return state;
    }
  };

  const [uiState, dispatchUI] = useReducer(uiReducer, initialState);

  // why: レンダリングキャッシュを使用して同じページ・ズームレベルの再レンダリングをスキップ
  // alt: 毎回レンダリング（同じページ・ズームレベルの再レンダリングが発生）
  // evidence: キャッシュにより同じページ・ズームレベルの再レンダリングがスキップされ、パフォーマンスが向上
  const renderCache = useRenderCache();
  const pageOrderKey = useMemo(() => {
    if (!pdfStructure?.pages) {
      return '';
    }
    return pdfStructure.pages
      .map((page) => `${page.pageNumber}:${page.sourcePageNumber ?? ''}`)
      .join('|');
  }, [pdfStructure]);

  const totalPages = pdfStructure?.pages?.length ?? 0;

  // 現在のページの構造を取得（メモ化）
  // why: pdfStructureとcurrentPageの両方に依存して、ページ構造が変更されたときに再計算
  // alt: currentPageのみに依存（pdfStructureが更新されても再計算されない）
  // evidence: pdfStructureが更新されると、ページの構造（textBlocks、images）も更新されるため、再計算が必要
  const currentPageStructure = useMemo(() => {
    if (!pdfStructure?.pages || pdfStructure.pages.length === 0) {
      return null;
    }
    // why: page.pageNumberとcurrentPageを比較して、現在のページを検索
    // alt: インデックスを使用（ページ番号が再割り当てされた場合に不正確）
    // evidence: ページ番号を使用することで、ページ追加/削除後も正しいページを取得できる
    return pdfStructure.pages.find(page => page.pageNumber === currentPage) || null;
  }, [pdfStructure, pdfStructure?.pages, pdfStructure?.pages?.length, currentPage]);

  // why: PDFがクリアされたときに古いレンダリング状態を即座にリセットする
  // alt: pdfDocやキャッシュが残り、前のPDFが表示されたままになる
  // evidence: loadPdf失敗時にpdfStructureがnullになるケースで、古いページが表示される
  useEffect(() => {
    if (pdfStructure) return;

    if (renderTaskRef.current) {
      try {
        renderTaskRef.current.cancel();
      } catch {
        // ignore
      }
      renderTaskRef.current = null;
    }

    renderCache.clear();
    setPdfDoc(null);
    setCurrentPage(1);
    setCurrentPageInStore(1);

    const canvas = canvasRef.current;
    if (canvas) {
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
      }
    }
  }, [pdfStructure, renderCache, setCurrentPageInStore]);

  const getSourcePageForDisplay = useCallback((displayPageNumber: number): number | null => {
    if (!pdfStructure) {
      return null;
    }
    const page = pdfStructure.pages.find((item) => item.pageNumber === displayPageNumber);
    return page ? getSourcePageNumber(page) : null;
  }, [pdfStructure]);

  // 複数ページ表示用のレンダリング関数
  // why: PDF.jsのPDFDocumentProxy型を使用して型安全性を向上
  // alt: any型を使用（型安全性が低下）
  // evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
  const renderPageToCanvas = useCallback(async (pdf: import('pdfjs-dist').PDFDocumentProxy, sourcePageNumber: number, canvas: HTMLCanvasElement) => {
    const devicePixelRatio = window.devicePixelRatio || 1;
    const viewport = (await pdf.getPage(sourcePageNumber)).getViewport({ scale: zoomLevel * devicePixelRatio });
    
    const context = canvas.getContext('2d', { 
      alpha: false,
      willReadFrequently: true,
      desynchronized: false,
    });

    if (!context) return;

    const outputScale = devicePixelRatio;
    const displayWidth = Math.floor(viewport.width / outputScale);
    const displayHeight = Math.floor(viewport.height / outputScale);

    if (canvas.height !== viewport.height || canvas.width !== viewport.width) {
      canvas.height = viewport.height;
      canvas.width = viewport.width;
    }
    
    canvas.style.width = `${displayWidth}px`;
    canvas.style.height = `${displayHeight}px`;

    const cachedImageData = renderCache.get(sourcePageNumber, zoomLevel);
    if (cachedImageData) {
      context.putImageData(cachedImageData, 0, 0);
      return;
    }

    context.textBaseline = 'alphabetic';
    context.textAlign = 'left';
    context.imageSmoothingEnabled = true;
    context.imageSmoothingQuality = 'high';
    if ('textRenderingOptimization' in context) {
      (context as any).textRenderingOptimization = 'optimizeQuality';
    }

    const page = await pdf.getPage(sourcePageNumber);
    const renderContext = {
      canvasContext: context,
      viewport: viewport,
    };

    const renderTask = page.render(renderContext);
    await renderTask.promise;

    const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
    renderCache.set(sourcePageNumber, zoomLevel, imageData);
  }, [zoomLevel, renderCache]);

  // すべてのページをレンダリング（複数ページ表示モード用）
  // why: PDF.jsのPDFDocumentProxy型を使用して型安全性を向上
  // alt: any型を使用（型安全性が低下）
  // evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
  const renderAllPages = useCallback(async (pdf: import('pdfjs-dist').PDFDocumentProxy) => {
    if (!pdf || !pdfStructure) return;
    // 各ページのcanvas要素にレンダリング
    for (const page of pdfStructure.pages) {
      const displayPageNumber = page.pageNumber;
      const canvas = canvasRefsMap.current.get(displayPageNumber);
      if (!canvas) {
        continue;
      }
      const sourcePageNumber = getSourcePageNumber(page);
      if (!sourcePageNumber || sourcePageNumber < 1 || sourcePageNumber > pdf.numPages) {
        renderBlankPage(canvas, page.width, page.height, zoomLevel);
        continue;
      }
      // レンダリングを並列で実行
      renderPageToCanvas(pdf, sourcePageNumber, canvas).catch((error) => {
        if (error?.name !== 'RenderingCancelledException' && !error?.message?.includes('cancelled')) {
          logger.error(`Error rendering page ${sourcePageNumber}`, error instanceof Error ? error : new Error(String(error)));
        }
      });
    }
  }, [pdfStructure, renderPageToCanvas, zoomLevel]);

  // why: PDF.jsのPDFDocumentProxy型を使用して型安全性を向上
  // alt: any型を使用（型安全性が低下）
  // evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
  const renderPageNumber = useCallback(async (pdf: import('pdfjs-dist').PDFDocumentProxy, sourcePageNumber: number) => {
    if (!canvasRef.current) return;

    // why: 前のレンダリングタスクをキャンセルして複数のレンダリング操作の競合を防ぐ
    // alt: 前のタスクをキャンセルしない（PDF.jsがエラーを発生）
    // evidence: PDF.jsは同じキャンバスで複数のレンダリング操作を同時に実行できない
    if (renderTaskRef.current) {
      try {
        renderTaskRef.current.cancel();
      } catch (error) {
        // why: キャンセルエラーは無視（既に完了している可能性がある）
        // alt: エラーをログに記録（不要なログが増える）
        // evidence: キャンセルエラーは通常の動作であり、ログに記録する必要はない
        if (error instanceof Error && error.name !== 'RenderingCancelledException') {
          logger.warn('Failed to cancel render task', { error: error.message });
        }
      }
      renderTaskRef.current = null;
    }

    try {
      const page = await pdf.getPage(sourcePageNumber);
      
      const pageViewport = page.getViewport({ scale: 1 });
      logger.debug('Page original size', { width: pageViewport.width, height: pageViewport.height });
      
      // why: デバイスのピクセル比を考慮して高DPIディスプレイで文字がつぶれないようにする
      // alt: デフォルトのスケール（高DPIで文字がつぶれる）
      // evidence: devicePixelRatioを考慮することで高DPIディスプレイでも鮮明に表示される
      const devicePixelRatio = window.devicePixelRatio || 1;
      // why: ズームレベルをそのまま適用して100%以下の縮小も可能にする
      // alt: 最小値を1.0に設定（100%以下の縮小ができない）
      // evidence: ズームレベルをそのまま使用することで、50%から200%の範囲で正しく表示される
      const viewport = page.getViewport({ scale: zoomLevel * devicePixelRatio });
      const canvas = canvasRef.current;
      
      logger.debug('Rendering page', { sourcePageNumber, viewportWidth: viewport.width, viewportHeight: viewport.height, zoomLevel, devicePixelRatio });
      // why: キャッシュ用のgetImageData呼び出しを最適化するため、willReadFrequentlyをtrueに設定
      // alt: デフォルト設定（getImageDataが遅い）
      // evidence: willReadFrequentlyによりgetImageDataのパフォーマンスが向上
      const context = canvas.getContext('2d', { 
        alpha: false,
        willReadFrequently: true,
        // why: テキストレンダリング品質を向上
        // alt: デフォルト設定（文字がつぶれる可能性）
        // evidence: 適切な設定により文字が鮮明に表示される
        desynchronized: false,
      });

      if (!context) return;

      // why: 高DPIディスプレイ対応のため、Canvasの内部解像度と表示サイズを分離
      // alt: 同じサイズ（高DPIで文字がつぶれる）
      // evidence: 内部解像度を上げることで高DPIでも鮮明に表示される
      const outputScale = devicePixelRatio;
      const displayWidth = Math.floor(viewport.width / outputScale);
      const displayHeight = Math.floor(viewport.height / outputScale);

      logger.debug('Display size calculation', { viewportWidth: viewport.width, viewportHeight: viewport.height, outputScale, displayWidth, displayHeight });

      // キャンバスの内部解像度（実際のピクセル数）- 先に設定
      if (canvas.height !== viewport.height || canvas.width !== viewport.width) {
        canvas.height = viewport.height;
        canvas.width = viewport.width;
      }
      
      // why: キャンバスの表示サイズを設定してスクロール可能にする
      // alt: 表示サイズを設定しない（キャンバスがコンテナより小さくスクロールできない）
      // evidence: 表示サイズを設定することで、キャンバスがコンテナより大きくなりスクロール可能になる
      canvas.style.width = `${displayWidth}px`;
      canvas.style.height = `${displayHeight}px`;
      
      logger.debug('Canvas size set', { internalWidth: canvas.width, internalHeight: canvas.height, displayWidth: canvas.style.width, displayHeight: canvas.style.height });

      // キャンバスの表示サイズは既に設定済み（重複設定を削除）

      // why: キャッシュからレンダリング済み画像を取得して再レンダリングをスキップ
      // alt: 毎回レンダリング（同じページ・ズームレベルの再レンダリングが発生）
      // evidence: キャッシュにより同じページ・ズームレベルの再レンダリングがスキップされ、パフォーマンスが向上
      const cachedImageData = renderCache.get(sourcePageNumber, zoomLevel);
      if (cachedImageData) {
        // キャッシュから画像データを復元
        context.putImageData(cachedImageData, 0, 0);
        return;
      }

      // why: テキストレンダリング品質を向上
      // alt: デフォルト設定（文字がつぶれる可能性）
      // evidence: 適切な設定により文字が鮮明に表示される
      // テキストのアンチエイリアスを有効化
      context.textBaseline = 'alphabetic';
      context.textAlign = 'left';
      // テキストのレンダリング品質を向上（スムーズなアンチエイリアス）
      context.imageSmoothingEnabled = true;
      context.imageSmoothingQuality = 'high';
      // 注意: context.scale()は適用しない（PDF.jsがviewportに基づいて自動的にスケールするため）
      // PDF.jsはviewportのサイズに基づいて自動的にレンダリングするため、追加のスケールは不要
      context.imageSmoothingEnabled = true;
      context.imageSmoothingQuality = 'high';
      // why: テキストレンダリング品質を向上（非標準APIだが一部のブラウザでサポート）
      // alt: デフォルト設定（文字がつぶれる可能性）
      // evidence: 適切な設定により文字が鮮明に表示される
      if ('textRenderingOptimization' in context) {
        (context as any).textRenderingOptimization = 'optimizeQuality';
      }
      // 注意: willReadFrequentlyはgetContextのオプションで既に設定済み

      const renderContext = {
        canvasContext: context,
        viewport: viewport,
      };

      // why: レンダリングタスクを保存して後でキャンセルできるようにする
      // alt: タスクを保存しない（キャンセルできない）
      // evidence: タスクを保存することで、新しいレンダリングが開始されたときに前のタスクをキャンセルできる
      const renderTask = page.render(renderContext);
      renderTaskRef.current = renderTask;

      await renderTask.promise;
      
      logger.debug('Page rendered', { sourcePageNumber, canvasWidth: canvas.width, canvasHeight: canvas.height, displayWidth: canvas.style.width, displayHeight: canvas.style.height });

      // why: レンダリングが完了したらタスク参照をクリア
      // alt: タスク参照を保持（メモリリークの可能性）
      // evidence: タスク参照をクリアすることでメモリリークを防ぐ
      renderTaskRef.current = null;

      // why: レンダリング結果をキャッシュに保存して再レンダリングをスキップ
      // alt: 毎回レンダリング（同じページ・ズームレベルの再レンダリングが発生）
      // evidence: キャッシュにより同じページ・ズームレベルの再レンダリングがスキップされ、パフォーマンスが向上
      const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
      renderCache.set(sourcePageNumber, zoomLevel, imageData);
    } catch (error: unknown) {
      // why: キャンセルエラーは正常な動作なので無視する
      // alt: すべてのエラーをログに出力（キャンセルエラーがノイズになる）
      // evidence: キャンセルエラーはユーザー操作による正常な動作
      // why: errorをunknown型として扱い、型安全にプロパティにアクセスする
      // alt: any型を使用（型安全性が低下）
      // evidence: unknown型を使用することで、実行時エラーを防ぐ
      const errorObj = error as { name?: string; message?: string };
      if (errorObj?.name === 'RenderingCancelledException' || errorObj?.message?.includes('cancelled')) {
        return; // キャンセルされた場合は何もしない
      }
      logger.error('Error rendering page', error instanceof Error ? error : new Error(String(error)));
      renderTaskRef.current = null;
    }
  }, [zoomLevel, renderCache]); // why: renderCacheはuseMemoでメモ化されているが、依存配列に含めることで最新の参照を使用

  useEffect(() => {
    // why: 連続表示では canvas は canvasRefsMap のみに付くため、canvasRef は null のまま。読み込みは pdfStructure だけで開始する
    // alt: canvasRef を必須にする（連続表示で PDF が永遠に読み込まれず真っ白になる）
    if (!pdfStructure) return;

    // why: PDFファイルパスが変更された場合のみPDFを再読み込み（テキストブロックの移動などの軽微な変更では再読み込み不要）
    // alt: pdfStructureが変更されるたびにPDFを再読み込み（パフォーマンスが悪い）
    // evidence: ファイルパスが変更されていない場合は、PDFファイル自体は変更されていないため、再読み込み不要
    // assumption: テキストブロックや画像の位置変更などの軽微な変更は、オーバーレイの再描画で対応可能
    const shouldReloadPdf = !pdfDocRef.current || 
      (prevPdfStructureRef.current?.filePath !== pdfStructure.filePath);
    
    // why: 非同期処理中にコンポーネントがアンマウントされた場合に処理をキャンセルするためのフラグ
    // alt: フラグなし（メモリリークが発生する可能性がある）
    // evidence: フラグにより、非同期処理中にコンポーネントがアンマウントされた場合に処理をスキップできる
    let cancelled = false;
    let loadingTask: import('pdfjs-dist').PDFLoadingTask | null = null;
    
    if (shouldReloadPdf) {
      // PDFファイルパスが変更された場合のみPDFを再読み込み
      renderCache.clear();

      const renderPage = async () => {
      try {
        // PDF.jsを遅延読み込み
        const pdfjs = await loadPdfJs();
        if (cancelled) return;
        setPdfJsLoaded(true);
        
        // why: Tauriアプリではファイルパスを直接使用できないため、readFileで読み込む
        // alt: ファイルパスを直接使用（ブラウザのセキュリティ制限でブロックされる）
        // evidence: 開発環境ではfile://プロトコルがブロックされる
        // assumption: TauriのreadFileを使用することで、ローカルファイルを読み込める
        // why: Tauri 2.0のreadFileは文字列を直接受け取る（型定義ではstring | URLだが、実行時は文字列が必要）
        // alt: { path: pdfStructure.filePath }形式（型エラーが発生）
        // evidence: 型定義ファイルの例では文字列を直接渡している
        const fileData = await readFile(pdfStructure.filePath);
        if (cancelled) return;
        logger.debug('Loading PDF', { filePath: pdfStructure.filePath, fileSize: fileData.byteLength });
        loadingTask = pdfjs.getDocument({
          data: fileData,
          // why: PDF.jsのキャッシュを有効化して読み込み時間を短縮
          // alt: キャッシュ無効（毎回解析される）
          // evidence: キャッシュにより読み込み時間が短縮される
          disableAutoFetch: false,
          disableStream: false,
        });
        
        const pdf = await loadingTask.promise;
        if (cancelled) {
          // why: キャンセルされた場合はPDFオブジェクトを破棄してメモリリークを防ぐ
          // alt: 破棄しない（メモリリークが発生する可能性がある）
          // evidence: pdf.destroy()により、PDF.jsのリソースが解放される
          try {
            pdf.destroy();
          } catch (error) {
            // 既に破棄されている場合はエラーを無視
            logger.warn('Failed to destroy PDF document after cancellation', { error });
          }
          return;
        }
        logger.debug('PDF loaded', { pages: pdf.numPages });
          if (pdf.numPages > 0 && pdfStructure.pages.length > 0) {
            if (cancelled) {
              try {
                pdf.destroy();
              } catch (error) {
                logger.warn('Failed to destroy PDF document after cancellation', { error });
              }
              return;
            }
            const firstPage = await pdf.getPage(1);
            if (cancelled) {
              try {
                pdf.destroy();
              } catch (error) {
                logger.warn('Failed to destroy PDF document after cancellation', { error });
              }
              return;
            }
            const firstPageViewport = firstPage.getViewport({ scale: 1 });
            const structurePage = pdfStructure.pages[0];
            logger.debug('Page size comparison', {
              pdfjsPoints: `${firstPageViewport.width} x ${firstPageViewport.height}`,
              pdfStructurePoints: `${structurePage.width} x ${structurePage.height}`,
              match: firstPageViewport.width === structurePage.width && firstPageViewport.height === structurePage.height
            });
          }
        
        if (cancelled) {
          try {
            pdf.destroy();
          } catch (error) {
            logger.warn('Failed to destroy PDF document after cancellation', { error });
          }
          return;
        }
        
        setPdfDoc(pdf);
        // why: pdfDocRefにも最新の値を設定して、クリーンアップ関数で参照できるようにする
        // alt: useStateのみを使用（クリーンアップ関数が古い値を参照する可能性がある）
        // evidence: useRefにより、クリーンアップ関数内で最新のpdfDocを参照できる
        pdfDocRef.current = pdf;

        // why: 最初のページを優先的にレンダリングしてUIの応答性を向上
        // alt: すべてのページを読み込んでからレンダリング（表示が遅れる）
        // evidence: 最初のページを優先的にレンダリングすることで、ユーザーがすぐにPDFを確認できる
        if (pdfStructure.pages.length > 0) {
          if (cancelled) {
            try {
              pdf.destroy();
            } catch (error) {
              logger.warn('Failed to destroy PDF document after cancellation', { error });
            }
            return;
          }
          if (uiState.viewMode === 'continuous') {
            await renderAllPages(pdf);
            if (cancelled) return;
          } else {
            const firstPage = pdfStructure.pages[0];
            const firstSourcePageNumber = getSourcePageNumber(firstPage);
            if (firstSourcePageNumber) {
              await renderPageNumber(pdf, firstSourcePageNumber);
              if (cancelled) return;
            } else if (canvasRef.current) {
              renderBlankPage(canvasRef.current, firstPage.width, firstPage.height, zoomLevel);
            }
          }
          if (!cancelled) {
            setCurrentPageInStore(1);
          }
        }
      } catch (error) {
        if (cancelled) return;
        logger.error('Error loading PDF', error instanceof Error ? error : new Error(String(error)));
        setError(error instanceof Error ? error.message : 'PDFの読み込みに失敗しました');
        // why: エラー時もpdfDocをnullに設定して状態をクリア
        // alt: pdfDocをnullに設定しない（古いPDFオブジェクトが残る可能性がある）
        // evidence: エラー時もpdfDocをnullに設定することで、状態をクリアできる
        setPdfDoc(null);
        pdfDocRef.current = null;
      }
    };

    renderPage();
    } else {
      // why: PDFファイルパスが変更されていない場合は、pdfStructureの更新のみを行う（オーバーレイが自動的に再描画される）
      // alt: PDFを再読み込み（パフォーマンスが悪い）
      // evidence: ファイルパスが変更されていない場合は、PDFファイル自体は変更されていないため、オーバーレイの再描画で十分
      // 注意: pdfStructureが更新されると、currentPageStructureが自動的に再計算され、オーバーレイが再描画される
    }
    
    // why: 現在のpdfStructureを保持して、次回の比較に使用する
    // alt: 保持しない（次回の比較ができない）
    // evidence: 現在のpdfStructureを保持することで、次回の変更検出が可能になる
    prevPdfStructureRef.current = pdfStructure;

    // why: コンポーネントのアンマウント時またはPDF構造変更時にPDF.jsオブジェクトをクリーンアップしてメモリリークを防ぐ
    // alt: クリーンアップしない（メモリリークが発生する可能性がある）
    // evidence: pdf.destroy()により、PDF.jsのリソースが解放される
    return () => {
      // why: クリーンアップ関数で非同期処理をキャンセルしてメモリリークを防ぐ
      // alt: キャンセルしない（非同期処理が完了するまで待つ）
      // evidence: キャンセルフラグとloadingTaskのキャンセルにより、メモリリークを防ぐ
      cancelled = true;
      if (loadingTask) {
        try {
          // why: loadingTaskをキャンセルしてリソースを解放する
          // alt: キャンセルしない（リソースが解放されない可能性がある）
          // evidence: loadingTask.cancel()により、PDF.jsの読み込み処理がキャンセルされる
          loadingTask.cancel();
        } catch (error) {
          // 既にキャンセルされている場合はエラーを無視
          logger.warn('Failed to cancel loading task', { error });
        }
      }
      // why: pdfDocRefを使用して最新のpdfDocを参照する（クロージャの問題を回避）
      // alt: useStateのpdfDocを参照（古い値を参照する可能性がある）
      // evidence: useRefにより、クリーンアップ関数内で最新のpdfDocを参照できる
      const currentPdfDoc = pdfDocRef.current;
      if (currentPdfDoc) {
        try {
          currentPdfDoc.destroy();
        } catch (error) {
          // 既に破棄されている場合はエラーを無視
          logger.warn('Failed to destroy PDF document on unmount', { error });
        }
        pdfDocRef.current = null;
      }
    };
  }, [pdfStructure, renderPageNumber, renderAllPages, uiState.viewMode, setCurrentPageInStore, setError]); // why: renderCacheはuseMemoでメモ化されているため、依存配列に含める必要がない。pdfDocRefとprevPdfStructureRefはuseRefで管理されているため、依存配列に含める必要がない

  // why: ズーム変更をデバウンスしてレンダリング負荷を削減
  // alt: 即座にレンダリング（頻繁な再レンダリングが発生）
  // evidence: デバウンスにより不要なレンダリングが削減され、パフォーマンスが向上
  const debouncedZoomLevel = useDebounce(zoomLevel, 100);
  
  // why: ページ変更とズーム変更をデバウンスしてレンダリング負荷を削減
  // alt: 即座にレンダリング（頻繁な再レンダリングが発生）
  // evidence: デバウンスにより不要なレンダリングが削減され、パフォーマンスが向上
  useEffect(() => {
    if (!pdfDoc) return;

    // why: 前のレンダリングタスクをキャンセルしてから新しいレンダリングを開始
    // alt: 前のタスクをキャンセルしない（PDF.jsがエラーを発生）
    // evidence: 前のタスクをキャンセルすることで、複数のレンダリング操作の競合を防ぐ
    if (renderTaskRef.current) {
      try {
        renderTaskRef.current.cancel();
      } catch (error) {
        // why: キャンセルエラーは無視（既に完了している可能性がある）
        // alt: エラーをログに記録（不要なログが増える）
        // evidence: キャンセルエラーは通常の動作であり、ログに記録する必要はない
        if (error instanceof Error && error.name !== 'RenderingCancelledException') {
          logger.warn('Failed to cancel render task', { error: error.message });
        }
      }
      renderTaskRef.current = null;
    }

    // レンダリングを次のフレームに遅延（requestAnimationFrame）
    // why: rafIdをuseRefで管理して、クリーンアップ関数で最新の値を参照できるようにする
    // alt: ローカル変数として管理（クリーンアップ関数が古い値を参照する可能性がある）
    // evidence: useRefにより、クリーンアップ関数内で最新のrafIdを参照できる
    if (zoomRafIdRef.current !== null) {
      cancelAnimationFrame(zoomRafIdRef.current);
    }
    zoomRafIdRef.current = requestAnimationFrame(() => {
      zoomRafIdRef.current = null;
      if (uiState.viewMode === 'continuous') {
        // 複数ページ表示モードでは、各ページのcanvas要素のrefコールバックでレンダリングされる
        // ズーム変更時は、すべてのページを再レンダリング
        if (pdfDoc) {
          renderAllPages(pdfDoc);
        }
      } else {
        if (!canvasRef.current) return;
        const sourcePageNumber = getSourcePageForDisplay(currentPage);
        if (sourcePageNumber) {
          renderPageNumber(pdfDoc, sourcePageNumber);
        } else if (currentPageStructure) {
          renderBlankPage(canvasRef.current, currentPageStructure.width, currentPageStructure.height, zoomLevel);
        }
      }
    });

    return () => {
      if (zoomRafIdRef.current !== null) {
        cancelAnimationFrame(zoomRafIdRef.current);
        zoomRafIdRef.current = null;
      }
      // why: クリーンアップ時にレンダリングタスクをキャンセル
      // alt: タスクをキャンセルしない（メモリリークの可能性）
      // evidence: クリーンアップ時にタスクをキャンセルすることで、不要なレンダリングを防ぐ
      if (renderTaskRef.current) {
        try {
          renderTaskRef.current.cancel();
        } catch (error) {
          // why: キャンセルエラーは無視（既に完了している可能性がある）
          // alt: エラーをログに記録（不要なログが増える）
          // evidence: キャンセルエラーは通常の動作であり、ログに記録する必要はない
          if (error instanceof Error && error.name !== 'RenderingCancelledException') {
            logger.warn('Failed to cancel render task', { error: error.message });
          }
        }
        renderTaskRef.current = null;
      }
    };
  }, [currentPage, debouncedZoomLevel, pdfDoc, renderPageNumber, renderAllPages, uiState.viewMode, currentPageStructure, pageOrderKey]);

  // ストアのcurrentPageが変更されたときに、PDFViewerをスクロール
  useEffect(() => {
    if (storeCurrentPage === currentPage || isScrollingRef.current) return;
    
    if (uiState.viewMode === 'continuous') {
      // 複数ページ表示モードでは、指定されたページまでスクロール
      if (!containerRef.current || !pdfStructure) return;

      isScrollingRef.current = true;
      const pageIndex = pdfStructure.pages.findIndex(p => p.pageNumber === storeCurrentPage);
      if (pageIndex === -1) {
        isScrollingRef.current = false;
        return;
      }

      // 各ページの位置を計算
      let cumulativeHeight = 0;
      const pageGap = 20;

      for (let i = 0; i < pageIndex; i++) {
        const page = pdfStructure.pages[i];
        // why: page.heightやzoomLevelが0または無効な値の場合をチェックしてNaNを防ぐ
        // alt: チェックなし（page.heightやzoomLevelが0の場合にNaNが発生する）
        // evidence: ゼロ除算によりNaNが発生し、スクロール位置が正しく計算されない
        if (page.height <= 0 || !Number.isFinite(page.height) || zoomLevel <= 0 || !Number.isFinite(zoomLevel)) {
          logger.warn('Invalid page height or zoom level', { pageHeight: page.height, zoomLevel });
          continue;
        }
        const pageHeight = Math.floor(page.height * zoomLevel);
        cumulativeHeight += pageHeight + pageGap;
      }

      const container = containerRef.current;
      container.scrollTo({ top: cumulativeHeight, behavior: 'smooth' });
      setCurrentPage(storeCurrentPage);

      setTimeout(() => {
        isScrollingRef.current = false;
      }, 500);
    } else {
      // 1ページ表示モードでは、ページを変更
      setCurrentPage(storeCurrentPage);
      if (pdfDoc) {
        const sourcePageNumber = getSourcePageForDisplay(storeCurrentPage);
        if (sourcePageNumber) {
          renderPageNumber(pdfDoc, sourcePageNumber);
        } else if (canvasRef.current && currentPageStructure) {
          renderBlankPage(canvasRef.current, currentPageStructure.width, currentPageStructure.height, zoomLevel);
        }
      }
    }
  }, [storeCurrentPage, currentPage, uiState.viewMode, pdfStructure, zoomLevel, pdfDoc, renderPageNumber, getSourcePageForDisplay, currentPageStructure]);

  // 複数ページ表示モードでスクロール位置に基づいて現在のページを更新
  useEffect(() => {
    if (uiState.viewMode !== 'continuous' || !containerRef.current || !pdfStructure || isScrollingRef.current) return;

    const handleScroll = () => {
      if (isScrollingRef.current) return;

      const container = containerRef.current;
      if (!container) return;

      const scrollTop = container.scrollTop;
      const containerHeight = container.clientHeight;
      const viewportCenter = scrollTop + containerHeight / 2;

      // 各ページの位置を計算
      let cumulativeHeight = 0;
      const pageGap = 20;
      let foundPage = 1;

      for (const page of pdfStructure.pages) {
        // why: page.heightやzoomLevelが0または無効な値の場合をチェックしてNaNを防ぐ
        // alt: チェックなし（page.heightやzoomLevelが0の場合にNaNが発生する）
        // evidence: ゼロ除算によりNaNが発生し、現在のページが正しく検出されない
        if (page.height <= 0 || !Number.isFinite(page.height) || zoomLevel <= 0 || !Number.isFinite(zoomLevel)) {
          logger.warn('Invalid page height or zoom level', { pageHeight: page.height, zoomLevel });
          continue;
        }
        const pageHeight = Math.floor(page.height * zoomLevel);
        const pageTop = cumulativeHeight;
        const pageBottom = cumulativeHeight + pageHeight;

        if (viewportCenter >= pageTop && viewportCenter <= pageBottom) {
          foundPage = page.pageNumber;
          break;
        }

        cumulativeHeight += pageHeight + pageGap;
      }

      if (foundPage !== currentPage) {
        setCurrentPage(foundPage);
        setCurrentPageInStore(foundPage);
      }
    };

    const container = containerRef.current;
    container.addEventListener('scroll', handleScroll, { passive: true });

    return () => {
      container.removeEventListener('scroll', handleScroll);
    };
  }, [uiState.viewMode, pdfStructure, zoomLevel, currentPage, setCurrentPageInStore]);

  // why: pdfStructure変更時にcurrentPageが有効な範囲内にあることを確認
  // alt: currentPageを調整しない（削除されたページを表示しようとしてエラーが発生）
  // evidence: ページ削除後、現在表示中のページが削除された場合、有効なページ番号に調整する必要がある
  useEffect(() => {
    if (!pdfStructure) return;
    
    const maxPage = pdfStructure.pages.length;
    if (maxPage === 0) return;
    
    if (currentPage > maxPage) {
      // 削除されたページが現在表示中のページの場合、最後のページに調整
      setCurrentPage(maxPage);
      onPageChange?.(maxPage);
    } else if (currentPage < 1) {
      // ページ番号が1未満の場合、1ページ目に調整
      setCurrentPage(1);
      onPageChange?.(1);
    }
  }, [pdfStructure, currentPage, onPageChange]);

  // オーバーレイのスケール計算をメモ化
  // why: canvasRefの値を直接参照すると再レンダリングが頻繁に発生するため、useStateで管理
  const [canvasSize, setCanvasSize] = useState({ width: 0, height: 0 });
  
  useEffect(() => {
    if (canvasRef.current) {
      // why: ResizeObserverのコールバックをデバウンスして頻繁な更新を抑制
      // alt: 即座に更新（頻繁な再レンダリングが発生）
      // evidence: デバウンスにより不要な再レンダリングが削減され、パフォーマンスが向上
      // why: rafIdをuseRefで管理して、クリーンアップ関数で最新の値を参照できるようにする
      // alt: ローカル変数として管理（クリーンアップ関数が古い値を参照する可能性がある）
      // evidence: useRefにより、クリーンアップ関数内で最新のrafIdを参照できる
      const updateSize = () => {
        if (resizeRafIdRef.current !== null) {
          cancelAnimationFrame(resizeRafIdRef.current);
        }
        resizeRafIdRef.current = requestAnimationFrame(() => {
          resizeRafIdRef.current = null;
          if (canvasRef.current) {
            // why: 表示サイズ（CSS）を使用してオーバーレイのスケールを計算
            // alt: 内部解像度（高DPIで不正確）
            // evidence: 表示サイズを使用することでオーバーレイが正確に配置される
            const rect = canvasRef.current.getBoundingClientRect();
            setCanvasSize({
              width: rect.width,
              height: rect.height,
            });
          }
        });
      };
      updateSize();
      // ResizeObserverでキャンバスサイズ変更を監視（パフォーマンス向上）
      const resizeObserver = new ResizeObserver(updateSize);
      resizeObserver.observe(canvasRef.current);
      return () => {
        if (resizeRafIdRef.current !== null) {
          cancelAnimationFrame(resizeRafIdRef.current);
          resizeRafIdRef.current = null;
        }
        resizeObserver.disconnect();
      };
    }
  }, [pdfDoc]);

  const overlayScale = useMemo(() => {
    if (!currentPageStructure || canvasSize.width === 0 || canvasSize.height === 0 || !currentPageStructure.width || !currentPageStructure.height) {
      return { scaleX: 1, scaleY: 1 };
    }
    // why: currentPageStructure.width/heightが0または無効な値の場合をチェックしてNaNを防ぐ
    // alt: チェックなし（currentPageStructure.width/heightが0の場合にInfinityが発生する）
    // evidence: ゼロ除算によりInfinityが発生し、オーバーレイの位置が正しく計算されない
    if (currentPageStructure.width <= 0 || !Number.isFinite(currentPageStructure.width) ||
        currentPageStructure.height <= 0 || !Number.isFinite(currentPageStructure.height)) {
      logger.warn('Invalid page dimensions', {
        width: currentPageStructure.width,
        height: currentPageStructure.height
      });
      return { scaleX: 1, scaleY: 1 };
    }
    return {
      scaleX: canvasSize.width / currentPageStructure.width,
      scaleY: canvasSize.height / currentPageStructure.height,
    };
  }, [currentPageStructure, canvasSize.width, canvasSize.height]);

  // テキストブロックオーバーレイをメモ化（コンポーネント用のデータのみ）
  const textBlockOverlays = useMemo(() => {
    if (!currentPageStructure?.textBlocks) return [];
    return currentPageStructure.textBlocks.map((textBlock) => ({
      id: textBlock.id,
      textBlock,
      isSelected: selectedTextBlockId === textBlock.id,
    }));
  }, [currentPageStructure, selectedTextBlockId]);

  // 画像オーバーレイをメモ化（コンポーネント用のデータのみ）
  const imageOverlays = useMemo(() => {
    if (!currentPageStructure?.images) return [];
    return currentPageStructure.images.map((image) => ({
      id: image.id,
      image,
      isSelected: selectedImageId === image.id,
    }));
  }, [currentPageStructure, selectedImageId]);

  // ハンドラーをメモ化
  const handleTextBlockClick = useCallback((textBlock: TextBlock) => {
    selectTextBlock(textBlock.id);
  }, [selectTextBlock]);

  // why: ダブルクリックでインライン編集を開始（Wordライクな操作）
  // alt: シングルクリックで編集（誤操作の可能性）
  // evidence: ダブルクリックにより意図的な編集開始を明確にする
  const handleTextBlockDoubleClick = useCallback((textBlock: TextBlock) => {
    selectTextBlock(textBlock.id);
    dispatchUI({ type: 'SET_INLINE_EDITING_TEXT_BLOCK_ID', payload: textBlock.id });
  }, [selectTextBlock]);


  const handlePageBreakAdjust = useCallback(async (pageNumber: number, newPosition: number) => {
    try {
      await adjustPageBreak(pageNumber, newPosition);
    } catch (error) {
      logger.error('Failed to adjust page break', error instanceof Error ? error : new Error(String(error)));
    }
  }, [adjustPageBreak]);

  // why: ページボタンとストアの currentPage を同期して表示を統一
  // alt: 独立した処理（バグが発生しやすい）
  // evidence: 同じロジックを共有することで、連続表示モードでも一貫した動作を保証
  const handlePrevPage = useCallback(() => {
    if (!pdfStructure) return;
    const pageNumbers = pdfStructure.pages.map(p => p.pageNumber).filter((p): p is number => p != null).sort((a, b) => a - b);
    if (pageNumbers.length === 0) return;
    
    const currentIndex = pageNumbers.indexOf(storeCurrentPage);
    if (currentIndex <= 0) return;
    
    const newPage = pageNumbers[currentIndex - 1];
    // why: setCurrentPageInStoreのみを呼び、既存のuseEffectでスクロール処理を行う
    // alt: setCurrentPageとsetCurrentPageInStoreの両方を呼ぶ（処理が重複する）
    // evidence: setCurrentPageInStoreのみを呼ぶことで、連続表示と1ページ表示の両方で同じロジックが動作する
    setCurrentPageInStore(newPage);
    onPageChange?.(newPage);
  }, [pdfStructure, storeCurrentPage, onPageChange, setCurrentPageInStore]);

  const handleNextPage = useCallback(() => {
    if (!pdfStructure) return;
    const pageNumbers = pdfStructure.pages.map(p => p.pageNumber).filter((p): p is number => p != null).sort((a, b) => a - b);
    if (pageNumbers.length === 0) return;
    
    const currentIndex = pageNumbers.indexOf(storeCurrentPage);
    if (currentIndex < 0 || currentIndex >= pageNumbers.length - 1) return;
    
    const newPage = pageNumbers[currentIndex + 1];
    // why: setCurrentPageInStoreのみを呼び、既存のuseEffectでスクロール処理を行う
    // alt: setCurrentPageとsetCurrentPageInStoreの両方を呼ぶ（処理が重複する）
    // evidence: setCurrentPageInStoreのみを呼ぶことで、連続表示と1ページ表示の両方で同じロジックが動作する
    setCurrentPageInStore(newPage);
    onPageChange?.(newPage);
  }, [pdfStructure, storeCurrentPage, onPageChange, setCurrentPageInStore, totalPages]);

  // why: スクロール機能を提供してユーザーがPDFを快適に閲覧できるようにする
  // alt: スクロールなし（大きなPDFで閲覧が困難）
  // evidence: スクロール機能によりユーザー体験が向上
  const scrollUp = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollBy({ top: -100, behavior: 'smooth' });
    }
  }, []);

  const scrollDown = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollBy({ top: 100, behavior: 'smooth' });
    }
  }, []);

  const scrollLeft = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollBy({ left: -100, behavior: 'smooth' });
    }
  }, []);

  const scrollRight = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollBy({ left: 100, behavior: 'smooth' });
    }
  }, []);

  const scrollToTop = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollTo({ top: 0, behavior: 'smooth' });
    }
  }, []);

  const scrollToBottom = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollTo({ top: containerRef.current.scrollHeight, behavior: 'smooth' });
    }
  }, []);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDraggingRef.current || !dragStartRef.current || !containerRef.current) return;

    e.preventDefault();
    const deltaX = dragStartRef.current.x - e.clientX;
    const deltaY = dragStartRef.current.y - e.clientY;

    containerRef.current.scrollLeft = dragStartRef.current.scrollLeft + deltaX;
    containerRef.current.scrollTop = dragStartRef.current.scrollTop + deltaY;
  }, []);

  const handleMouseUp = useCallback(() => {
    if (isDraggingRef.current && containerRef.current) {
      containerRef.current.style.cursor = 'grab';
    }
    isDraggingRef.current = false;
    dragStartRef.current = null;
  }, []);

  // why: マウスイベントリスナーを登録してドラッグによるパン機能を実現
  // alt: イベントリスナーなし（ドラッグで移動できない）
  // evidence: イベントリスナーによりドラッグによる移動が可能になる
  useEffect(() => {
    if (!containerRef.current) return;

    const container = containerRef.current;
    
    // マウス移動とマウスアップはグローバルに登録（ドラッグ中にマウスがコンテナ外に出ても動作するように）
    globalThis.window.addEventListener('mousemove', handleMouseMove);
    globalThis.window.addEventListener('mouseup', handleMouseUp);
    
    // 中ボタンと右ボタンクリックのデフォルト動作を無効化（コンテキストメニューを防ぐ）
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
    };
    
    // why: mousedownイベントを直接登録してドラッグによるパン機能を実現
    // alt: ReactのonMouseDownのみ（イベントが正しく処理されない可能性がある）
    // evidence: 直接登録することで、確実にドラッグイベントが処理される
      const handleContainerMouseDown = (e: MouseEvent) => {
      // why: e.targetがHTMLElementでない場合をチェックして型安全性を向上
      // alt: 型アサーションのみ（実行時エラーが発生する可能性がある）
      // evidence: 型チェックにより、実行時エラーを防ぐ
      if (!(e.target instanceof HTMLElement)) {
        return;
      }
      const target = e.target;
      
      // テキスト選択やボタンクリックの場合は無視
      if (target.tagName === 'BUTTON' || 
          target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          globalThis.window.getSelection()?.toString()) {
        return;
      }

      // why: レイアウト編集時のオーバーレイ要素のクリックを除外（previewOnly ではオーバーレイ非表示）
      // alt: すべてのクリックをドラッグとして処理（編集機能が動作しない）
      // evidence: オーバーレイ要素のクリックを除外することで、編集モードで編集が優先される
      if (target.closest('.image-overlay') || 
          target.closest('.text-block-overlay') ||
          target.closest('.image-resizer') ||
          target.closest('.resize-handle')) {
        return;
      }

      // why: 旧ページ一覧 UI のドラッグハンドル等を除外（互換のためセレクタは残す）
      // alt: すべてのクリックをパンとして処理
      // evidence: 一覧 UI 要素を除外すると意図しないパンを防げる
      if (target.closest('.page-item') || 
          target.closest('.drag-handle')) {
        return;
      }

      // 左ボタン、中ボタン、右ボタンすべてでパン可能
      if (e.button === 0 || e.button === 1 || e.button === 2) {
        e.preventDefault();
        if (containerRef.current) {
          isDraggingRef.current = true;
          dragStartRef.current = {
            x: e.clientX,
            y: e.clientY,
            scrollLeft: containerRef.current.scrollLeft,
            scrollTop: containerRef.current.scrollTop,
          };
          containerRef.current.style.cursor = 'grabbing';
        }
      }
      
      // 中ボタン（ホイールクリック）のデフォルト動作を無効化
      if (e.button === 1) {
        e.preventDefault();
      }
    };
    
    container.addEventListener('contextmenu', handleContextMenu);
    container.addEventListener('mousedown', handleContainerMouseDown);

    return () => {
      globalThis.window.removeEventListener('mousemove', handleMouseMove);
      globalThis.window.removeEventListener('mouseup', handleMouseUp);
      container.removeEventListener('contextmenu', handleContextMenu);
      container.removeEventListener('mousedown', handleContainerMouseDown);
    };
  }, [handleMouseMove, handleMouseUp]);

  // why: キーボードショートカットでスクロール操作を提供
  // alt: マウススクロールのみ（キーボードユーザーが不便）
  // evidence: キーボードショートカットによりアクセシビリティが向上
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // テキスト入力中はスクロールショートカットを無効化
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      // Ctrl/Cmdキーが押されている場合は他のショートカットを優先
      if (e.ctrlKey || e.metaKey) {
        return;
      }

      switch (e.key) {
        case 'F2':
          if (previewOnly) {
            break;
          }
          // why: F2キーで選択中のテキストブロックを編集開始（Wordライクな操作）
          // alt: ダブルクリックのみ（キーボードユーザーが不便）
          // evidence: F2キーによりキーボードのみで編集を開始できる
          if (selectedTextBlockId && currentPageStructure) {
            const textBlock = currentPageStructure.textBlocks.find(block => block.id === selectedTextBlockId);
            if (textBlock) {
              e.preventDefault();
              dispatchUI({ type: 'SET_INLINE_EDITING_TEXT_BLOCK_ID', payload: textBlock.id });
            }
          }
          break;
        case 'ArrowUp':
          e.preventDefault();
          scrollUp();
          break;
        case 'ArrowDown':
          e.preventDefault();
          scrollDown();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          scrollLeft();
          break;
        case 'ArrowRight':
          e.preventDefault();
          scrollRight();
          break;
        case 'PageUp':
          e.preventDefault();
          if (containerRef.current) {
            containerRef.current.scrollBy({ top: -containerRef.current.clientHeight * 0.9, behavior: 'smooth' });
          }
          break;
        case 'PageDown':
          e.preventDefault();
          if (containerRef.current) {
            containerRef.current.scrollBy({ top: containerRef.current.clientHeight * 0.9, behavior: 'smooth' });
          }
          break;
        case 'Home':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            scrollToTop();
          }
          break;
        case 'End':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            scrollToBottom();
          }
          break;
      }
    };

    globalThis.window.addEventListener('keydown', handleKeyDown);
    return () => globalThis.window.removeEventListener('keydown', handleKeyDown);
  }, [scrollUp, scrollDown, scrollLeft, scrollRight, scrollToTop, scrollToBottom, selectedTextBlockId, currentPageStructure, dispatchUI, previewOnly]);

  if (!pdfStructure) {
    return <div className="pdf-viewer-empty">Markdownを入力すると印刷プレビューが表示されます</div>;
  }

  return (
    <div className="pdf-viewer">
      <div className="pdf-viewer-controls">
        <button 
          onClick={handlePrevPage} 
          disabled={(() => {
            if (!pdfStructure) return true;
            const pageNumbers = pdfStructure.pages.map(p => p.pageNumber).filter((p): p is number => p != null).sort((a, b) => a - b);
            if (pageNumbers.length === 0) return true;
            const currentIndex = pageNumbers.indexOf(storeCurrentPage);
            return currentIndex <= 0;
          })()}
        >
          前のページ
        </button>
        <span>
          ページ {storeCurrentPage} / {totalPages}
        </span>
        <button 
          onClick={handleNextPage} 
          disabled={(() => {
            if (!pdfStructure) return true;
            const pageNumbers = pdfStructure.pages.map(p => p.pageNumber).filter((p): p is number => p != null).sort((a, b) => a - b);
            if (pageNumbers.length === 0) return true;
            const currentIndex = pageNumbers.indexOf(storeCurrentPage);
            return currentIndex < 0 || currentIndex >= pageNumbers.length - 1;
          })()}
        >
          次のページ
        </button>
        <button
          onClick={() => {
            dispatchUI({ 
              type: 'SET_VIEW_MODE', 
              payload: uiState.viewMode === 'single' ? 'continuous' : 'single' 
            });
          }}
          title={uiState.viewMode === 'single' ? '連続表示モードに切り替え' : '1ページ表示モードに切り替え'}
        >
          {uiState.viewMode === 'single' ? '連続表示' : '1ページ表示'}
        </button>
        {!previewOnly && currentPageStructure && (
          <span className="pdf-structure-info">
            (テキスト: {currentPageStructure.textBlocks.length}個, 
            画像: {currentPageStructure.images.length}個)
          </span>
        )}
      </div>
      <div 
        className="pdf-viewer-canvas-container" 
        ref={containerRef}
        data-view-mode={uiState.viewMode}
      >
        {uiState.viewMode === 'continuous' ? (
          // 複数ページ表示モード
          <>
            {pdfStructure?.pages && pdfStructure.pages.length > 0 && pdfStructure.pages.map((pageStructure, pageIndex) => {
              const pageNum = pageStructure.pageNumber;
              const sourcePageNumber = getSourcePageNumber(pageStructure);
              const pageKey = `page-${pageNum}-${sourcePageNumber ?? 'none'}`;

              // why: pageStructure.width/heightやzoomLevelが0または無効な値の場合をチェックしてNaNを防ぐ
              // alt: チェックなし（pageStructure.width/heightやzoomLevelが0の場合にNaNが発生する）
              // evidence: ゼロ除算によりNaNが発生し、オーバーレイの位置が正しく計算されない
              if (pageStructure.width <= 0 || !Number.isFinite(pageStructure.width) ||
                  pageStructure.height <= 0 || !Number.isFinite(pageStructure.height) ||
                  zoomLevel <= 0 || !Number.isFinite(zoomLevel)) {
                logger.warn('Invalid page dimensions or zoom level', {
                  width: pageStructure.width,
                  height: pageStructure.height,
                  zoomLevel
                });
                return null;
              }
              const displayWidth = Math.floor(pageStructure.width * zoomLevel);
              const displayHeight = Math.floor(pageStructure.height * zoomLevel);
              
              const overlayScale = {
                scaleX: displayWidth / pageStructure.width,
                scaleY: displayHeight / pageStructure.height,
              };

              const pageTextBlockOverlays = (pageStructure.textBlocks || []).map((textBlock) => ({
                id: textBlock.id,
                textBlock,
                isSelected: selectedTextBlockId === textBlock.id,
              }));

              const pageImageOverlays = (pageStructure.images || []).map((image) => ({
                id: image.id,
                image,
                isSelected: selectedImageId === image.id,
              }));

              return (
                <div 
                  key={pageKey} 
                  className="pdf-page-container" 
                  ref={(el) => {
                    if (el) pageContainerRefsMap.current.set(pageNum, el);
                  }}
                  style={{
                    height: `${displayHeight}px`,
                    width: `${displayWidth}px`,
                  } as React.CSSProperties}
                >
                  <canvas 
                    ref={(el) => {
                      if (el) {
                        canvasRefsMap.current.set(pageNum, el);
                        if (pdfDoc) {
                          if (sourcePageNumber) {
                            // レンダリングを実行
                            renderPageToCanvas(pdfDoc, sourcePageNumber, el).catch((error) => {
                              if (error?.name !== 'RenderingCancelledException' && !error?.message?.includes('cancelled')) {
                                logger.error(`Error rendering page ${sourcePageNumber}`, error instanceof Error ? error : new Error(String(error)));
                              }
                            });
                          } else {
                            renderBlankPage(el, pageStructure.width, pageStructure.height, zoomLevel);
                          }
                        }
                      }
                    }}
                    className="pdf-viewer-canvas"
                    data-page-number={pageNum}
                    style={{
                      width: `${displayWidth}px`,
                      height: `${displayHeight}px`,
                    } as React.CSSProperties}
                  />
                  {!previewOnly && (
                  <div 
                    className="pdf-structure-overlay"
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: `${displayWidth}px`,
                      height: `${displayHeight}px`,
                    } as React.CSSProperties}
                  >
                    {/* 各ページのオーバーレイ */}
                    <Suspense fallback={null}>
                      {pageTextBlockOverlays.map((overlay) => (
                        <TextBlockOverlay
                          key={overlay.id}
                          textBlock={overlay.textBlock}
                          scaleX={overlayScale.scaleX}
                          scaleY={overlayScale.scaleY}
                          pageHeight={pageStructure.height}
                          pageWidth={pageStructure.width}
                          isSelected={overlay.isSelected}
                          onClick={handleTextBlockClick}
                          onDoubleClick={handleTextBlockDoubleClick}
                        />
                      ))}
                    </Suspense>
                    <Suspense fallback={null}>
                      {pageImageOverlays.map((overlay) => (
                        <ImageOverlay
                          key={overlay.id}
                          image={overlay.image}
                          scaleX={overlayScale.scaleX}
                          scaleY={overlayScale.scaleY}
                          pageHeight={pageStructure.height}
                          pageWidth={pageStructure.width}
                          isSelected={overlay.isSelected}
                        />
                      ))}
                    </Suspense>
                  </div>
                  )}
                </div>
              );
            })}
          </>
        ) : (
          // 1ページ表示モード（既存の実装）
          <div className="pdf-viewer-canvas-wrapper">
            <canvas ref={canvasRef} className="pdf-viewer-canvas" />
            {/* 抽出された構造をオーバーレイ表示 */}
            {currentPageStructure && pdfDoc && canvasRef.current && !previewOnly && (
            <div 
              className="pdf-structure-overlay"
              style={{
                '--overlay-width': `${canvasRef.current.width}px`,
                '--overlay-height': `${canvasRef.current.height}px`,
              } as React.CSSProperties}
            >
            <Suspense fallback={null}>
              {currentPageStructure && textBlockOverlays.map((overlay) => (
                <TextBlockOverlay
                  key={overlay.id}
                  textBlock={overlay.textBlock}
                  scaleX={overlayScale.scaleX}
                  scaleY={overlayScale.scaleY}
                  pageHeight={currentPageStructure.height}
                  pageWidth={currentPageStructure.width}
                  isSelected={overlay.isSelected}
                  onClick={handleTextBlockClick}
                  onDoubleClick={handleTextBlockDoubleClick}
                />
              ))}
            </Suspense>
            <Suspense fallback={null}>
              {currentPageStructure && imageOverlays.map((overlay) => (
                <ImageOverlay
                  key={overlay.id}
                  image={overlay.image}
                  scaleX={overlayScale.scaleX}
                  scaleY={overlayScale.scaleY}
                  pageHeight={currentPageStructure.height}
                  pageWidth={currentPageStructure.width}
                  isSelected={overlay.isSelected}
                />
              ))}
            </Suspense>
            {uiState.showPageBreakEditor && currentPageStructure && (
              <Suspense fallback={null}>
                <div className="page-break-editor-overlay">
                  <PageBreakEditor
                    page={{
                      ...currentPageStructure,
                      height: canvasRef.current?.height && overlayScale.scaleY > 0 && Number.isFinite(overlayScale.scaleY)
                        ? canvasRef.current.height / overlayScale.scaleY
                        : currentPageStructure.height,
                    }}
                    onAdjust={handlePageBreakAdjust}
                  />
                </div>
              </Suspense>
            )}
            </div>
          )}
          </div>
        )}
        
        {/* インライン編集（Wordライク） */}
        {!previewOnly && uiState.inlineEditingTextBlockId && currentPageStructure && (() => {
          const editingBlock = currentPageStructure.textBlocks.find(
            block => block.id === uiState.inlineEditingTextBlockId
          );
          if (!editingBlock) return null;
          return (
            <Suspense fallback={null}>
              <InlineTextEditor
                textBlock={editingBlock}
                scaleX={overlayScale.scaleX}
                scaleY={overlayScale.scaleY}
                pageHeight={currentPageStructure.height}
                onSave={() => {
                  dispatchUI({ type: 'SET_INLINE_EDITING_TEXT_BLOCK_ID', payload: null });
                  selectTextBlock(null);
                }}
                onCancel={() => {
                  dispatchUI({ type: 'SET_INLINE_EDITING_TEXT_BLOCK_ID', payload: null });
                  selectTextBlock(null);
                }}
              />
            </Suspense>
          );
        })()}
        
        {/* テキスト編集ダイアログ */}
        {!previewOnly && uiState.editingTextBlock && (
          <div className="editor-dialog-overlay" onClick={() => {
            dispatchUI({ type: 'SET_EDITING_TEXT_BLOCK', payload: null });
            selectTextBlock(null);
          }}>
            <div className="editor-dialog" onClick={(e) => e.stopPropagation()}>
              <Suspense fallback={<div>読み込み中...</div>}>
                <TextEditor 
                  textBlock={uiState.editingTextBlock} 
                  onClose={() => {
                    dispatchUI({ type: 'SET_EDITING_TEXT_BLOCK', payload: null });
                    selectTextBlock(null);
                  }} 
                />
              </Suspense>
            </div>
          </div>
        )}
        
        {/* テキスト入力ダイアログ */}
        {!previewOnly && uiState.showTextInput && currentPageStructure && (
          <div className="editor-dialog-overlay" onClick={() => dispatchUI({ type: 'SET_SHOW_TEXT_INPUT', payload: false })}>
            <div className="editor-dialog" onClick={(e) => e.stopPropagation()}>
              <Suspense fallback={<div>読み込み中...</div>}>
                <TextInput
                  pageNumber={currentPageStructure.pageNumber}
                  x={uiState.textInputPosition.x}
                  y={uiState.textInputPosition.y}
                  width={uiState.textInputPosition.width}
                  height={uiState.textInputPosition.height}
                  onSave={() => dispatchUI({ type: 'SET_SHOW_TEXT_INPUT', payload: false })}
                  onCancel={() => dispatchUI({ type: 'SET_SHOW_TEXT_INPUT', payload: false })}
                />
              </Suspense>
            </div>
          </div>
        )}
        
        {/* 画像挿入ダイアログ */}
        {!previewOnly && uiState.showImageInserter && currentPageStructure && (
          <div className="editor-dialog-overlay" onClick={() => dispatchUI({ type: 'SET_SHOW_IMAGE_INSERTER', payload: false })}>
            <div className="editor-dialog" onClick={(e) => e.stopPropagation()}>
              <Suspense fallback={<div>読み込み中...</div>}>
                <ImageInserter
                  pageNumber={currentPageStructure.pageNumber}
                  x={uiState.imageInserterPosition.x}
                  y={uiState.imageInserterPosition.y}
                  onCancel={() => dispatchUI({ type: 'SET_SHOW_IMAGE_INSERTER', payload: false })}
                />
              </Suspense>
            </div>
          </div>
        )}
      </div>
      
      {/* ツールバー */}
      <div className="pdf-viewer-toolbar">
        {!previewOnly && (
          <>
        <button 
          onClick={() => {
            if (canvasRef.current && currentPageStructure) {
              dispatchUI({
                type: 'SET_TEXT_INPUT_POSITION',
                payload: {
                  x: 72,
                  y: currentPageStructure.height - 200,
                  width: 400,
                  height: 100,
                },
              });
              dispatchUI({ type: 'SET_SHOW_TEXT_INPUT', payload: true });
            }
          }}
          title="新しいテキストブロックを追加します"
        >
          新しいテキストブロックを追加
        </button>
        <button 
          onClick={() => {
            if (canvasRef.current && currentPageStructure) {
              dispatchUI({
                type: 'SET_IMAGE_INSERTER_POSITION',
                payload: {
                  x: 100,
                  y: currentPageStructure.height - 300,
                },
              });
              dispatchUI({ type: 'SET_SHOW_IMAGE_INSERTER', payload: true });
            }
          }}
          title="新しい画像を挿入します"
        >
          新しい画像を挿入
        </button>
        <button 
          onClick={() => {
            dispatchUI({ type: 'SET_SHOW_PAGE_BREAK_EDITOR', payload: !uiState.showPageBreakEditor });
          }}
          data-page-break-active={uiState.showPageBreakEditor}
          title={uiState.showPageBreakEditor ? '改ページ編集を無効化' : '改ページ編集を有効化'}
        >
          {uiState.showPageBreakEditor ? '改ページ編集を閉じる' : '改ページを編集'}
        </button>
          </>
        )}
        <div className="scroll-controls">
          <button onClick={scrollUp} title="上にスクロール (↑)">
            <span className="scroll-button-icon">↑</span>
            <span className="scroll-button-label">上</span>
          </button>
          <button onClick={scrollDown} title="下にスクロール (↓)">
            <span className="scroll-button-icon">↓</span>
            <span className="scroll-button-label">下</span>
          </button>
          <button onClick={scrollLeft} title="左にスクロール (←)">
            <span className="scroll-button-icon">←</span>
            <span className="scroll-button-label">左</span>
          </button>
          <button onClick={scrollRight} title="右にスクロール (→)">
            <span className="scroll-button-icon">→</span>
            <span className="scroll-button-label">右</span>
          </button>
        </div>
      </div>
    </div>
  );
}, (prevProps, nextProps) => {
  // カスタム比較関数：変更がない場合は再レンダリングをスキップ
  // why: pdfStructureの参照が変更された場合は常に再レンダリング（オーバーレイの位置を更新するため）
  // alt: 参照のみを比較（深い変更が検出されない）
  // evidence: pdfStructureの参照が変更された場合、編集操作（画像・テキストブロックの移動など）が反映されたことを意味する
  // 注意: pdfStructureの参照が同じでも、内部の変更（画像・テキストブロックの位置変更）は検出できない
  // しかし、編集操作では常に新しいpdfStructureオブジェクトが作成されるため、参照は変更される
  // ファイルパスが変更された場合も再レンダリングが必要（PDFの再読み込みが必要）
  if (prevProps.pdfStructure !== nextProps.pdfStructure) {
    return false; // 再レンダリングが必要
  }
  // pdfStructureが同じでも、zoomLevelやonPageChangeが変更された場合は再レンダリング
  return (
    prevProps.zoomLevel === nextProps.zoomLevel &&
    prevProps.onPageChange === nextProps.onPageChange &&
    prevProps.previewOnly === nextProps.previewOnly
  );
});

PDFViewer.displayName = 'PDFViewer';
