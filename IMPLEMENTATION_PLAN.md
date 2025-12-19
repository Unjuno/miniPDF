# miniPDF - 実装計画書

## 1. 技術スタック詳細

### 1.1 フロントエンド

- **フレームワーク**: React 18+ (TypeScript)
- **ビルドツール**: Vite
- **UIライブラリ**: 
  - 軽量なUIコンポーネント（必要に応じて選択）
  - ドラッグ&ドロップ: react-draggable または カスタム実装
- **PDF表示**: 
  - PDF.js (Mozilla) - PDF読み込み・表示用
  - または react-pdf ライブラリ
- **スタイリング**: 
  - CSS Modules または Tailwind CSS（軽量重視）
  - 必要最小限のスタイル

### 1.2 バックエンド（Tauri）

- **フレームワーク**: Tauri 2.x
- **言語**: Rust
- **PDF操作ライブラリ**:
  - `pdf` crate - PDF読み込み・基本操作
  - `lopdf` - PDF構造解析
  - または `pdfium-render` - PDFレンダリング
- **画像処理**:
  - `image` crate - 画像リサイズ・変換
- **ファイル操作**:
  - Tauri API - ファイル選択・保存

### 1.3 開発環境

- **Node.js**: 18+
- **Rust**: 最新安定版
- **パッケージマネージャ**: npm
- **開発ツール**:
  - ESLint / TypeScript
  - rustfmt / clippy

## 2. アーキテクチャ設計

### 2.1 全体構成

```
┌─────────────────────────────────────┐
│         React UI (Frontend)         │
│  - PDF表示コンポーネント             │
│  - ドラッグ操作UI                    │
│  - ツールバー                        │
└──────────────┬──────────────────────┘
               │ IPC通信
┌──────────────▼──────────────────────┐
│      Tauri Backend (Rust)           │
│  - PDF読み込み                       │
│  - PDF構造解析                       │
│  - レイアウト調整処理                │
│  - PDF再生成                         │
└─────────────────────────────────────┘
```

### 2.2 データフロー

1. **PDF読み込み**
   - ユーザーがPDFファイルを選択
   - Tauri Backend がPDFを読み込み
   - PDF構造（ページ、画像、テキスト）を解析
   - Frontend に構造データを送信

2. **表示**
   - Frontend がPDF.js等でPDFをレンダリング
   - 編集可能な要素（画像、改ページ位置）を識別
   - WYSIWYG表示

3. **編集操作**
   - ユーザーがドラッグ操作で調整
   - Frontend が変更内容を記録
   - リアルタイムプレビュー更新

4. **PDF再生成**
   - 編集完了後、変更内容をBackendに送信
   - Backend がPDF構造を再構築
   - 新しいPDFファイルを生成

### 2.3 コアモジュール

#### Frontend モジュール

- **PDFViewer**: PDF表示コンポーネント
- **ImageResizer**: 画像（Mermaid図）のサイズ調整UI
- **PageBreakEditor**: 改ページ位置調整UI
- **LineSpacingEditor**: 行間調整UI
- **Toolbar**: ファイル操作・保存ボタン
- **StateManager**: 編集状態管理（Redux/Zustand等）

#### Backend モジュール

- **PdfReader**: PDF読み込み・解析
- **PdfStructure**: PDF構造データ構造体
- **LayoutAdjuster**: レイアウト調整ロジック
- **PdfGenerator**: PDF再生成
- **ImageProcessor**: 画像リサイズ処理

## 3. 実装フェーズ

### フェーズ1: 基盤構築

#### 1.1 プロジェクトセットアップ
- Tauri + React + TypeScript プロジェクト初期化
- 開発環境構築
- 基本的なディレクトリ構造作成
- ビルド・実行確認

#### 1.2 基本UI構築
- メインウィンドウレイアウト
- ファイル選択ダイアログ
- ツールバー基本UI
- エラーハンドリング表示

### フェーズ2: PDF読み込み・表示

#### 2.1 PDF読み込み機能
- Tauri Backend: PDFファイル読み込み
- PDF構造解析（ページ数、画像位置、テキスト位置）
- IPC通信でFrontendにデータ送信

#### 2.2 PDF表示機能
- Frontend: PDF.js または react-pdf で表示
- ページ単位での表示
- ズーム機能（基本）

#### 2.3 データ構造定義
- PDF構造を表現するTypeScript型定義
- Rust側の構造体定義
- IPC通信の型安全性確保

### フェーズ3: 画像サイズ調整

#### 3.1 画像検出・表示
- PDF内の画像（Mermaid図）を識別
- 画像位置・サイズ情報の取得
- 画像を編集可能な要素として表示

#### 3.2 ドラッグ操作UI
- 画像の選択機能
- リサイズハンドルの表示
- ドラッグによるサイズ変更
- アスペクト比維持オプション

#### 3.3 画像リサイズ処理
- Backend: 画像リサイズ処理
- 変更内容の記録
- プレビュー更新

### フェーズ4: 改ページ位置調整

#### 4.1 改ページ検出
- PDF内の改ページ位置を識別
- ページ境界の可視化

#### 4.2 改ページ調整UI
- 改ページ位置のドラッグ操作
- 前後のコンテンツを再配置
- 自動詰め込み機能

#### 4.3 改ページ処理ロジック
- Backend: コンテンツの再配置
- オーバーフロー処理
- ページ分割・結合

### フェーズ5: 行間調整

#### 5.1 テキストブロック識別
- PDF内のテキストブロックを識別
- 行単位での情報取得

#### 5.2 行間調整UI
- 行間調整スライダー
- 段落単位での調整
- 全体一括調整オプション

