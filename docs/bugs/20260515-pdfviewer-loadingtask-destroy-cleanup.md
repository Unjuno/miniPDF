# PDFViewer loading task cleanup

- Symptom: `PDFViewer` のアンマウント時に `loadingTask.cancel is not a function` が出る。
- Root cause: 使っている `pdfjs-dist` の `PDFDocumentLoadingTask` には `cancel()` がなく、cleanup で存在しない API を呼んでいた。
- Fix: cleanup を `loadingTask.destroy()` ベースに変更し、destroy 中の例外は `logger.warn` で吸収するようにした。
- Prevention: PDF.js の型定義に合わせて cleanup API を確認し、テストで unmount 時に `destroy()` が呼ばれることを検証する。
