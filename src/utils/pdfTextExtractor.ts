// why: PDF.jsを使用して正確なテキスト抽出を実現（lopdfでは日本語が文字化けする）
// alt: lopdfのextract_text（日本語や複雑なPDFで正しく動作しない）
// evidence: PDF.jsは日本語にも対応しており、位置情報も正確に取得できる
import type { TextBlock } from '../types/pdf';

export interface TextItem {
  str: string;
  dir: string;
  width: number;
  height: number;
  transform: number[];
  fontName: string;
}

// why: PDF.jsの型定義を使用して型安全性を向上
// alt: any型を使用（型安全性が低下）
// evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
export async function extractTextWithPDFjs(
  pdfjs: typeof import('pdfjs-dist'), // PDF.jsライブラリの型
  pdf: import('pdfjs-dist').PDFDocumentProxy, // PDF.jsのPDFDocument型
  pageNumber: number,
  pageWidth: number,
  pageHeight: number,
  images: Array<{ x: number; y: number; width: number; height: number }> = []
): Promise<TextBlock[]> {
  const page = await pdf.getPage(pageNumber);
  const textContent = await page.getTextContent({
    normalizeWhitespace: false,
    disableCombineTextItems: false,
  });

  const textBlocks: TextBlock[] = [];
  const items = textContent.items as TextItem[];

  if (items.length === 0) {
    return textBlocks;
  }

  // why: テキストアイテムをグループ化してブロックを作成（連続するテキストを1つのブロックに）
  // alt: 各アイテムを個別のブロックに（ブロック数が多すぎる）
  // evidence: グループ化により、編集しやすいブロック構造を実現
  let currentBlock: {
    text: string;
    items: TextItem[];
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
    fontSize: number;
    fontFamily: string;
  } | null = null;

  const LINE_HEIGHT_THRESHOLD = 5; // 同じ行とみなすY座標の差（ポイント）
  const X_DISTANCE_THRESHOLD = 50; // 同じブロックとみなすX座標の距離（ポイント）

  for (const item of items) {
    if (!item.str || item.str.trim().length === 0) {
      continue;
    }

    // transform配列から位置とサイズを取得
    // transform: [a, b, c, d, e, f] は変換行列
    // e, f がX, Y座標（左下が原点）
    const x = item.transform[4] || 0;
    const y = item.transform[5] || 0;
    const width = item.width || 0;
    const height = item.height || 0;
    const fontSize = height || 12;
    const fontFamily = item.fontName || 'Arial';

    // PDF座標系は左下が原点なので、Y座標を反転
    const pdfY = pageHeight - y - height;

    // why: 画像領域内のテキストを除外（Mermaid図などは画像として扱う）
    // alt: 画像領域内のテキストも抽出（画像の縮尺変更ができない）
    // evidence: 画像領域内のテキストを除外することで、画像として扱えるようになる
    const isInsideImage = images.some(image => {
      // テキストアイテムの中心点が画像領域内にあるかチェック
      const textCenterX = x + width / 2;
      const textCenterY = pdfY + height / 2;
      return (
        textCenterX >= image.x &&
        textCenterX <= image.x + image.width &&
        textCenterY >= image.y &&
        textCenterY <= image.y + image.height
      );
    });

    if (isInsideImage) {
      // 画像領域内のテキストは除外
      continue;
    }

    // 現在のブロックがある場合、同じ行かチェック
    if (currentBlock) {
      const isSameLine = Math.abs(currentBlock.minY - pdfY) < LINE_HEIGHT_THRESHOLD;
      const isNearX = Math.abs(x - currentBlock.maxX) < X_DISTANCE_THRESHOLD;

      if (isSameLine && isNearX) {
        // 同じブロックに追加
        currentBlock.text += item.str;
        currentBlock.items.push(item);
        currentBlock.maxX = Math.max(currentBlock.maxX, x + width);
        currentBlock.minX = Math.min(currentBlock.minX, x);
        currentBlock.minY = Math.min(currentBlock.minY, pdfY);
        currentBlock.maxY = Math.max(currentBlock.maxY, pdfY + height);
        continue;
      } else {
        // 新しいブロックを作成（前のブロックを保存）
        const blockWidth = currentBlock.maxX - currentBlock.minX;
        const blockHeight = currentBlock.maxY - currentBlock.minY;
        textBlocks.push({
          id: `text_${pageNumber}_${textBlocks.length}`,
          x: currentBlock.minX,
          y: pageHeight - currentBlock.maxY, // PDF座標系に変換
          width: blockWidth,
          height: blockHeight,
          text: currentBlock.text.trim(),
          fontSize: currentBlock.fontSize,
          lineHeight: 1.2,
          fontFamily: currentBlock.fontFamily,
        });
      }
    }

    // 新しいブロックを開始
    currentBlock = {
      text: item.str,
      items: [item],
      minX: x,
      minY: pdfY,
      maxX: x + width,
      maxY: pdfY + height,
      fontSize,
      fontFamily,
    };
  }

  // 最後のブロックを保存
  if (currentBlock) {
    const blockWidth = currentBlock.maxX - currentBlock.minX;
    const blockHeight = currentBlock.maxY - currentBlock.minY;
    textBlocks.push({
      id: `text_${pageNumber}_${textBlocks.length}`,
      x: currentBlock.minX,
      y: pageHeight - currentBlock.maxY, // PDF座標系に変換
      width: blockWidth,
      height: blockHeight,
      text: currentBlock.text.trim(),
      fontSize: currentBlock.fontSize,
      lineHeight: 1.2,
      fontFamily: currentBlock.fontFamily,
    });
  }

  return textBlocks;
}

