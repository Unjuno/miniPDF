# miniPDF - やらないこと制約リスト

## 1. 概要

この文書は、miniPDFプロジェクトで意図的に実装しない機能や制約事項を明確化するものです。開発方針の一貫性を保ち、スコープクリープを防ぐことを目的としています。

## 2. 機能制約（やらない機能）

### 2.1 コア機能外の機能

以下の機能は、プロジェクトの設計思想により意図的に実装しません：

- **PDF 画面上でのレイアウト編集（ブロック移動・画像リサイズ・改ページドラッグ等）**
  - 理由: 製品は Markdown 編集 + プレビュー専用。インタラクティブな PDF レイアウト UI は提供しない
  - 参照: [FEATURES.md](./FEATURES.md#レイアウト編集について非対応)

- **PDFの完全な再構造化**
  - 理由: PDF内部構造の完全編集は行わない（Acrobat路線は捨てる）
  - 参照: [CONCEPT.md](./CONCEPT.md)（「PDF をプレビュー成果物として扱う」）

- **署名・フォーム・高度注釈機能**
  - 理由: インタラクティブなレイアウト調整ツールではない
  - 参照: [CONCEPT.md](./CONCEPT.md)

- **複雑なPDF構造の完全対応**
  - 理由: フォーム、署名など複雑なPDF構造は完全には対応していない場合がある
  - 参照: [FEATURES.md](./FEATURES.md#pdfファイルの読み込み)

### 2.2 将来実装予定の機能（MVPでは未実装）

以下の機能は仕様書で「将来実装」と明記されており、現時点では未実装です：

- **アンドゥ・リドゥ機能**
  - 状態: 将来実装予定
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#26-アンドゥリドゥ将来実装)

- **画像位置移動**
  - 状態: 将来実装予定（現在はサイズ調整のみ対応）
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#233-画像位置の調整将来実装)

- **複数ページ表示**
  - 状態: プレビューでは連続表示を既定にしている（仕様書の「将来実装」記述は旧稿）
  - 参照: [FEATURES.md](./FEATURES.md#pdf-プレビュー表示)

- **メニューバー（ファイル、編集、表示、ヘルプ）**
  - 状態: 未実装（機能はツールバーで実現されている）
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#32-メニューバー), [IMPLEMENTATION_VERIFICATION.md](./IMPLEMENTATION_VERIFICATION.md#131-未実装機能仕様書に記載)

- **ステータスバー**
  - 状態: 一部実装（ページ番号表示はPDFViewer内、ズーム倍率はヘッダーに表示）
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#31-ウィンドウレイアウト), [IMPLEMENTATION_VERIFICATION.md](./IMPLEMENTATION_VERIFICATION.md#131-未実装機能仕様書に記載)

- **一部キーボードショートカット**
  - `Ctrl+Shift+S` (名前を付けて保存) - 未実装（現在は`Ctrl+S`で保存ダイアログを開く）
  - `Ctrl+F` (フィット表示) - 未実装
  - `Alt+F4` (終了) - OS標準機能
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#35-キーボードショートカット), [IMPLEMENTATION_VERIFICATION.md](./IMPLEMENTATION_VERIFICATION.md#131-未実装機能仕様書に記載)

### 2.3 操作制約

以下の制約は、機能の安全性と一貫性を保つために実装されています：

#### 画像操作の制約

- **画像の最小サイズ制限**: 元のサイズの10%以上
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#323-画像のリサイズ)

- **画像の最大サイズ制限**: ページサイズの200%以下
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#323-画像のリサイズ)

- **画像がページ外に出ない制限**: 画像はページ範囲内に収まる必要がある
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#323-画像のリサイズ)

#### 改ページ操作の制約

- **改ページ位置の移動範囲制限**: 前後のページ境界内
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#243-改ページ位置の調整)

- **最小ページ高さ制限**: ページサイズの10%以上
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#243-改ページ位置の調整)

- **ページ内コンテンツの制約**: ページ内のコンテンツがページ外に出ないように制限
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#243-改ページ位置の調整)

#### ページ操作の制約

- **最後の1ページ削除不可**: PDFには最低1ページ必要
  - 参照: [FEATURES.md](./FEATURES.md#ページの削除), [SPECIFICATION.md](./SPECIFICATION.md#12-制約事項)

#### ファイル操作の制約

- **ファイルサイズ制限**: 100MB以上のファイルは警告が表示される場合がある
  - 参照: [FEATURES.md](./FEATURES.md#pdfファイルの読み込み)

## 3. 設計方針による制約

### 3.1 PDF編集アプローチ

- **PDF内部構造の完全編集は行わない**
  - 理由: Acrobat路線は捨てる。視覚的配置の再配置と再レンダリングによる新PDF生成のみ
  - 参照: [CONCEPT.md](./CONCEPT.md#pdfを最終成果物として扱う)

- **視覚的配置の再配置と再レンダリングによる新PDF生成のみ**
  - 理由: 軽量性とシンプルさを維持するため
  - 参照: [CONCEPT.md](./CONCEPT.md#技術的割り切り)

### 3.2 技術スタックの制約

- **PDFライブラリの制約**: lopdfの機能範囲内で実装
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#12-制約事項)（12.1 技術的制約セクション参照）

- **メモリ制約**: 大きなPDFファイルは段階的読み込み（将来実装予定）
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#12-制約事項)（12.1 技術的制約セクション参照）

- **パフォーマンス制約**: リアルタイム処理の最適化が必要
  - 参照: [SPECIFICATION.md](./SPECIFICATION.md#12-制約事項)（12.1 技術的制約セクション参照）

## 4. 開発プロセス制約

### 4.1 エージェント開発ルール

以下の制約は、[AGENTS.md](../AGENTS.md)で定義されています：

- **推測的変更の禁止**: 明示的な要求またはテスト要件がない限り、機能を追加しない
- **変更の正当化**: すべての変更は以下のいずれかで正当化される必要がある
  - 失敗したテストの修正 / 再現可能なバグの修正
  - 証明された欠陥の排除（静的解析 / 明らかなランタイムエラー）
  - 動作を同一に保ちながら複雑さを削減
- **検証の必須**: すべての変更には検証が必要（テストの実行または追加）
- **小さな差分**: 1回の実行につき1つの論理的な変更セットのみ

## 5. 参照

### 関連ドキュメント

- [CONCEPT.md](./CONCEPT.md) - プロジェクトのコンセプト・設計思想
- [SPECIFICATION.md](./SPECIFICATION.md) - 詳細な機能仕様・実装方法
- [FEATURES.md](./FEATURES.md) - 機能の詳細と操作方法
- [IMPLEMENTATION_VERIFICATION.md](./IMPLEMENTATION_VERIFICATION.md) - 実装検証レポート
- [AGENTS.md](../AGENTS.md) - エージェント開発ルール

### ドキュメント索引

- [INDEX.md](./INDEX.md) - ドキュメント索引（ドキュメントの読み方）

