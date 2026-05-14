# リリース手順（GitHub Actions）

## 前提

- リポジトリは **GitHub 上のパブリックリポジトリ**で、`Actions` が有効なこと。
- アプリのライセンスは `src-tauri/Cargo.toml` の **AGPL-3.0** 等に従うこと（配布物に同梱される旨を Release 本文に書くとよいです）。

## バンドル形式（`tauri.conf.json`）

現状の `bundle.targets` は次を想定しています。

| OS | 主な成果物 |
|----|------------|
| Windows | NSIS インストーラー（`.exe`） |
| macOS | `.dmg` |
| Linux | `.deb` と `.AppImage` |

## バージョンを上げる

1. `package.json` の `version` を更新する。  
2. `src-tauri/tauri.conf.json` の `version` を **同じ値**に合わせる。  
3. コミットする。

## リリースビルド（CI / CD）

### タグでビルド + GitHub Release（ドラフト）

```bash
git tag v1.0.1
git push origin v1.0.1
```

- ワークフロー **Release** が Windows / macOS / Linux で `npm run tauri build` を実行します。
- タグが `v` で始まる **push** のときだけ、その後 **ドラフトの GitHub Release** が作成され、`upload/` に集めたインストーラー類が添付されます。公開前に Release 画面で本文と添付を確認してください。

### 手動（workflow_dispatch）

Actions の「Release」ワークフローを手動実行すると、**三 OS 分の成果物は Artifact として**保存されます（タグでない場合は GitHub Release は作られません）。

## アイコン

既定は `scripts/gen_placeholder_icon.py` で生成した単色 PNG を `npx @tauri-apps/cli icon` に渡して作った一式です。製品用に差し替える場合は高解像度のマスター PNG を用意し、リポジトリルートで:

```bash
cd src-tauri
npx @tauri-apps/cli@2 icon path/to/app-icon.png
```

生成物をコミットしてください。

## コード署名（任意）

現状のワークフローは **未署名** です。Windows の SmartScreen や macOS の Gatekeeper 向けには、各 OS の証明書とシークレットを用意し、Tauri の署名オプションを追加する必要があります（別途設計）。

## ローカルでインストーラーを試す

```bash
npm ci
npm run tauri build
```

成果物は `src-tauri/target/release/bundle/` 以下に出力されます。
