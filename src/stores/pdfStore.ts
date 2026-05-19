import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';
import { PdfStructure, ImageElement, Page } from '../types/pdf';
import { logger } from '../utils/logger';
import { getSourcePageNumber } from '../utils/pageMapping';
import { isTauriRuntimeAvailable } from '../utils/tauriRuntime';

type PageProcessingError = { type: 'imageExtraction' | 'textExtraction' | 'unexpected'; pageNumber: number };

export interface MarkdownPreviewResult {
  filePath: string;
  linePageMap: number[];
}

interface PdfStore {
  pdfStructure: PdfStructure | null;
  selectedImageId: string | null;
  selectedTextBlockId: string | null;
  zoomLevel: number;
  error: string | null;
  currentPage: number;
  markdownText: string;
  isPreviewBuilding: boolean;
  previewError: string | null;
  previewPdfPath: string | null;
  previewHtml: string | null;
  previewLinePageMap: number[] | null;
  previewRequestId: number;
  isLoading: boolean; // why: PDF読み込み中のフラグ（競合状態を防ぐ）
  // alt: フラグなし（読み込み中に他の操作が実行される可能性がある）
  // evidence: フラグにより、読み込み中の操作をブロックできる
  isEditing: boolean; // why: 編集操作中のフラグ（同時編集操作の競合状態を防ぐ）
  // alt: フラグなし（複数の編集操作が同時に実行される可能性がある）
  // evidence: フラグにより、編集操作中の他の編集操作をブロックできる

  // Actions
  loadPdf: (filePath: string, options?: { preserveCurrentPage?: boolean }) => Promise<void>;
  selectImage: (imageId: string | null) => void;
  selectTextBlock: (textBlockId: string | null) => void;
  resizeImage: (imageId: string, width: number, height: number) => Promise<void>;
  moveImage: (imageId: string, x: number, y: number) => Promise<void>;
  moveTextBlock: (textBlockId: string, x: number, y: number) => Promise<void>;
  adjustPageBreak: (pageNumber: number, position: number) => Promise<void>;
  addPage: (pageNumber: number, width: number, height: number) => Promise<void>;
  deletePage: (pageNumber: number) => Promise<void>;
  reorderPages: (fromIndex: number, toIndex: number) => Promise<void>;
  editTextBlock: (textBlockId: string, newText: string) => Promise<void>;
  addTextBlock: (pageNumber: number, x: number, y: number, width: number, height: number, text: string, fontSize: number, lineHeight: number, fontFamily: string) => Promise<void>;
  insertImage: (pageNumber: number, x: number, y: number, width: number, height: number, imageData: string, format: string) => Promise<void>;
  savePdf: (filePath: string) => Promise<void>;
  setZoomLevel: (level: number) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
  setCurrentPage: (page: number) => void;
  setMarkdownText: (text: string) => void;
  requestMarkdownPreview: (markdown: string, requestId: number) => Promise<string | null>;
}

// セレクター関数
export const usePdfStructure = () => usePdfStore((state) => state.pdfStructure);
export const useSelectedTextBlockId = () => usePdfStore((state) => state.selectedTextBlockId);
export const useSelectedImageId = () => usePdfStore((state) => state.selectedImageId);
export const useZoomLevel = () => usePdfStore((state) => state.zoomLevel);

