// PDF全体の構造
export interface PdfStructure {
  pages: Page[];
  metadata: PdfMetadata;
  filePath: string;
}

// ページ情報
export interface Page {
  pageNumber: number;        // 1始まり
  sourcePageNumber?: number; // 元のPDF上のページ番号（削除/並び替え時の表示用）
  width: number;             // ポイント単位
  height: number;            // ポイント単位
  images: ImageElement[];    // 画像要素
  textBlocks: TextBlock[];    // テキストブロック
}

// 画像要素
export interface ImageElement {
  id: string;                // 一意のID
  x: number;                 // X座標（ポイント単位）
  y: number;                 // Y座標（ポイント単位）
  width: number;             // 幅（ポイント単位）
  height: number;            // 高さ（ポイント単位）
  originalWidth: number;     // 元の幅
  originalHeight: number;    // 元の高さ
  data: string;              // Base64エンコードされた画像データ
  format: 'png' | 'jpeg';    // 画像形式
}

// テキストブロック
export interface TextBlock {
  id: string;                // 一意のID
  x: number;                 // X座標
  y: number;                 // Y座標
  width: number;             // 幅
  height: number;            // 高さ
  text: string;              // テキスト内容
  fontSize: number;          // フォントサイズ
  lineHeight: number;         // 行間（倍率）
  fontFamily: string;         // フォントファミリー
}

// PDFメタデータ
export interface PdfMetadata {
  title?: string;
  author?: string;
  subject?: string;
  creator?: string;
  producer?: string;
  creationDate?: string;
  modificationDate?: string;
}
