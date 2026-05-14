# Cargoプロファイル設定の分析

> **参照**: [Cargo Profiles Documentation](https://doc.rust-lang.org/cargo/reference/profiles.html#default-profiles)

## 現在の設定

**ファイル**: `src-tauri/Cargo.toml`

```toml
[profile.release]
opt-level = "z"  # サイズ最適化（起動時間重視）
lto = true  # リンク時最適化
codegen-units = 1  # コード生成単位を1に（最適化品質向上）
strip = true  # シンボル情報を削除（バイナリサイズ削減）
```

## 設定の詳細分析

### 1. `opt-level = "z"`

**意味**: バイナリサイズを最適化し、ループベクトル化を無効化

**デフォルト**: `3` (すべての最適化)

**現在の設定の効果**:
- ✅ バイナリサイズが最小化される
- ✅ 起動時間が短縮される可能性がある
- ⚠️ 実行時のパフォーマンスが若干低下する可能性がある

**推奨事項**:
- デスクトップアプリケーション（miniPDF）では、起動時間とバイナリサイズが重要
- `"z"`は適切な選択
- ただし、PDF処理のパフォーマンスが問題になる場合は`3`に変更を検討

### 2. `lto = true`

**意味**: リンク時最適化（Link Time Optimization）を有効化

**デフォルト**: `false`

**現在の設定の効果**:
- ✅ 全体プログラム最適化により、より良い最適化が可能
- ✅ バイナリサイズの削減
- ⚠️ コンパイル時間が大幅に増加（特に初回ビルド）

**推奨事項**:
- リリースビルドでは有効化が推奨
- 開発時は`cargo build`（dev profile）を使用するため影響なし
- 現在の設定は適切

### 3. `codegen-units = 1`

**意味**: コード生成単位を1に設定

**デフォルト**: 
- インクリメンタルビルド: `256`
- 非インクリメンタルビルド: `16`

**現在の設定の効果**:
- ✅ 最適化品質が向上（コンパイラがより多くのコードを一度に最適化できる）
- ✅ バイナリサイズの削減
- ⚠️ コンパイル時間が増加
- ⚠️ 並列コンパイルの機会が減少

**推奨事項**:
- リリースビルドでは最適化品質を優先する設定
- 現在の設定は適切
- ただし、コンパイル時間が問題になる場合は`16`に変更を検討

### 4. `strip = true`

**意味**: シンボル情報を削除

**デフォルト**: `"none"`

**現在の設定の効果**:
- ✅ バイナリサイズが削減される
- ✅ デバッグ情報が削除される（リリースビルドでは問題なし）
- ⚠️ デバッグが困難になる（リリースビルドでは通常問題なし）

**推奨事項**:
- リリースビルドでは推奨される設定
- 現在の設定は適切

## デフォルトプロファイルとの比較

### デフォルト `[profile.release]`

```toml
[profile.release]
opt-level = 3
debug = false
split-debuginfo = '...'  # プラットフォーム依存
strip = "none"
debug-assertions = false
overflow-checks = false
lto = false
panic = 'unwind'
incremental = false
codegen-units = 16
rpath = false
```

### 現在の設定との違い

| 設定 | デフォルト | 現在の設定 | 理由 |
|------|-----------|-----------|------|
| `opt-level` | `3` | `"z"` | バイナリサイズと起動時間を優先 |
| `lto` | `false` | `true` | より良い最適化 |
| `codegen-units` | `16` | `1` | 最適化品質の向上 |
| `strip` | `"none"` | `true` | バイナリサイズの削減 |

## 推奨事項

### 現在の設定の評価

✅ **現在の設定は適切です**

理由:
1. **デスクトップアプリケーション**: 起動時間とバイナリサイズが重要
2. **PDF処理**: 軽量な処理が中心で、極端な最適化は不要
3. **リリースビルド**: コンパイル時間よりも実行時のパフォーマンスとサイズを優先

### オプション: 開発プロファイルの最適化

開発時のビルド時間を短縮したい場合:

```toml
[profile.dev]
opt-level = 0  # デフォルト（最適化なし）
incremental = true  # デフォルト（インクリメンタルコンパイル）
codegen-units = 256  # デフォルト（並列コンパイルを最大化）
```

現在はデフォルト設定を使用しているため、追加の設定は不要です。

### オプション: カスタムプロファイル

異なる最適化レベルを試したい場合:

```toml
[profile.release-fast]
inherits = "release"
opt-level = 3  # 実行速度を優先
lto = true
codegen-units = 1

[profile.release-small]
inherits = "release"
opt-level = "z"  # サイズを優先（現在の設定）
lto = true
codegen-units = 1
```

使用例:
```bash
cargo build --profile release-fast
cargo build --profile release-small
```

## パフォーマンス測定の推奨

現在の設定が最適かどうかを確認するために、以下を測定することを推奨:

1. **バイナリサイズ**: `target/release/minipdf.exe`のサイズ
2. **起動時間**: アプリケーションの起動からPDF読み込みまで
3. **実行時パフォーマンス**: PDF処理の速度
4. **コンパイル時間**: リリースビルドの所要時間

## まとめ

現在の`[profile.release]`設定は、デスクトップアプリケーション（miniPDF）の要件に適しています：

- ✅ バイナリサイズの最小化
- ✅ 起動時間の短縮
- ✅ 適切な最適化レベル
- ✅ リリースビルドでの推奨設定

変更は不要ですが、パフォーマンスに問題が発生した場合は`opt-level = 3`への変更を検討してください。

---

**参照**: [Cargo Profiles Documentation](https://doc.rust-lang.org/cargo/reference/profiles.html#default-profiles)

