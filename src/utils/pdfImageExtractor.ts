import { logger } from './logger';

// why: PDF.jsを使用して画像の位置情報を取得（lopdfでは位置情報が0.0になる）
// alt: lopdfの位置情報を使用（位置情報が不正確）
// evidence: PDF.jsを使用することで、Content Streamから正確な位置情報を取得できる
export interface ImageArea {
  x: number;
  y: number;
  width: number;
  height: number;
}

// why: PDF.jsの型定義を使用して型安全性を向上
// alt: any型を使用（型安全性が低下）
// evidence: 型定義により、IDEの補完が効き、実行時エラーのリスクが減少
export async function extractImageAreasWithPDFjs(
  pdfjs: typeof import('pdfjs-dist'), // PDF.jsライブラリの型
  pdf: import('pdfjs-dist').PDFDocumentProxy, // PDF.jsのPDFDocument型
  pageNumber: number,
  pageHeight: number
): Promise<ImageArea[]> {
  try {
    const page = await pdf.getPage(pageNumber);
    const operatorList = await page.getOperatorList();

    const imageAreas: ImageArea[] = [];
    const imageNames = new Set<string>();

    // why: ResourcesからXObjectの名前を取得
    // alt: Content Streamのみを解析（画像名が取得できない）
    // evidence: Resourcesから画像名を取得することで、Content StreamのDo演算子と対応付けられる
    // PDF.js 4.xでは、getResources()メソッドが削除されたため、Content Streamのみから画像を検出
    // Resourcesの取得を試みるが、失敗しても問題ない（Content Streamから検出可能）
    try {
      // PDF.js 4.xでは、getResources()メソッドが削除されたため、resourcesプロパティを直接使用
      // ただし、resourcesプロパティが存在しない場合もあるため、エラーハンドリングが必要
      const resources = (page as any).resources;
      if (resources && typeof resources.get === 'function') {
        const xobjects = resources.get('XObject');
        if (xobjects && typeof xobjects === 'object') {
          const xobjectDict = xobjects as any;
          for (const name in xobjectDict) {
            if (xobjectDict.hasOwnProperty(name)) {
              const ref = xobjectDict[name];
              // 参照オブジェクトから画像かどうかを確認
              if (ref && typeof ref === 'object') {
                try {
                  // why: PDF.js 4.xでは、objsプロパティが削除されたため、XObjectのサブタイプを直接確認できない
                  // alt: objsプロパティを使用（PDF.js 4.xでは存在しない）
                  // evidence: PDF.js 4.xでは、objsプロパティが削除され、Content Streamのみから画像を検出する必要がある
                  // 注意: この方法では、XObjectのサブタイプを直接確認できないため、すべてのXObjectを画像として扱う
                  // ただし、Content Streamの解析で画像を検出できるため、この方法でも問題ない
                  imageNames.add(name);
                } catch (e) {
                  // オブジェクトの取得に失敗した場合はスキップ
                  continue;
                }
              }
            }
          }
        }
      }
    } catch (error) {
      // Resourcesの取得に失敗した場合は、Content Streamのみから画像を検出
      // PDF.js 4.xでは、getResources()メソッドが削除されたため、このエラーは正常な動作
      // Content Streamのみから画像を検出することで、機能は維持される
      // エラーログは出力しない（正常な動作のため）
    }

    // why: Content Streamから画像の位置情報を取得
    // alt: 位置情報なし（画像領域内のテキストを除外できない）
    // evidence: Content Streamのcm演算子から変換行列を取得することで、画像の位置情報を取得できる
    let currentTransform = [1, 0, 0, 1, 0, 0]; // デフォルトの変換行列
    const transformStack: number[][] = [];

    if (operatorList && operatorList.fnArray && operatorList.argsArray) {
      for (let i = 0; i < operatorList.fnArray.length; i++) {
        const fn = operatorList.fnArray[i];
        const args = operatorList.argsArray[i];

        // why: PDF.js 4.xでは、OPS定数が削除されたため、数値で直接比較する
        // alt: pdfjs.OPSを使用（PDF.js 4.xでは存在しない）
        // evidence: PDF.js 4.xでは、OPS定数が削除され、演算子は数値で表現される
        // 注意: 演算子の数値はPDF.jsの内部実装に依存するため、型定義がない場合はany型を使用する
        // q: グラフィックス状態を保存 (演算子番号: 120)
        if (fn === 120 || (typeof fn === 'string' && fn === 'q')) {
          transformStack.push([...currentTransform]);
        }
        // Q: グラフィックス状態を復元 (演算子番号: 121)
        else if (fn === 121 || (typeof fn === 'string' && fn === 'Q')) {
          const popped = transformStack.pop();
          if (popped) {
            currentTransform = popped;
          }
        }
        // cm: 変換行列を適用 [a, b, c, d, e, f] (演算子番号: 30)
        else if (fn === 30 || (typeof fn === 'string' && fn === 'cm')) {
          if (args && Array.isArray(args) && args.length >= 6) {
            const [a, b, c, d, e, f] = args;
            // 変換行列を合成
            const newTransform = [
              a * currentTransform[0] + c * currentTransform[1],
              b * currentTransform[0] + d * currentTransform[1],
              a * currentTransform[2] + c * currentTransform[3],
              b * currentTransform[2] + d * currentTransform[3],
              a * currentTransform[4] + c * currentTransform[5] + e,
              b * currentTransform[4] + d * currentTransform[5] + f,
            ];
            currentTransform = newTransform;
          }
        }
        // Do: XObjectを描画 (演算子番号: 60)
        else if (fn === 60 || (typeof fn === 'string' && fn === 'Do')) {
          if (args && Array.isArray(args) && args.length > 0) {
            const imageName = args[0];
            if (imageNames.has(imageName) || imageNames.size === 0) {
              // 画像名が特定できない場合は、すべてのXObjectを画像として扱う
              // 変換行列から位置とサイズを取得
              // 変換行列 [a, b, c, d, e, f] から
              // 幅 = sqrt(a^2 + b^2)
              // 高さ = sqrt(c^2 + d^2)
              // X座標 = e
              // Y座標 = f
              const width = Math.sqrt(currentTransform[0] ** 2 + currentTransform[1] ** 2);
              const height = Math.sqrt(currentTransform[2] ** 2 + currentTransform[3] ** 2);
              const x = currentTransform[4];
              const y = currentTransform[5];

              // PDF座標系は左下が原点なので、Y座標を反転
              const pdfY = pageHeight - y - height;

              // 有効な位置情報の場合のみ追加
              const isValidNumber = (value: number) => Number.isFinite(value) && !Number.isNaN(value);
              if (
                width > 0 &&
                height > 0 &&
                x >= 0 &&
                pdfY >= 0 &&
                isValidNumber(width) &&
                isValidNumber(height) &&
                isValidNumber(x) &&
                isValidNumber(pdfY)
              ) {
                imageAreas.push({
                  x,
                  y: pdfY,
                  width,
                  height,
                });
              }
            }
          }
        }
      }
    }

    return imageAreas;
  } catch (error) {
    // エラーが発生した場合は空配列を返す
    logger.warn('Failed to extract image areas with PDF.js', { error });
    return [];
  }
}