#### 5.3 行間処理ロジック
- Backend: テキスト位置の再計算
- 行間変更の適用
- ページ内収まり確認

### フェーズ6: PDF再生成

#### 6.1 変更内容の統合
- 全編集内容の統合
- 変更履歴の管理

#### 6.2 PDF再構築
- Backend: 新しいPDF構造の構築
- 画像・テキスト・ページの再配置
- メタデータの保持

#### 6.3 ファイル保存
- 保存先選択ダイアログ
- PDFファイル生成
- エラーハンドリング

### フェーズ7: 最適化・UX改善

#### 7.1 パフォーマンス最適化
- 大きなPDFファイルの処理最適化
- レンダリング速度向上
- メモリ使用量削減

#### 7.2 UX改善
- 操作の直感性向上
- キーボードショートカット
- アンドゥ・リドゥ機能
- 操作ガイド・ツールチップ

#### 7.3 エラーハンドリング強化
- 詳細なエラーメッセージ
- リカバリー機能
- ログ機能

## 4. 技術的詳細設計

### 4.1 PDF構造データモデル

```typescript
// TypeScript型定義（概念）
interface PdfStructure {
  pages: Page[];
  metadata: PdfMetadata;
}

interface Page {
  pageNumber: number;
  width: number;
  height: number;
  images: ImageElement[];
  textBlocks: TextBlock[];
  pageBreaks: PageBreak[];
}

interface ImageElement {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  data: Uint8Array; // 画像データ
  originalSize: { width: number; height: number };
}

interface TextBlock {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  text: string;
  fontSize: number;
  lineHeight: number;
}

interface PageBreak {
  id: string;
  position: number; // Y座標
  pageNumber: number;
}
```

### 4.2 IPC通信仕様

#### コマンド一覧

- `load_pdf`: PDFファイル読み込み
- `get_pdf_structure`: PDF構造取得
- `resize_image`: 画像リサイズ
- `adjust_page_break`: 改ページ調整
- `adjust_line_spacing`: 行間調整
- `generate_pdf`: PDF再生成
- `save_pdf`: PDF保存

### 4.3 レイアウト調整アルゴリズム

#### 画像リサイズ
1. ユーザーが画像サイズを変更
2. 新しいサイズを記録
3. 周囲の要素との衝突検出
4. 必要に応じて周囲要素を再配置
5. プレビュー更新

#### 改ページ調整
1. 改ページ位置を移動
2. 前後のコンテンツを再計算
3. オーバーフロー検出
4. 必要に応じてページ分割
5. 全体の再レイアウト

#### 行間調整
1. 行間値を変更
2. テキストブロック内の各行の位置を再計算
3. ブロック全体の高さを更新
4. ページ内収まり確認
5. 必要に応じて改ページ調整

## 5. 使用ライブラリ候補

### Frontend

- **react-pdf** または **pdfjs-dist**: PDF表示
- **react-draggable**: ドラッグ操作
- **zustand** または **jotai**: 状態管理（軽量）
- **react-hotkeys-hook**: キーボードショートカット

### Backend (Rust)

- **tauri**: デスクトップアプリフレームワーク
- **pdf**: PDF読み込み・操作
- **image**: 画像処理
- **serde**: シリアライゼーション
- **anyhow**: エラーハンドリング

## 6. テスト方針

### 6.1 単体テスト

- **Frontend**: React Testing Library
- **Backend**: Rust標準テストフレームワーク
- 各モジュールの独立テスト

### 6.2 統合テスト

- IPC通信のテスト
- PDF読み込み・生成のテスト
- エンドツーエンドの操作フロー

### 6.3 テストデータ

- 様々なサイズのPDFファイル
- Mermaid図を含むPDF
- 複数ページのPDF
- エッジケース（空ページ、巨大画像等）

## 7. パフォーマンス目標

- **起動時間**: 1秒以内
- **PDF読み込み**: 10MB以下で1秒以内
- **操作応答性**: ドラッグ操作は60fps
- **メモリ使用量**: 100MB以下（小規模PDF）
- **ファイルサイズ**: 実行ファイル50MB以下

## 8. セキュリティ考慮事項

- ファイル読み込み時の検証
- 悪意のあるPDFファイルの処理
- メモリリーク対策
- サンドボックス環境の活用（Tauri）

## 9. 開発マイルストーン

### マイルストーン1: MVP（最小実用版）
- PDF読み込み・表示
- 画像サイズ調整
- PDF再生成・保存

### マイルストーン2: 基本機能完成
- 改ページ位置調整
- 行間調整
- 基本的なUX改善

### マイルストーン3: リリース準備
- パフォーマンス最適化
- エラーハンドリング強化
- ドキュメント整備
- 初回リリース

## 10. 今後の拡張可能性

- 複数PDFの一括処理
- プリセット機能（よく使う調整を保存）
- プラグインシステム
- コマンドラインインターフェース
- バッチ処理機能

## 11. リスクと対策

### リスク1: PDFライブラリの制約
- **対策**: 複数のライブラリを検証し、最適なものを選択

### リスク2: 大きなPDFファイルの処理速度
- **対策**: 段階的読み込み、非同期処理、最適化

### リスク3: 複雑なPDF構造への対応
- **対策**: 基本的な構造に特化、段階的な機能拡張

### リスク4: テキスト検索可能性の維持
- **対策**: PDF再生成時にテキストレイヤーを保持

## 12. 開発環境・ツール

- **バージョン管理**: Git
- **Issue管理**: GitHub Issues
- **CI/CD**: GitHub Actions（将来的に）
- **ドキュメント**: Markdown
- **コードレビュー**: GitHub Pull Requests