export const usePdfStore = create<PdfStore>((set, get) => ({
  pdfStructure: null,
  selectedImageId: null,
  selectedTextBlockId: null,
  zoomLevel: 1,
  error: null,
  currentPage: 1,
  markdownText: '',
  isPreviewBuilding: false,
  previewError: null,
  previewPdfPath: null,
  previewHtml: null,
  previewLinePageMap: null,
  previewRequestId: 0,
  isLoading: false,
  isEditing: false,

  loadPdf: async (filePath: string, options?: { preserveCurrentPage?: boolean }) => {
    // why: 既に読み込み中の場合は、新しい読み込みをブロック（競合状態を防ぐ）
    // alt: 読み込み中でも新しい読み込みを許可（状態が不整合になる可能性がある）
    // evidence: フラグにより、同時に複数の読み込み操作が実行されることを防ぐ
    const state = get();
    if (state.isLoading) {
      logger.warn('PDF is already loading, skipping new load request');
      return;
    }

    // why: PDF.jsオブジェクトとloadingTaskを管理して、エラー時に確実に破棄・キャンセルする
    // alt: 変数スコープ内で管理（エラー時に破棄されない可能性がある）
    // evidence: 変数を外側で宣言することで、catchブロックからもアクセスできる
    let pdf: import('pdfjs-dist').PDFDocumentProxy | null = null;
    let loadingTask: import('pdfjs-dist').PDFLoadingTask | null = null;
    const shouldPreserveCurrentPage = options?.preserveCurrentPage === true;
    
    try {
      logger.info('Loading PDF', { filePath });
      set({ 
        error: null, 
        pdfStructure: null,
        selectedImageId: null,
        selectedTextBlockId: null,
        currentPage: shouldPreserveCurrentPage ? state.currentPage : 1,
        zoomLevel: 1,
        isLoading: true, // why: 読み込み開始をマーク
      });
      
      const structure = await invoke<PdfStructure>('load_pdf', { filePath });
      logger.info('PDF loaded successfully', { 
        pages: (structure.pages || []).length,
        filePath 
      });
      try {
        const pdfjs = await import('pdfjs-dist');
        if (globalThis.window !== undefined && pdfjs.GlobalWorkerOptions) {
          pdfjs.GlobalWorkerOptions.workerSrc = new URL(
            'pdfjs-dist/build/pdf.worker.min.mjs',
            import.meta.url
          ).toString();
        }

        const { readFile } = await import('@tauri-apps/plugin-fs');
        // why: Tauri 2.0のreadFileは文字列を直接受け取る（型定義ではstring | URLだが、実行時は文字列が必要）
        // alt: { path: filePath }形式（型エラーが発生）
        // evidence: 型定義ファイルの例では文字列を直接渡している
        const fileData = await readFile(filePath);
        loadingTask = pdfjs.getDocument({ data: fileData });
        pdf = await loadingTask.promise;

        const { extractTextWithPDFjs } = await import('../utils/pdfTextExtractor');
        const { extractImageAreasWithPDFjs } = await import('../utils/pdfImageExtractor');

        // why: Promise.allを使用して並列処理を高速化するが、各ページの処理でエラーが発生しても全体が失敗しないようにする
        // alt: 順次処理（パフォーマンスが悪い）またはPromise.allで全体が失敗する（一部のページでエラーが発生すると全体が失敗する）
        // evidence: 各ページの処理を完全にラップすることで、予期しないエラーが発生しても処理を続行できる
        // why: エラーメッセージを集約して、複数のページでエラーが発生した場合でもすべてのエラーをユーザーに通知する
        // alt: 最後のエラーメッセージのみを表示（他のエラーが失われる）
        // evidence: エラーの数をカウントして集約することで、すべてのエラーをユーザーに通知できる
        // why: 各ページの処理でエラー情報を返し、最後に集約することで、並列処理での競合状態を完全に排除
        // alt: 共有配列にpush（理論的な競合状態の可能性がある）
        // evidence: 各ページの処理でエラー情報を返すことで、並列処理での競合状態を完全に排除できる
        const pageResults = await Promise.all(
          (structure.pages || []).map(async (page, index): Promise<{
            page: Page;
            errors: PageProcessingError[];
          }> => {
            // why: 各ページの処理を完全にラップして、予期しないエラーが発生しても処理を続行できるようにする
            // alt: ラップしない（予期しないエラーが発生するとPromise.all全体が失敗する）
            // evidence: 完全にラップすることで、どのようなエラーが発生しても処理を続行できる
            try {
              const pageNumber = page.pageNumber ?? (index + 1);
              const sourcePageNumber = getSourcePageNumber({ ...page, pageNumber });
              // why: 各ページの処理でエラー情報を収集するための配列を初期化
              // alt: 共有配列にpush（理論的な競合状態の可能性がある）
              // evidence: 各ページの処理でエラー情報を収集することで、並列処理での競合状態を完全に排除できる
              const errors: PageProcessingError[] = [];
              let imageAreas: Array<{ x: number; y: number; width: number; height: number }> = [];
              try {
                // why: ページ番号が有効な範囲内かチェック（PDF.jsは1ベースのページ番号を使用）
                // alt: チェックなし（無効なページ番号でエラーが発生）
                // evidence: ページ番号が範囲外の場合、PDF.jsはエラーを返す
                if (!pdf) {
                  throw new Error('PDF document is not loaded');
                }
                if (!sourcePageNumber || sourcePageNumber < 1 || sourcePageNumber > pdf.numPages) {
                  throw new Error(`Invalid page number: ${sourcePageNumber} (valid range: 1-${pdf.numPages})`);
                }
                const pdfjsImageAreas = await extractImageAreasWithPDFjs(
                  pdfjs,
                  pdf,
                  sourcePageNumber,
                  page.height
                );
                if (pdfjsImageAreas.length > 0) {
                  imageAreas = pdfjsImageAreas;
                } else {
                  imageAreas = (page.images || []).map(img => ({
                    x: img.x,
                    y: img.y,
                    width: img.width,
                    height: img.height,
                  }));
                }
              } catch (error) {
                // why: 画像領域の抽出に失敗した場合、警告ログを出力し、バックエンドの結果を使用
                // alt: エラーをユーザーに通知（画像領域の抽出失敗は致命的ではないが、ユーザーに通知する）
                // evidence: バックエンドの結果を使用することで、処理を続行できるが、ユーザーに通知することで問題を認識できる
                const errorMessage = error instanceof Error ? error.message : String(error);
                logger.warn('Failed to extract image areas with PDF.js, using backend result', { 
                  pageNumber,
                  error: errorMessage
                });
                // why: 画像領域の抽出失敗は致命的ではないが、ユーザーに通知する（軽微な警告として）
                // alt: エラーを無視（ユーザーが問題に気づかない）
                // evidence: ユーザーに通知することで、問題を認識できる
                // 注意: エラーメッセージは最後に集約して表示するため、ここではエラー情報を収集する
                errors.push({ type: 'imageExtraction', pageNumber });
                imageAreas = (page.images || []).map(img => ({
                  x: img.x,
                  y: img.y,
                  width: img.width,
                  height: img.height,
                }));
              }

              // why: ページ番号が有効な範囲内かチェック（PDF.jsは1ベースのページ番号を使用）
              // alt: チェックなし（無効なページ番号でエラーが発生）
              // evidence: ページ番号が範囲外の場合、PDF.jsはエラーを返す
              // why: TextBlock型を使用して型安全性を向上
              // alt: any[]型を使用（型安全性が低下）
              // evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
              let textBlocks: import('../types/pdf').TextBlock[] = [];
              try {
                if (pdf && sourcePageNumber && sourcePageNumber >= 1 && sourcePageNumber <= pdf.numPages) {
                  textBlocks = await extractTextWithPDFjs(
                pdfjs,
                pdf,
                    sourcePageNumber,
                page.width,
                page.height,
                imageAreas
              );
                }
              } catch (error) {
                const errorMessage = error instanceof Error ? error.message : String(error);
                logger.warn('PDF.js text extraction failed, using backend result', { 
                  pageNumber,
                  error: errorMessage
                });
                // why: テキスト抽出失敗は致命的ではないが、ユーザーに通知する（軽微な警告として）
                // alt: エラーを無視（ユーザーが問題に気づかない）
                // evidence: ユーザーに通知することで、問題を認識できる
                // 注意: エラーメッセージは最後に集約して表示するため、ここではエラー情報を収集する
                errors.push({ type: 'textExtraction', pageNumber });
                textBlocks = page.textBlocks || [];
              }
              return {
                page: {
                  ...page,
                  pageNumber,
                  textBlocks: textBlocks.length > 0 ? textBlocks : (page.textBlocks || []),
                },
                errors,
              };
            } catch (error) {
              // why: 予期しないエラーが発生した場合でも、ページを返して処理を続行する
              // alt: エラーを再スロー（Promise.all全体が失敗する）
              // evidence: 予期しないエラーが発生しても、バックエンドの結果を使用することで処理を続行できる
              const pageNumber = page.pageNumber ?? (index + 1);
              const errorMessage = error instanceof Error ? error.message : String(error);
              logger.error('Unexpected error processing page, using backend result', 
                error instanceof Error ? error : new Error(errorMessage),
                { pageNumber }
              );
              // 注意: エラーメッセージは最後に集約して表示するため、ここではエラー情報を返す
              const errors: Array<{ type: 'imageExtraction' | 'textExtraction' | 'unexpected'; pageNumber: number }> = [
                { type: 'unexpected', pageNumber }
              ];
              // why: エラーが発生した場合でも、元のページデータを返して処理を続行する
              // alt: エラーを再スロー（全体が失敗する）
              // evidence: 元のページデータを返すことで、他のページの処理を続行できる
              return {
                page: {
                  ...page,
                  pageNumber,
                  textBlocks: page.textBlocks || [],
                },
                errors,
              };
            }
          })
        );

        // why: 各ページの処理結果からページデータとエラー情報を分離
        // alt: そのまま使用（型エラーが発生する）
        // evidence: 各ページの処理でエラー情報を返すように変更したため、結果を分離する必要がある
        const updatedPages = pageResults.map(result => result.page);
        const pageErrors = pageResults.flatMap(result => result.errors);

        // why: エラーメッセージを集約して、複数のページでエラーが発生した場合でもすべてのエラーをユーザーに通知する
        // alt: 最後のエラーメッセージのみを表示（他のエラーが失われる）
        // evidence: エラーの数をカウントして集約することで、すべてのエラーをユーザーに通知できる
        // why: 各ページの処理で収集したエラー情報を集約して、エラーメッセージを生成する
        // alt: 共有オブジェクトを直接更新（理論的な競合状態の可能性がある）
        // evidence: エラー情報を収集してから集約することで、並列処理での競合状態を防げる
        // why: 1回のループでエラー数をカウントすることで、パフォーマンスを向上
        // alt: filter()を3回呼び出す（O(3n)の時間計算量）
        // evidence: 1回のループでカウントすることで、O(n)の時間計算量に改善できる
        const errorCounts = {
          imageExtraction: 0,
          textExtraction: 0,
          unexpected: 0,
        };
        for (const error of pageErrors) {
          switch (error.type) {
            case 'imageExtraction':
              errorCounts.imageExtraction++;
              break;
            case 'textExtraction':
              errorCounts.textExtraction++;
              break;
            case 'unexpected':
              errorCounts.unexpected++;
              break;
          }
        }
        // why: エラーメッセージを簡潔にし、UIでの表示を改善する
        // alt: 長いメッセージをそのまま表示（UIで読みにくい）
        // evidence: エラーメッセージを簡潔にすることで、ユーザーが情報を理解しやすくなる
        const errorMessages: string[] = [];
        if (errorCounts.imageExtraction > 0) {
          errorMessages.push(`画像領域抽出失敗: ${errorCounts.imageExtraction}ページ`);
        }
        if (errorCounts.textExtraction > 0) {
          errorMessages.push(`テキスト抽出失敗: ${errorCounts.textExtraction}ページ`);
        }
        if (errorCounts.unexpected > 0) {
          errorMessages.push(`予期しないエラー: ${errorCounts.unexpected}ページ`);
        }
        // why: エラーが発生した場合のみエラーメッセージを設定し、発生しなかった場合は明示的にクリア
        // alt: エラーが発生しなかった場合、以前のエラーメッセージが残る可能性がある
        // evidence: エラーが発生しなかった場合でも、明示的にエラーをクリアすることで、状態の一貫性を保つ
        if (errorMessages.length > 0) {
          // why: エラーメッセージを改行で区切って表示することで、UIでの可読性を向上
          // alt: スペースで結合（長いメッセージが読みにくい）
          // evidence: 改行で区切ることで、各エラーを明確に区別できる
          const errorMessage = errorMessages.join('\n');
          // why: エラーメッセージの長さを制限して、UIでの表示を改善
          // alt: 制限なし（非常に長いメッセージが表示される可能性がある）
          // evidence: 長すぎるメッセージはユーザーにとって読みにくい
          const maxLength = 500;
          const truncatedMessage = errorMessage.length > maxLength 
            ? errorMessage.substring(0, maxLength) + '...'
            : errorMessage;
          set({ 
            error: truncatedMessage
          });
        } else {
          // why: エラーが発生しなかった場合、明示的にエラーをクリア
          // alt: エラーをクリアしない（以前のエラーメッセージが残る可能性がある）
          // evidence: エラーが発生しなかった場合でも、明示的にエラーをクリアすることで、状態の一貫性を保つ
          set({ 
            error: null
          });
        }

        const updatedStructure: PdfStructure = {
          ...structure,
          pages: updatedPages,
          metadata: structure.metadata || {
            title: undefined,
            author: undefined,
            subject: undefined,
            creator: undefined,
            producer: undefined,
            creationDate: undefined,
            modificationDate: undefined,
          },
        };

        // why: 成功時にもPDF.jsオブジェクトを破棄してメモリリークを防ぐ
        // alt: 破棄しない（メモリリークが発生する可能性がある）
        // evidence: pdf.destroy()により、PDF.jsのリソースが解放される
        // 注意: PDFViewerコンポーネントは独自にPDF.jsオブジェクトを作成して使用するため、
        // loadPdf関数内で作成したPDF.jsオブジェクトは破棄しても問題ない
        if (pdf) {
          try {
            pdf.destroy();
          } catch (destroyError) {
            // 既に破棄されている場合はエラーを無視
            logger.warn('Failed to destroy PDF document after successful load', { error: destroyError instanceof Error ? destroyError.message : String(destroyError) });
          }
        }
        
        set({ 
          pdfStructure: updatedStructure,
          selectedImageId: null,
          selectedTextBlockId: null,
          currentPage: shouldPreserveCurrentPage ? state.currentPage : 1,
          zoomLevel: 1,
          isLoading: false, // why: 読み込み完了をマーク
        });
      } catch (textExtractionError) {
        // why: PDF.jsのテキスト抽出に失敗した場合、ユーザーに通知する
        // alt: 警告ログのみ（ユーザーがエラーに気づかない）
        // evidence: エラーが発生した場合、ユーザーに通知することで、問題を認識できる
        const errorMessage = textExtractionError instanceof Error 
          ? textExtractionError.message 
          : 'PDF.jsのテキスト抽出に失敗しました。バックエンドの結果を使用します。';
        logger.warn('PDF.js text extraction failed, using backend result', { 
          error: errorMessage 
        });
        
        // why: エラー時にloadingTaskをキャンセルしてリソースを解放する
        // alt: キャンセルしない（リソースが解放されない可能性がある）
        // evidence: loadingTask.cancel()により、PDF.jsの読み込み処理がキャンセルされる
        if (loadingTask) {
          try {
            loadingTask.cancel();
          } catch (cancelError) {
            // 既にキャンセルされている場合はエラーを無視
            logger.warn('Failed to cancel loading task after text extraction error', { error: cancelError instanceof Error ? cancelError.message : String(cancelError) });
          }
        }
        
        // why: エラー時にPDF.jsオブジェクトを破棄してメモリリークを防ぐ
        // alt: 破棄しない（メモリリークが発生する可能性がある）
        // evidence: pdf.destroy()により、PDF.jsのリソースが解放される
        if (pdf) {
          try {
            pdf.destroy();
          } catch (destroyError) {
            // 既に破棄されている場合はエラーを無視
            logger.warn('Failed to destroy PDF document after text extraction error', { error: destroyError instanceof Error ? destroyError.message : String(destroyError) });
          }
        }
        
        set({ 
          pdfStructure: structure,
          selectedImageId: null,
          selectedTextBlockId: null,
          currentPage: shouldPreserveCurrentPage ? state.currentPage : 1,
          zoomLevel: 1,
          isLoading: false,
          error: errorMessage, // why: エラーをユーザーに通知
        });
      }
    } catch (error) {
      // why: エラーメッセージを統一して、ユーザーに分かりやすく表示する
      // alt: エラーメッセージが統一されていない（ユーザーが混乱する可能性がある）
      // evidence: 統一されたエラーメッセージにより、ユーザーが問題を理解しやすくなる
      const errorMessage = error instanceof Error ? error.message : 'PDFの読み込みに失敗しました';
      logger.error('Failed to load PDF', error instanceof Error ? error : new Error(errorMessage), { filePath });
      
      // why: エラー時にloadingTaskをキャンセルしてリソースを解放する
      // alt: キャンセルしない（リソースが解放されない可能性がある）
      // evidence: loadingTask.cancel()により、PDF.jsの読み込み処理がキャンセルされる
      // 注意: loadingTaskは内側のtry-catchブロック内で作成されるが、外側のcatchブロックからもアクセスできるように外側で宣言している
      // why: != nullを使用してnullとundefinedの両方を一度にチェック（より簡潔）
      // alt: !== null && !== undefined（冗長）
      // evidence: != nullはnullとundefinedの両方をチェックする
      if (loadingTask != null) {
        try {
          (loadingTask as import('pdfjs-dist').PDFLoadingTask).cancel();
        } catch (cancelError) {
          // 既にキャンセルされている場合はエラーを無視
          logger.warn('Failed to cancel loading task after load error', { error: cancelError instanceof Error ? cancelError.message : String(cancelError) });
        }
      }
      
      // why: エラー時にPDF.jsオブジェクトが存在する場合は破棄してメモリリークを防ぐ
      // alt: 破棄しない（メモリリークが発生する可能性がある）
      // evidence: pdf.destroy()により、PDF.jsのリソースが解放される
      // 注意: pdf変数は内側のtry-catchブロック内で宣言されているため、ここではアクセスできない
      // ただし、外側のcatchブロックでエラーが発生した場合は、pdfはまだ作成されていない可能性が高い
      // 修正: pdf変数を外側で宣言することで、catchブロックからもアクセスできるようにした
      // why: != nullを使用してnullとundefinedの両方を一度にチェック（より簡潔）
      // alt: !== null && !== undefined（冗長）
      // evidence: != nullはnullとundefinedの両方をチェックする
      if (pdf != null) {
        try {
          (pdf as import('pdfjs-dist').PDFDocumentProxy).destroy();
        } catch (destroyError) {
          // 既に破棄されている場合はエラーを無視
          logger.warn('Failed to destroy PDF document after load error', { error: destroyError instanceof Error ? destroyError.message : String(destroyError) });
        }
      }
      
      set({ 
        error: errorMessage, 
        pdfStructure: null,
        selectedImageId: null,
        selectedTextBlockId: null,
        currentPage: shouldPreserveCurrentPage ? state.currentPage : 1,
        zoomLevel: 1,
        isLoading: false, // why: エラー時も読み込み完了をマーク
      });
      throw error;
    }
  },

  selectImage: (imageId: string | null) => {
    set({ selectedImageId: imageId });
  },

  resizeImage: async (imageId: string, width: number, height: number) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(width) || Number.isNaN(height) || !Number.isFinite(width) || !Number.isFinite(height)) {
        throw new TypeError('無効なサイズです（NaNまたはInfinity）');
      }
      if (width <= 0 || height <= 0) {
        throw new Error('サイズは0より大きい値である必要があります');
      }
      if (width > 10000 || height > 10000) {
        throw new Error('サイズが大きすぎます（最大10000ポイント）');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      // why: 編集操作中は他の編集操作をブロック（同時編集操作の競合状態を防ぐ）
      // alt: 編集操作を許可（複数の編集操作が同時に実行され、状態が不整合になる可能性がある）
      // evidence: フラグにより、編集操作中の他の編集操作をブロックできる
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }

      set({ isEditing: true }); // why: 編集操作開始をマーク

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const targetImage = snapshotPdfStructure.pages
        .flatMap(p => p.images || [])
        .find(img => img.id === imageId);
      
      if (!targetImage) {
        set({ isEditing: false });
        // why: より具体的なエラーメッセージを提供してデバッグを容易にする
        // alt: 汎用的なエラーメッセージ（デバッグが困難）
        // evidence: 具体的なエラーメッセージにより、問題の原因を特定しやすくなる
        const allImageIds = snapshotPdfStructure.pages.flatMap(p => (p.images || []).map(img => img.id));
        throw new Error(`画像が見つかりません（ID: ${imageId}）。利用可能なID: ${allImageIds.length > 0 ? allImageIds.slice(0, 5).join(', ') : 'なし'}${allImageIds.length > 5 ? '...' : ''}`);
      }

      const format = targetImage.format === 'png' ? 'png' : 'jpeg';
      const updatedImage = await invoke<ImageElement>('resize_image', {
        imageId,
        newWidth: width,
        newHeight: height,
        imageData: targetImage.data,
        format,
        x: targetImage.x,
        y: targetImage.y,
        originalWidth: targetImage.originalWidth,
        originalHeight: targetImage.originalHeight,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }
      const updatedPdfStructure = {
        ...currentState.pdfStructure,
        pages: currentState.pdfStructure.pages.map(page => ({
          ...page,
          images: (page.images || []).map(img =>
            img.id === imageId ? updatedImage : img
          ),
        })),
      };

      set({ pdfStructure: updatedPdfStructure, isEditing: false }); // why: 編集操作完了をマーク
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '画像のリサイズに失敗しました';
      set({ error: errorMessage, isEditing: false }); // why: エラー時も編集操作完了をマーク
      throw error;
    }
  },

  moveImage: async (imageId: string, x: number, y: number) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(x) || Number.isNaN(y) || !Number.isFinite(x) || !Number.isFinite(y)) {
        throw new TypeError('無効な座標です（NaNまたはInfinity）');
      }
      if (x < 0 || y < 0) {
        throw new Error('座標は0以上である必要があります');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const updatedStructure = await invoke<PdfStructure>('move_image', {
        pdfStructure: snapshotPdfStructure,
        imageId,
        newX: x,
        newY: y,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      // 注意: invokeの結果（updatedStructure）を直接使用する方が安全（バックエンドが正しい状態を返す）
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      set({ 
        pdfStructure: updatedStructure,
        selectedImageId: imageId,
        selectedTextBlockId: null,
        isEditing: false,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '画像の移動に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  moveTextBlock: async (textBlockId: string, x: number, y: number) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(x) || Number.isNaN(y) || !Number.isFinite(x) || !Number.isFinite(y)) {
        throw new TypeError('無効な座標です（NaNまたはInfinity）');
      }
      if (x < 0 || y < 0) {
        throw new Error('座標は0以上である必要があります');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const updatedStructure = await invoke<PdfStructure>('move_text_block', {
        pdfStructure: snapshotPdfStructure,
        textBlockId,
        newX: x,
        newY: y,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      // 注意: invokeの結果（updatedStructure）を直接使用する方が安全（バックエンドが正しい状態を返す）
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      set({ 
        pdfStructure: updatedStructure,
        selectedTextBlockId: textBlockId,
        selectedImageId: null,
        isEditing: false,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'テキストブロックの移動に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  adjustPageBreak: async (pageNumber: number, position: number) => {
    try {
      set({ error: null });
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const updatedStructure = await invoke<PdfStructure>('adjust_page_break', {
        pdfStructure: snapshotPdfStructure,
        pageNumber,
        newPosition: position,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      set({ pdfStructure: updatedStructure, isEditing: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '改ページ位置の調整に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },


  savePdf: async (filePath: string) => {
    try {
      set({ error: null });
      
      // why: filePathのバリデーションを行い、無効な値のリクエストを防ぐ
      // alt: バリデーションなし（無効なファイルパスでエラーが発生）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (!filePath || typeof filePath !== 'string' || filePath.trim().length === 0) {
        throw new Error('ファイルパスが指定されていません');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }

      set({ isEditing: true });

      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません');
      }

      const shouldReusePreviewPdf =
        !!currentState.previewPdfPath &&
        currentState.previewPdfPath === currentState.pdfStructure.filePath;

      if (shouldReusePreviewPdf) {
        try {
          const pdfData = await readFile(currentState.previewPdfPath as string);
          if (!pdfData || pdfData.length === 0) {
            throw new Error('プレビューPDFの読み込みに失敗しました（データが空です）');
          }

          await invoke('save_pdf', {
            filePath,
            pdfData,
          });

          logger.info('PDF saved successfully from preview source', { filePath });
          set({ isEditing: false });
          return;
        } catch (error) {
          logger.warn('Failed to reuse preview PDF, falling back to regeneration', {
            error: error instanceof Error ? error.message : String(error),
          });
        }
      }

      const shouldReuseSourcePdf =
        !currentState.previewPdfPath &&
        !!currentState.pdfStructure.filePath &&
        currentState.pdfStructure.pages.every((page) =>
          (page.images || []).every((image) => !image.data || image.data.trim().length === 0)
        );

      if (shouldReuseSourcePdf) {
        try {
          const sourceBytes = await readFile(currentState.pdfStructure.filePath);
          if (!sourceBytes || sourceBytes.length === 0) {
            throw new Error('元のPDFの読み込みに失敗しました（データが空です）');
          }

          await invoke('save_pdf', {
            filePath,
            pdfData: sourceBytes,
          });

          logger.info('PDF saved successfully from source file', { filePath });
          set({ isEditing: false });
          return;
        } catch (error) {
          logger.warn('Failed to reuse source PDF, falling back to regeneration', {
            error: error instanceof Error ? error.message : String(error),
          });
        }
      }

      // why: 最新のpdfStructureを使用してPDFを生成（編集内容を反映するため）
      // alt: 古いpdfStructureを使用（編集内容が反映されない）
      // evidence: state.pdfStructureを使用することで、最新の編集内容が反映される
      const pdfData = await invoke<number[]>('generate_pdf', {
        pdfStructure: currentState.pdfStructure,
      });

      // why: pdfDataが空の場合をチェックして、無効なPDFの保存を防ぐ
      // alt: チェックなし（空のPDFが保存される可能性がある）
      // evidence: 空のPDFデータの保存を防ぐことで、ユーザーに問題を通知できる
      if (!pdfData || pdfData.length === 0) {
        set({ isEditing: false });
        throw new Error('PDFデータの生成に失敗しました（データが空です）');
      }
      
      await invoke('save_pdf', {
        filePath,
        pdfData: new Uint8Array(pdfData),
      });
      
      logger.info('PDF saved successfully', { filePath });
      set({ isEditing: false });
    } catch (error) {
      // why: エラーメッセージを統一して、ユーザーに分かりやすく表示する
      // alt: エラーメッセージが統一されていない（ユーザーが混乱する可能性がある）
      // evidence: 統一されたエラーメッセージにより、ユーザーが問題を理解しやすくなる
      const errorMessage = error instanceof Error ? error.message : '保存に失敗しました';
      logger.error('Failed to save PDF', error instanceof Error ? error : new Error(errorMessage), { filePath });
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  setZoomLevel: (level: number) => {
    set({ zoomLevel: Math.max(0.5, Math.min(2, level)) });
  },

  setError: (error: string | null) => {
    set({ error });
  },

  selectTextBlock: (textBlockId: string | null) => {
    set({ selectedTextBlockId: textBlockId });
  },

  addPage: async (pageNumber: number, width: number, height: number) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(pageNumber) || !Number.isFinite(pageNumber) || pageNumber <= 0) {
        throw new TypeError('無効なページ番号です');
      }
      if (Number.isNaN(width) || Number.isNaN(height) || !Number.isFinite(width) || !Number.isFinite(height)) {
        throw new TypeError('無効なページサイズです（NaNまたはInfinity）');
      }
      if (width <= 0 || height <= 0) {
        throw new Error('ページサイズは0より大きい値である必要があります');
      }
      if (width > 10000 || height > 10000) {
        throw new Error('ページサイズが大きすぎます（最大10000ポイント）');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }
      if (pageNumber > state.pdfStructure.pages.length + 1) {
        throw new Error(`無効なページ番号です（最大: ${state.pdfStructure.pages.length + 1}）`);
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const updatedStructure = await invoke<PdfStructure>('add_page', {
        pdfStructure: snapshotPdfStructure,
        pageNumber,
        width,
        height,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      set({ pdfStructure: updatedStructure, isEditing: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'ページの追加に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  deletePage: async (pageNumber: number) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(pageNumber) || !Number.isFinite(pageNumber) || pageNumber <= 0) {
        throw new TypeError('無効なページ番号です');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }
      if (pageNumber > state.pdfStructure.pages.length) {
        throw new Error(`無効なページ番号です（最大: ${state.pdfStructure.pages.length}）`);
      }
      if (state.pdfStructure.pages.length === 0) {
        throw new Error('削除できるページがありません');
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      // why: Rust側でページ削除とページ番号の再割り当てが正しく処理されるため、そのまま使用
      // alt: 複雑なマッピングロジックを使用（不整合が発生する可能性）
      // evidence: Rust側のdelete_pageとrenumber_pagesにより、ページ番号が正しく管理される
      const updatedStructure = await invoke<PdfStructure>('delete_page', {
        pdfStructure: snapshotPdfStructure,
        pageNumber,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      const nextPageCount = updatedStructure.pages.length;
      const nextCurrentPage = Math.min(
        Math.max(1, get().currentPage),
        Math.max(1, nextPageCount)
      );

      if (nextPageCount === 0) {
        set({
          pdfStructure: null,
          selectedImageId: null,
          selectedTextBlockId: null,
          currentPage: 1,
          zoomLevel: 1,
          isEditing: false,
        });
      } else {
        set({ 
          pdfStructure: updatedStructure,
          selectedImageId: null,
          selectedTextBlockId: null,
          currentPage: nextCurrentPage,
          isEditing: false,
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'ページの削除に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  reorderPages: async (fromIndex: number, toIndex: number) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(fromIndex) || !Number.isFinite(fromIndex) || fromIndex <= 0) {
        throw new TypeError('無効な開始インデックスです');
      }
      if (Number.isNaN(toIndex) || !Number.isFinite(toIndex) || toIndex <= 0) {
        throw new TypeError('無効な終了インデックスです');
      }
      if (fromIndex === toIndex) {
        throw new Error('開始インデックスと終了インデックスが同じです');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }
      if (fromIndex > state.pdfStructure.pages.length || toIndex > state.pdfStructure.pages.length) {
        throw new Error(`無効なインデックスです（最大: ${state.pdfStructure.pages.length}）`);
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const updatedStructure = await invoke<PdfStructure>('reorder_pages', {
        pdfStructure: snapshotPdfStructure,
        fromIndex,
        toIndex,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      // why: テキストブロックのIDは一意の識別子として使用されるため、変更しない
      // alt: IDを再生成（既存の選択状態や編集状態が失われる）
      // evidence: IDを変更すると、既存のテキストブロックを識別できなくなり、選択状態や編集状態が失われる
      // 注意: ページ番号が再割り当てされても、テキストブロックのIDは変更しない
      const finalStructure = updatedStructure;

      const nextPageCount = finalStructure.pages.length;
      const nextCurrentPage = Math.min(
        Math.max(1, get().currentPage),
        Math.max(1, nextPageCount)
      );

      set({ 
        pdfStructure: finalStructure,
        selectedImageId: null,
        selectedTextBlockId: null,
        currentPage: nextCurrentPage,
        isEditing: false,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'ページの並び替えに失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  editTextBlock: async (textBlockId: string, newText: string) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (!textBlockId || textBlockId.trim().length === 0) {
        throw new Error('テキストブロックIDが空です');
      }
      if (newText === undefined || newText === null) {
        throw new Error('テキストが指定されていません');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const updatedStructure = await invoke<PdfStructure>('edit_text_block', {
        pdfStructure: snapshotPdfStructure,
        textBlockId,
        newText,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      set({ pdfStructure: updatedStructure, isEditing: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'テキストブロックの編集に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  addTextBlock: async (pageNumber: number, x: number, y: number, width: number, height: number, text: string, fontSize: number, lineHeight: number, fontFamily: string) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(pageNumber) || !Number.isFinite(pageNumber) || pageNumber <= 0) {
        throw new TypeError('無効なページ番号です');
      }
      if (Number.isNaN(x) || Number.isNaN(y) || !Number.isFinite(x) || !Number.isFinite(y)) {
        throw new TypeError('無効な座標です（NaNまたはInfinity）');
      }
      if (x < 0 || y < 0) {
        throw new Error('座標は0以上である必要があります');
      }
      if (Number.isNaN(width) || Number.isNaN(height) || !Number.isFinite(width) || !Number.isFinite(height)) {
        throw new TypeError('無効なサイズです（NaNまたはInfinity）');
      }
      if (width <= 0 || height <= 0) {
        throw new Error('サイズは0より大きい値である必要があります');
      }
      if (Number.isNaN(fontSize) || !Number.isFinite(fontSize) || fontSize <= 0) {
        throw new TypeError('無効なフォントサイズです');
      }
      if (Number.isNaN(lineHeight) || !Number.isFinite(lineHeight) || lineHeight < 0.5 || lineHeight > 2) {
        throw new TypeError('行間は0.5から2.0の範囲である必要があります');
      }
      if (!text || text.trim().length === 0) {
        throw new Error('テキストが空です');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }
      if (pageNumber > state.pdfStructure.pages.length + 1) {
        throw new Error(`無効なページ番号です（最大: ${state.pdfStructure.pages.length + 1}）`);
      }

      set({ isEditing: true });

      const updatedStructure = await invoke<PdfStructure>('add_text_block', {
        pdfStructure: state.pdfStructure,
        pageNumber,
        x,
        y,
        width,
        height,
        text,
        fontSize,
        lineHeight,
        fontFamily,
      });

      set({ pdfStructure: updatedStructure, isEditing: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'テキストブロックの追加に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  insertImage: async (pageNumber: number, x: number, y: number, width: number, height: number, imageData: string, format: string) => {
    try {
      set({ error: null });
      
      // why: フロントエンドでバリデーションを行い、無効な値のリクエストを防ぐ（UXの向上）
      // alt: バリデーションなし（バックエンドでエラーが返されるまで待つ）
      // evidence: フロントエンドでバリデーションを行うことで、即座にエラーメッセージを表示できる
      if (Number.isNaN(pageNumber) || !Number.isFinite(pageNumber) || pageNumber <= 0) {
        throw new TypeError('無効なページ番号です');
      }
      if (Number.isNaN(x) || Number.isNaN(y) || !Number.isFinite(x) || !Number.isFinite(y)) {
        throw new TypeError('無効な座標です（NaNまたはInfinity）');
      }
      if (x < 0 || y < 0) {
        throw new Error('座標は0以上である必要があります');
      }
      if (Number.isNaN(width) || Number.isNaN(height) || !Number.isFinite(width) || !Number.isFinite(height)) {
        throw new TypeError('無効なサイズです（NaNまたはInfinity）');
      }
      if (width <= 0 || height <= 0) {
        throw new Error('サイズは0より大きい値である必要があります');
      }
      if (!imageData || imageData.trim().length === 0) {
        throw new Error('画像データが空です');
      }
      if (format !== 'png' && format !== 'jpeg') {
        throw new Error('無効な画像形式です（pngまたはjpegのみ）');
      }
      
      const state = get();
      if (state.isLoading) {
        throw new Error('PDFの読み込み中です。しばらくお待ちください。');
      }
      if (state.isEditing) {
        throw new Error('他の編集操作が実行中です。しばらくお待ちください。');
      }
      if (!state.pdfStructure) {
        throw new Error('PDFが読み込まれていません');
      }
      if (pageNumber > state.pdfStructure.pages.length) {
        throw new Error(`無効なページ番号です（最大: ${state.pdfStructure.pages.length}）`);
      }

      set({ isEditing: true });

      // why: 操作開始時にpdfStructureのスナップショットを取得して、状態の不整合を防ぐ
      // alt: 操作中に状態が変更される可能性がある（状態の不整合が発生する）
      // evidence: スナップショットにより、操作開始時点の状態を保持できる
      const snapshotPdfStructure = { ...state.pdfStructure };

      const updatedStructure = await invoke<PdfStructure>('insert_image', {
        pdfStructure: snapshotPdfStructure,
        pageNumber,
        x,
        y,
        width,
        height,
        imageData,
        format,
      });

      // why: 操作完了時に最新の状態を再取得してから更新（状態の不整合を防ぐ）
      // alt: スナップショットの状態をそのまま使用（操作中に変更された状態が反映されない）
      // evidence: 最新の状態を再取得することで、操作中に変更された状態も反映される
      const currentState = get();
      if (!currentState.pdfStructure) {
        set({ isEditing: false });
        throw new Error('PDFが読み込まれていません（操作中にPDFが削除されました）');
      }

      set({ pdfStructure: updatedStructure, isEditing: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '画像の挿入に失敗しました';
      set({ error: errorMessage, isEditing: false });
      throw error;
    }
  },

  clearError: () => {
    set({ error: null });
  },
  setMarkdownText: (text: string) => {
    set({ markdownText: text });
  },
  requestMarkdownPreview: async (markdown: string, requestId: number) => {
    try {
      if (!isTauriRuntimeAvailable()) {
        set({
          isPreviewBuilding: false,
          previewError: 'ブラウザモードではPDFライブプレビューは利用できません。tauri:dev で確認してください。',
          previewPdfPath: null,
          previewHtml: null,
          previewLinePageMap: null,
          previewRequestId: requestId,
        });
        return null;
      }
      if (markdown.trim().length === 0) {
        set({
          isPreviewBuilding: false,
          previewError: null,
          previewPdfPath: null,
          previewHtml: null,
          previewLinePageMap: null,
          previewRequestId: requestId,
        });
        return null;
      }
      set({
        isPreviewBuilding: true,
        previewError: null,
        previewRequestId: requestId,
      });
      const preview = await invoke<MarkdownPreviewResult>('render_markdown_to_pdf_preview', {
        markdown,
      });
      const current = get();
      if (current.previewRequestId !== requestId) {
        return null;
      }
      set({
        isPreviewBuilding: false,
        previewError: null,
        previewPdfPath: preview.filePath,
        previewHtml: null,
        previewLinePageMap: preview.linePageMap,
      });
      return preview.filePath;
    } catch (error) {
      const current = get();
      if (current.previewRequestId !== requestId) {
        return null;
      }
      const message = error instanceof Error ? error.message : String(error);
      set({
        isPreviewBuilding: false,
        previewError: message,
        previewHtml: null,
        previewLinePageMap: null,
      });
      throw error;
    }
  },
  setCurrentPage: (page: number) => {
    set((state) => {
      const pageNumbers = state.pdfStructure?.pages?.map(p => p.pageNumber) ?? [];

      if (pageNumbers.length === 0) {
        return {
          currentPage: 1,
          selectedImageId: null,
          selectedTextBlockId: null,
        };
      }

      const sortedPages = [...pageNumbers].sort((a, b) => a - b);
      const minPage = sortedPages[0];
      const maxPage = sortedPages[sortedPages.length - 1];
      const clamped = Math.min(Math.max(minPage, page), maxPage);

      // choose the closest existing page <= clamped; if none, fallback to smallest
      let targetPage = sortedPages
        .filter(pn => pn <= clamped)
        .sort((a, b) => b - a)[0];
      if (!targetPage) {
        targetPage = minPage;
      }
      let selectedImageId = state.selectedImageId;
      let selectedTextBlockId = state.selectedTextBlockId;

      if (state.pdfStructure) {
        const pageData = state.pdfStructure.pages.find(p => p.pageNumber === targetPage);
        if (pageData) {
          const imageIds = new Set((pageData.images || []).map(img => img.id));
          const textIds = new Set((pageData.textBlocks || []).map(tb => tb.id));
          if (selectedImageId && !imageIds.has(selectedImageId)) {
            selectedImageId = null;
          }
          if (selectedTextBlockId && !textIds.has(selectedTextBlockId)) {
            selectedTextBlockId = null;
          }
        } else {
          selectedImageId = null;
          selectedTextBlockId = null;
        }
      } else {
        selectedImageId = null;
        selectedTextBlockId = null;
      }

      return { currentPage: targetPage, selectedImageId, selectedTextBlockId };
    });
  },
}));
