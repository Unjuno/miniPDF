// PDF.jsの型定義
// why: PDF.jsは動的にインポートされるため、型定義を提供して型安全性を向上
// alt: any型を使用（型安全性が低下）
// evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
// assumption: PDF.jsのAPIは比較的安定している

declare module 'pdfjs-dist' {
  export interface PDFDocumentProxy {
    numPages: number;
    getPage(pageNumber: number): Promise<PDFPageProxy>;
    destroy(): void;
  }

  export interface PDFPageProxy {
    getTextContent(options?: {
      normalizeWhitespace?: boolean;
      disableCombineTextItems?: boolean;
    }): Promise<TextContent>;
    getOperatorList(): Promise<OperatorList>;
    getViewport(options: { scale: number; rotation?: number }): Viewport;
    render(options: RenderParameters): RenderTask;
  }

  export interface TextContent {
    items: TextItem[];
  }

  export interface TextItem {
    str: string;
    dir: string;
    width: number;
    height: number;
    transform: number[];
    fontName: string;
  }

  export interface OperatorList {
    fnArray: number[];
    argsArray: any[];
  }

  export interface Viewport {
    width: number;
    height: number;
  }

  export interface RenderParameters {
    canvasContext: CanvasRenderingContext2D;
    viewport: Viewport;
  }

  export interface RenderTask {
    promise: Promise<void>;
    cancel(): void;
  }

  export interface PDFLoadingTask {
    promise: Promise<PDFDocumentProxy>;
    cancel(): void;
  }

  export interface GlobalWorkerOptions {
    workerSrc: string;
  }

  export function getDocument(src: {
    data: Uint8Array;
    disableAutoFetch?: boolean;
    disableStream?: boolean;
  }): PDFLoadingTask;

  export const GlobalWorkerOptions: GlobalWorkerOptions;
}

