# miniPDF

> Markdown→PDFで「最後にズレるところだけ」を、Word感覚でサクッと直せる、超軽量PDFレイアウト調整ツール

[![GitHub](https://img.shields.io/github/license/Unjuno/miniPDF)](https://github.com/Unjuno/miniPDF)
[![GitHub](https://img.shields.io/github/stars/Unjuno/miniPDF)](https://github.com/Unjuno/miniPDF)

## 概要

miniPDF は、Markdown → PDF 変換後に発生するレイアウトの「ズレ」を簡単に修正するための軽量デスクトップアプリケーションです。

### 解決する問題

- Mermaid 図がデカすぎる
- 意図しない改ページ・改行
- 行が詰まらず、Word みたいに整わない

### 特徴

- 🚀 **超軽量**：起動が速い（1秒以内）
- 🎯 **特化**：レイアウト調整だけに集中
- 🔍 **検索可能**：テキスト検索機能を維持
- 🆓 **OSS**：完全オープンソース

## 機能

- PDF内画像（Mermaid図など）のサイズ調整
- 改ページ位置の微調整
- 行間・詰まりの調整（Word的見た目）

## 技術スタック

- [Tauri](https://tauri.app/) - 軽量デスクトップアプリケーションフレームワーク
- React + TypeScript
- PDF読み込み・生成ライブラリ

## 開発状況

🚧 **開発中** - 初期セットアップ段階

## ライセンス

MIT License - 詳細は [LICENSE](./LICENSE) を参照してください。

## コントリビューション

コントリビューションを歓迎します！IssueやPull Requestをお気軽に作成してください。

- [GitHub Issues](https://github.com/Unjuno/miniPDF/issues) - バグ報告・機能要望
- [GitHub Pull Requests](https://github.com/Unjuno/miniPDF/pulls) - コード貢献

## ドキュメント

プロジェクトの詳細情報は以下の文書を参照してください：

- **[CONCEPT.md](./CONCEPT.md)** - プロジェクトのコンセプト・設計思想
- **[SPECIFICATION.md](./SPECIFICATION.md)** - 機能仕様・実装方法（**実装時の主要参照文書**）
- **[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)** - 実装計画・技術詳細
- **[TECHNICAL_REFERENCES.md](./TECHNICAL_REFERENCES.md)** - 技術リファレンス（公式ドキュメントURL）

## プロジェクト構造

```
miniPDF/
├── README.md                    # プロジェクト概要（このファイル）
├── CONCEPT.md                   # コンセプト文書
├── SPECIFICATION.md             # 仕様書（機能仕様・実装方法）
├── IMPLEMENTATION_PLAN.md       # 実装計画書
├── TECHNICAL_REFERENCES.md     # 技術リファレンス
└── docs/
    └── INDEX.md                # ドキュメント索引
```

## クイックスタート

### ドキュメントの読み方

- **プロジェクトの概要を知りたい** → [README.md](./README.md)
- **設計思想を理解したい** → [CONCEPT.md](./CONCEPT.md)
- **実装を始める** → [SPECIFICATION.md](./SPECIFICATION.md) **（最重要）**
- **実装計画を確認したい** → [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)
- **技術の公式ドキュメントを参照したい** → [TECHNICAL_REFERENCES.md](./TECHNICAL_REFERENCES.md)

