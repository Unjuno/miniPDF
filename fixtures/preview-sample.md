# miniPDF プレビュー用サンプル

このファイルは **PDF プレビュー**の動作確認用です。アプリで「MDを開く」から読み込むか、内容をコピーしてエディタに貼り付けてください。

## 見出しと段落

これは通常の段落です。編集画面で単一改行した行は、プレビュー PDF でも改行として表示されます。

### 小見出し

**太字**、*斜体*、`インラインコード` を含む文です。

## リスト

- 箇条書きの一行目
- 箇条書きの二行目
  - ネストした項目

1. 番号付きリスト
2. 二番目
3. 三番目

## コードブロック

```typescript
function greet(name: string): string {
  return `Hello, ${name}!`;
}
```

## Mermaid（図）

Mermaid CLI（`mmdc`）が使えると図としてレンダリングされます。`MINIPDF_MERMAID_CLI` に絶対パスを指定するか、アプリ実行ファイルと同じフォルダに `mmdc` を置くか、PATH に入れてください。無い場合はコードブロックのまま表示されます。

```mermaid
flowchart LR
  A[Markdown] --> B[プレビュー]
  B --> C[PDF保存]
```

## 引用

> 引用ブロックの例です。  
> 複数行にまたがることもあります。

---

## 表

| 列A | 列B |
|-----|-----|
| 1   | alpha |
| 2   | beta |

## 長めの本文（改ページの目安用）

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
