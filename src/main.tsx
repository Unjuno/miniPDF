import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
// why: CSSの読み込みを最適化するため、クリティカルCSSのみ先読み
// alt: すべてのCSSを先読み
// evidence: 大きなCSSファイルはレンダリングブロックを引き起こす
// assumption: 非クリティカルCSSは必要時に読み込まれる
import './index.css';
import { logger } from './utils/logger';

// why: 本番ビルドでの起動時間を最適化するため、StrictModeを条件付きにする
// alt: 常にStrictModeを有効にする（開発時の警告が失われる）
// evidence: StrictModeは開発時の二重レンダリングを引き起こし、起動時間に影響する
// assumption: 開発時はNODE_ENV=developmentで実行される
// why: root要素の存在を確認してからレンダリングすることで、実行時エラーを防ぐ
// alt: 非nullアサーション演算子を使用（root要素が存在しない場合にエラーが発生する）
// evidence: root要素の存在確認により、より明確なエラーメッセージを提供できる
const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found. Please ensure the HTML file contains a <div id="root"></div> element.');
}

// why: グローバルエラーハンドラーを追加して、未処理のエラーをキャッチし、ログに記録する
// alt: グローバルエラーハンドラーなし（未処理のエラーがユーザーに通知されない）
// evidence: グローバルエラーハンドラーにより、予期しないエラーをキャッチし、デバッグを容易にできる
// why: globalThis.windowの存在を確認してからイベントリスナーを追加することで、SSR環境でもエラーを防ぐ
// alt: チェックなし（SSR環境でエラーが発生する可能性がある）
// evidence: typeofチェックにより、ブラウザ環境でのみイベントリスナーを追加できる
if (globalThis?.window) {
  globalThis.window.addEventListener('error', (event: ErrorEvent) => {
    const errorObj = event.error instanceof Error ? event.error : new Error(event.message);
    logger.error('Unhandled error', errorObj, { 
      filename: event.filename, 
      lineno: event.lineno, 
      colno: event.colno,
    });
  });

  globalThis.window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
    const errorObj = event.reason instanceof Error ? event.reason : new Error(String(event.reason));
    logger.error('Unhandled promise rejection', errorObj);
    // why: デフォルトの動作を防ぐことで、エラーがコンソールに表示されないようにする
    // alt: デフォルトの動作を許可（エラーがコンソールに表示される）
    // evidence: ログに記録した後、デフォルトの動作を防ぐことで、エラーの重複表示を防ぐ
    event.preventDefault();
  });
}

const root = ReactDOM.createRoot(rootElement);

if (import.meta.env?.DEV || import.meta.env?.MODE === 'development') {
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
} else {
  root.render(<App />);
}
