# フォントディレクトリ

このディレクトリには、PDF生成時に使用するカスタムフォントファイルを配置します。

## サポートされるフォント形式

- TrueType Font (.ttf)
- OpenType Font (.otf)

## 推奨フォント

日本語テキストを正しく表示するために、以下のフォントを推奨します：

- **Noto Sans JP** - Google Noto Fonts（日本語対応）
  - `NotoSansJP-Regular.ttf`
  - `NotoSansJP-Bold.ttf`
- **Noto Serif JP** - Google Noto Fonts（日本語対応、セリフ体）
  - `NotoSerifJP-Regular.ttf`

## フォントの取得方法

1. [Google Noto Fonts](https://fonts.google.com/noto/specimen/Noto+Sans+JP)からダウンロード
2. フォントファイルをこのディレクトリに配置
3. アプリケーションを再起動

## フォント名のマッピング

フォントファイル名とフォント名のマッピング：

- `NotoSansJP-Regular.ttf` → `NotoSansJP`
- `NotoSansJP-Bold.ttf` → `NotoSansJP-Bold`
- `NotoSerifJP-Regular.ttf` → `NotoSerifJP`

## 注意事項

- フォントファイルはライセンスを確認して使用してください
- フォントファイルのサイズが大きい場合、アプリケーションのサイズが増加します
- リリースでは実行ファイルと同じディレクトリの `fonts/` に配置されます
- **開発中**は `src-tauri/fonts/`（Cargo マニフェスト直下の `fonts/`）に置いたファイルも自動で読み込みます（`NotoSansJP-Regular.ttf` など）
- **Windows**: 上記が無い場合、`%WINDIR%\Fonts\NotoSansJP-VF.ttf` など OS 同梱の **単一 `.ttf`** を `NotoSansJP` として自動登録します（`.ttc` は読み込めない場合があります）

