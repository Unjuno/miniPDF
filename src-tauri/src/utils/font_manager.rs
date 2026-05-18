/// フォント管理ユーティリティ
/// why: カスタムフォントを管理し、日本語などのマルチバイト文字をサポートする
/// alt: 標準フォントのみを使用（日本語が文字化けする）
/// evidence: oxidize-pdfはカスタムフォントの埋め込みをサポートしている
/// assumption: フォントファイルはsrc-tauri/fonts/ディレクトリに配置される
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use log::{info, warn};
use oxidize_pdf::Document;

/// フォントレジストリ（シングルトン）
static FONT_REGISTRY: OnceLock<FontRegistry> = OnceLock::new();

/// フォントレジストリ
struct FontRegistry {
    fonts: HashMap<String, PathBuf>,
    font_dir: PathBuf,
}

impl FontRegistry {
    fn new() -> Self {
        // why: 実行ファイルのディレクトリから相対パスでフォントディレクトリを取得
        // alt: 絶対パスを使用（デプロイ時に問題が発生する）
        // evidence: 実行ファイルのディレクトリから相対パスで取得することで、デプロイ先でも動作する
        let font_dir = if let Ok(exe_dir) = std::env::current_exe() {
            exe_dir
                .parent()
                .map(|p| p.join("fonts"))
                .unwrap_or_else(|| PathBuf::from("fonts"))
        } else {
            PathBuf::from("fonts")
        };

        Self {
            fonts: HashMap::new(),
            font_dir,
        }
    }

    /// フォントを登録
    fn register_font(&mut self, name: String, font_path: PathBuf) {
        self.fonts.insert(name, font_path);
    }

    /// フォントパスを取得
    fn get_font_path(&self, name: &str) -> Option<&PathBuf> {
        self.fonts.get(name)
    }

    /// フォントディレクトリを取得
    #[allow(dead_code)]
    fn font_dir(&self) -> &Path {
        &self.font_dir
    }

    /// 利用可能なフォントを初期化
    fn initialize_default_fonts(&mut self) {
        // why: デフォルトの日本語フォントを登録（Noto Sans JPなど）
        // alt: フォントを登録しない（日本語が文字化けする）
        // evidence: デフォルトフォントを登録することで、日本語テキストを正しく表示できる
        let default_fonts = vec![
            ("NotoSansJP", "NotoSansJP-Regular.ttf"),
            ("NotoSansJP-Bold", "NotoSansJP-Bold.ttf"),
            ("NotoSerifJP", "NotoSerifJP-Regular.ttf"),
        ];

        for (name, filename) in default_fonts {
            let font_path = resolve_font_path(&self.font_dir, filename);
            if let Some(font_path) = font_path {
                let font_path_clone = font_path.clone();
                self.register_font(name.to_string(), font_path);
                info!("フォントを登録しました: {} -> {:?}", name, font_path_clone);
            } else {
                warn!(
                    "フォントファイルが見つかりません: {} (検索: {:?}, {:?})",
                    filename,
                    self.font_dir.join(filename),
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("fonts")
                        .join(filename)
                );
            }
        }

        // 同梱 Noto が無い環境向け: OS 同梱の日本語サンセリフを `NotoSansJP` 名で使う（プレビュー／PDF 生成と名前を揃える）
        if !self.fonts.contains_key("NotoSansJP") {
            if let Some(p) = system_fallback_japanese_sans() {
                match oxidize_pdf::fonts::FontLoader::load_from_file(&p) {
                    Ok(_) => {
                        let p2 = p.clone();
                        self.register_font("NotoSansJP".to_string(), p);
                        info!("システムフォントを NotoSansJP として登録しました: {:?}", p2);
                    }
                    Err(e) => warn!(
                        "システムフォントを PDF 用に読み込めませんでした（スキップ）: {:?} — {}",
                        p, e
                    ),
                }
            }
        }

        if !self.fonts.contains_key("Emoji") {
            if let Some(p) = system_fallback_emoji_font() {
                match oxidize_pdf::fonts::FontLoader::load_from_file(&p) {
                    Ok(_) => {
                        let p2 = p.clone();
                        self.register_font("Emoji".to_string(), p);
                        info!("システム絵文字フォントを Emoji として登録しました: {:?}", p2);
                    }
                    Err(e) => warn!(
                        "システム絵文字フォントを PDF 用に読み込めませんでした（スキップ）: {:?} — {}",
                        p, e
                    ),
                }
            }
        }
    }
}

