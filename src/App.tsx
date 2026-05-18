import { invoke } from '@tauri-apps/api/core';
import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ErrorDisplay } from './components/ErrorDisplay';
import { PDFViewer } from './components/PDFViewer';
import { ToastContainer } from './components/Toast';
import { KeyboardShortcutsHelp } from './components/KeyboardShortcutsHelp';
import { useToast } from './hooks/useToast';
import { useDebounce } from './hooks/useDebounce';
import { usePdfStore } from './stores/pdfStore';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { logger } from './utils/logger';
import { isMarkdownFilePath } from './utils/markdownFile';
import { isTauriRuntimeAvailable } from './utils/tauriRuntime';
import { estimatePreviewPageFromMarkdown, getLineNumberFromOffset } from './utils/markdownPositionSync';
import {
  insertMarkdownHardBreak,
  restoreTextareaCursor,
  resolveEditorCursorOffset,
} from './utils/markdownEditor';
import './App.css';

function App() {
  const isTauriRuntime = useMemo(() => isTauriRuntimeAvailable(), []);
  // why: Zustandストアから個別にセレクターを使用して、不要な再レンダリングを防ぐ
  // alt: オブジェクト分割代入を使用（すべての状態変更で再レンダリングが発生）
  // evidence: 個別のセレクターにより、必要な状態のみを購読し、不要な再レンダリングを防ぐ
  const pdfStructure = usePdfStore((state) => state.pdfStructure);
  const zoomLevel = usePdfStore((state) => state.zoomLevel);
  const error = usePdfStore((state) => state.error);
  const loadPdf = usePdfStore((state) => state.loadPdf);
  const savePdf = usePdfStore((state) => state.savePdf);
  const markdownText = usePdfStore((state) => state.markdownText);
  const isPreviewBuilding = usePdfStore((state) => state.isPreviewBuilding);
  const previewError = usePdfStore((state) => state.previewError);
  const currentPage = usePdfStore((state) => state.currentPage);
  const setMarkdownText = usePdfStore((state) => state.setMarkdownText);
  const requestMarkdownPreview = usePdfStore((state) => state.requestMarkdownPreview);
  const setCurrentPage = usePdfStore((state) => state.setCurrentPage);
  const setZoomLevel = usePdfStore((state) => state.setZoomLevel);
  const clearError = usePdfStore((state) => state.clearError);

  const { toasts, dismissToast, success, error: showError } = useToast();
  // why: pdfStoreのisLoadingを使用して、状態の重複を避ける
  // alt: ローカルのisLoading stateを使用（状態の重複と不整合の可能性）
  // evidence: pdfStoreのisLoadingを使用することで、状態の一貫性を保つ
  const isLoading = usePdfStore((state) => state.isLoading);
  const debouncedMarkdown = useDebounce(markdownText, 450);
  const [editorCursorLine, setEditorCursorLine] = useState(1);
  const previewRequestIdRef = useRef(0);
  const markdownEditorRef = useRef<HTMLTextAreaElement | null>(null);
  const pendingCursorOffsetRef = useRef<number | null>(null);
  const currentPageRef = useRef(currentPage);
  const debouncedCursorLine = useDebounce(editorCursorLine, 120);

  const syncEditorCursor = useCallback((text: string, cursorOffset: number) => {
    const lineNumber = getLineNumberFromOffset(text, cursorOffset);
    setEditorCursorLine(lineNumber);
  }, []);

  const handleOpenMarkdown = useCallback(async () => {
    if (!isTauriRuntime) {
      showError('ブラウザモードではファイルダイアログは利用できません。tauri:dev で確認してください。');
      return;
    }
    try {
      const filePath = await invoke<string | null>('open_file_dialog');
      if (filePath) {
        if (!isMarkdownFilePath(filePath)) {
          showError('Markdownファイル（.md / .markdown / .mdown）のみ開けます。');
          return;
        }
        try {
          const bytes = await readFile(filePath);
          const text = new TextDecoder().decode(bytes);
          setMarkdownText(text);
          success('Markdownファイルを読み込みました');
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          showError(`Markdownの読み込みに失敗しました: ${errorMessage}`);
          logger.error('Error opening markdown file', error instanceof Error ? error : new Error(String(error)));
        }
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError(`ファイルダイアログの表示に失敗しました: ${errorMessage}`);
      logger.error('Error opening markdown file', error instanceof Error ? error : new Error(String(error)));
    }
  }, [isTauriRuntime, setMarkdownText, success, showError]);

  const handleSaveMarkdown = useCallback(async () => {
    if (!isTauriRuntime) {
      showError('ブラウザモードではファイル保存は利用できません。tauri:dev で確認してください。');
      return;
    }
    if (!markdownText.trim()) return;
    try {
      const filePath = await invoke<string | null>('save_file_dialog', { target: 'markdown' });
      if (!filePath) return;
      const bytes = new TextEncoder().encode(markdownText);
      await writeFile(filePath, bytes);
      success('Markdownファイルを保存しました');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError(`Markdownの保存に失敗しました: ${errorMessage}`);
      logger.error('Error saving markdown file', error instanceof Error ? error : new Error(String(error)));
    }
  }, [isTauriRuntime, markdownText, success, showError]);

  const handleSavePdf = useCallback(async () => {
    if (!isTauriRuntime) {
      showError('ブラウザモードではPDF保存は利用できません。tauri:dev で確認してください。');
      return;
    }
    if (!pdfStructure) return;

    try {
      const filePath = await invoke<string | null>('save_file_dialog', { target: 'pdf' });
      if (!filePath) return;
      await savePdf(filePath);
      success('PDFファイルを保存しました');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError(`ファイルの保存に失敗しました: ${errorMessage}`);
      logger.error('Error saving PDF', error instanceof Error ? error : new Error(String(error)));
    }
  }, [isTauriRuntime, pdfStructure, savePdf, success, showError]);

  const handleZoomIn = useCallback(() => {
    setZoomLevel(zoomLevel + 0.1);
  }, [zoomLevel, setZoomLevel]);

  const handleZoomOut = useCallback(() => {
    setZoomLevel(zoomLevel - 0.1);
  }, [zoomLevel, setZoomLevel]);

  const handleZoomReset = useCallback(() => {
    setZoomLevel(1);
  }, [setZoomLevel]);

  useEffect(() => {
    currentPageRef.current = currentPage;
  }, [currentPage]);

  useEffect(() => {
    if (!pdfStructure?.pages?.length) return;
    const targetPage = estimatePreviewPageFromMarkdown(
      markdownText,
      debouncedCursorLine,
      pdfStructure.pages.length
    );
    if (targetPage !== currentPageRef.current) {
      setCurrentPage(targetPage);
    }
  }, [debouncedCursorLine, markdownText, pdfStructure?.pages?.length, setCurrentPage]);

  useEffect(() => {
    if (!isTauriRuntime) {
      return;
    }
    const runPreview = async () => {
      const requestId = previewRequestIdRef.current + 1;
      previewRequestIdRef.current = requestId;
      try {
        const previewPdfPath = await requestMarkdownPreview(debouncedMarkdown, requestId);
        if (!previewPdfPath) return;
        await loadPdf(previewPdfPath, { preserveCurrentPage: true });
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        showError(`プレビュー生成に失敗しました: ${errorMessage}`);
        logger.error('Error building markdown preview', error instanceof Error ? error : new Error(String(error)));
      }
    };
    runPreview().catch((error) => {
      logger.error('Unexpected preview build error', error instanceof Error ? error : new Error(String(error)));
    });
  }, [isTauriRuntime, debouncedMarkdown, requestMarkdownPreview, loadPdf, showError]);

  const shortcuts = useMemo(() => ({
    ...(isTauriRuntime ? { 'Ctrl+o': handleOpenMarkdown, 'Ctrl+s': handleSaveMarkdown } : {}),
    'Ctrl++': handleZoomIn,
    'Ctrl+-': handleZoomOut,
    'Ctrl+0': handleZoomReset,
  }), [isTauriRuntime, handleOpenMarkdown, handleSaveMarkdown, handleZoomIn, handleZoomOut, handleZoomReset]);

  useKeyboardShortcuts(shortcuts);

  return (
    <div className="app">
      <header className="app-header">
        <h1>miniPDF</h1>
        <div className="app-header-actions">
          <button onClick={handleOpenMarkdown} disabled={!isTauriRuntime}>MDを開く (Ctrl+O)</button>
          <button onClick={handleSaveMarkdown} disabled={!isTauriRuntime || !markdownText.trim()}>
            MD保存 (Ctrl+S)
          </button>
          <button onClick={handleSavePdf} disabled={!isTauriRuntime || !pdfStructure}>
            PDF保存
          </button>
          <KeyboardShortcutsHelp />
          <div className="zoom-controls">
            <button 
              onClick={handleZoomOut} 
              disabled={!pdfStructure || zoomLevel <= 0.5}
              title="ズームアウト (Ctrl+-)"
              className="zoom-button"
            >
              -
            </button>
            <span className="zoom-level">{Math.round(zoomLevel * 100)}%</span>
            <button 
              onClick={handleZoomIn} 
              disabled={!pdfStructure || zoomLevel >= 2}
              title="ズームイン (Ctrl++)"
              className="zoom-button"
            >
              +
            </button>
            <button 
              onClick={handleZoomReset} 
              disabled={!pdfStructure}
              className="zoom-reset-button"
              title="リセット (Ctrl+0)"
            >
              リセット
            </button>
          </div>
        </div>
      </header>
      <main className="app-main">
        <ErrorDisplay error={error} onDismiss={clearError} />
        <section className="markdown-pane">
          <div className="markdown-pane-header">
            <h2>Markdown Editor</h2>
            {isPreviewBuilding && <span className="preview-status">プレビュー更新中...</span>}
            {previewError && <span className="preview-status error">{previewError}</span>}
          </div>
          <textarea
            ref={markdownEditorRef}
            className="markdown-editor"
            value={markdownText}
            onKeyDown={(event) => {
              if (event.key !== 'Enter' || event.shiftKey || event.altKey || event.ctrlKey || event.metaKey) {
                return;
              }
              const el = markdownEditorRef.current;
              if (!el) return;
              event.preventDefault();
              const { text, cursorOffset } = insertMarkdownHardBreak(
                markdownText,
                el.selectionStart ?? markdownText.length,
                el.selectionEnd ?? markdownText.length
              );
              pendingCursorOffsetRef.current = cursorOffset;
              setMarkdownText(text);
              syncEditorCursor(text, cursorOffset);
              requestAnimationFrame(() => {
                restoreTextareaCursor(markdownEditorRef.current, cursorOffset);
                pendingCursorOffsetRef.current = null;
              });
            }}
            onChange={(event) => {
              const nextValue = event.target.value;
              const nextCursorOffset = resolveEditorCursorOffset(
                pendingCursorOffsetRef.current,
                event.target.selectionStart ?? nextValue.length
              );
              setMarkdownText(nextValue);
              syncEditorCursor(nextValue, nextCursorOffset);
            }}
            onSelect={() => {
              const el = markdownEditorRef.current;
              if (!el) return;
              syncEditorCursor(
                el.value,
                resolveEditorCursorOffset(pendingCursorOffsetRef.current, el.selectionStart ?? el.value.length)
              );
            }}
            onKeyUp={() => {
              const el = markdownEditorRef.current;
              if (!el) return;
              syncEditorCursor(
                el.value,
                resolveEditorCursorOffset(pendingCursorOffsetRef.current, el.selectionStart ?? el.value.length)
              );
            }}
            onClick={() => {
              const el = markdownEditorRef.current;
              if (!el) return;
              syncEditorCursor(
                el.value,
                resolveEditorCursorOffset(pendingCursorOffsetRef.current, el.selectionStart ?? el.value.length)
              );
            }}
            placeholder="# Markdownを入力してください"
          />
        </section>
        {isLoading && (
          <div className="loading-overlay">
            <div className="loading-spinner"></div>
            <p className="loading-text">処理中...</p>
          </div>
        )}
        <section className="preview-pane">
          <PDFViewer pdfStructure={pdfStructure} zoomLevel={zoomLevel} previewOnly />
        </section>
        <ToastContainer toasts={toasts} onDismiss={dismissToast} />
      </main>
    </div>
  );
}

export default App;