/// プロジェクトに TTF が無いとき、よくある OS 標準パスから日本語サンセリフを探す。
fn system_fallback_japanese_sans() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let windir = std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into());
        let fonts = Path::new(&windir).join("Fonts");
        // Windows 10/11 で配布される Noto Sans JP 可変フォント（ユーザー環境で実在を確認済みの例）
        let candidates = [
            fonts.join("NotoSansJP-VF.ttf"),
            fonts.join("YuGothR.ttf"),
            fonts.join("YuGothM.ttf"),
            fonts.join("YuGothic.ttf"),
            fonts.join("meiryo.ttf"),
        ];
        for p in candidates {
            if p.is_file() {
                return Some(p);
            }
        }
        // フォールバック: Fonts 内で NotoSansJP*.ttf（TTC は oxidize が読めないことが多いので除外）
        if let Ok(entries) = std::fs::read_dir(&fonts) {
            for ent in entries.flatten() {
                let p = ent.path();
                if !p.is_file() {
                    continue;
                }
                let Some(fname) = p.file_name() else {
                    continue;
                };
                let name = fname.to_string_lossy().to_ascii_lowercase();
                if name.starts_with("notosansjp") && name.ends_with(".ttf") {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// 絵文字や記号の表示を補うためのシステムフォントを探す。
fn system_fallback_emoji_font() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let windir = std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into());
        let fonts = Path::new(&windir).join("Fonts");
        // why: Segoe UI Emoji は `🧪` のような広い絵文字集合を持つため、
        //      まずこちらを試してから、記号系の Segoe UI Symbol に落とす
        // alt: seguisym.ttf を先に使う（カバー範囲が狭く、絵文字が抜けやすい）
        let candidates = [fonts.join("seguiemj.ttf"), fonts.join("seguisym.ttf")];
        for p in candidates {
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// 実行ファイル隣接の `fonts/` に加え、開発時は `src-tauri/fonts/`（Cargo マニフェスト直下）も参照する。
fn resolve_font_path(exe_font_dir: &Path, filename: &str) -> Option<PathBuf> {
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fonts")
        .join(filename);
    if dev.exists() {
        return Some(dev);
    }
    let prod = exe_font_dir.join(filename);
    if prod.exists() {
        return Some(prod);
    }
    None
}

/// レジストリに登録済みのフォントを PDF ドキュメントへ埋め込む。
pub fn register_fonts_on_document(doc: &mut Document) -> Result<(), String> {
    for font_name in get_available_fonts() {
        let Some(path) = get_font_path(&font_name) else {
            continue;
        };
        match doc.add_font(&font_name, &path) {
            Ok(()) => info!(
                "PDF にフォントを埋め込みました: {} -> {:?}",
                font_name, path
            ),
            Err(e) => warn!("フォントの埋め込みに失敗しました {}: {}", font_name, e),
        }
    }
    Ok(())
}

/// フォントレジストリを取得
fn get_registry() -> &'static FontRegistry {
    FONT_REGISTRY.get_or_init(|| {
        let mut registry = FontRegistry::new();
        registry.initialize_default_fonts();
        registry
    })
}

/// フォントレジストリを再スキャン
/// why: 実行中にフォントファイルが追加された場合に、再スキャンして認識できるようにする
/// alt: 再スキャンしない（実行中に追加されたフォントが認識されない）
/// evidence: 再スキャンにより、実行中に追加されたフォントも使用できる
#[allow(dead_code)]
pub fn rescan_fonts() {
    // why: OnceLockは一度初期化されると変更できないため、新しいレジストリを作成できない
    // alt: 再スキャン機能を実装（OnceLockの制約により不可能）
    // evidence: OnceLockはスレッドセーフな初期化を保証するが、再初期化はできない
    // 注意: 実行中にフォントを追加する場合は、アプリケーションの再起動が必要
    log::warn!("フォントレジストリの再スキャンは現在サポートされていません。アプリケーションを再起動してください。");
}

/// フォント名からフォントパスを取得
pub fn get_font_path(font_name: &str) -> Option<PathBuf> {
    get_registry().get_font_path(font_name).cloned()
}

/// フォントディレクトリのパスを取得
#[allow(dead_code)]
pub fn get_font_dir() -> PathBuf {
    get_registry().font_dir().to_path_buf()
}

/// 利用可能なフォント名のリストを取得
pub fn get_available_fonts() -> Vec<String> {
    get_registry().fonts.keys().cloned().collect()
}

/// フォントが利用可能かチェック
#[allow(dead_code)] // 将来のコマンドや UI から参照する想定の公開 API
pub fn is_font_available(font_name: &str) -> bool {
    get_registry().get_font_path(font_name).is_some()
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use std::path::Path;

    /// Windows に同梱される Noto Sans JP VF があれば oxidize が読めること（日本語プレビュー前提）
    #[cfg(windows)]
    #[test]
    fn noto_vf_loads_when_present() {
        let windir = std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into());
        let p = Path::new(&windir).join("Fonts").join("NotoSansJP-VF.ttf");
        if !p.is_file() {
            return;
        }
        let r = oxidize_pdf::fonts::FontLoader::load_from_file(&p);
        assert!(
            r.is_ok(),
            "NotoSansJP-VF.ttf は PDF 用に読み込める必要があります: {:?} — {:?}",
            p,
            r.err()
        );
    }

    /// Windows の絵文字フォントがあれば oxidize が読めること。
    #[cfg(windows)]
    #[test]
    fn emoji_font_loads_when_present() {
        let Some(p) = super::system_fallback_emoji_font() else {
            return;
        };
        let r = oxidize_pdf::fonts::FontLoader::load_from_file(&p);
        assert!(
            r.is_ok(),
            "emoji font should be loadable for PDF preview: {:?} — {:?}",
            p,
            r.err()
        );
    }

    /// Windows では広い絵文字カバレッジを持つ Segoe UI Emoji を優先する。
    #[cfg(windows)]
    #[test]
    fn emoji_font_prefers_seguiemj_when_available() {
        let Some(p) = super::system_fallback_emoji_font() else {
            return;
        };
        let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        assert_eq!(
            fname.to_ascii_lowercase(),
            "seguiemj.ttf",
            "expected Segoe UI Emoji to be preferred for broader emoji coverage"
        );
    }
}
