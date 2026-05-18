use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use minipdf::commands::markdown_preview::render_markdown_preview_pdf_bytes;

fn main() -> ExitCode {
    match run() {
        Ok(output_path) => {
            println!("{}", output_path.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<PathBuf, String> {
    let (input_path, output_path) = parse_args(env::args_os().skip(1))?;
    let markdown = fs::read_to_string(&input_path)
        .map_err(|error| format!("Markdownの読み込みに失敗しました: {error}"))?;
    let pdf_bytes = render_markdown_preview_pdf_bytes(&markdown)?;
    fs::write(&output_path, pdf_bytes)
        .map_err(|error| format!("PDFの書き込みに失敗しました: {error}"))?;
    Ok(output_path)
}

fn parse_args<I>(args: I) -> Result<(PathBuf, PathBuf), String>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut input_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "-h" | "--help" => return Err(usage()),
            "-o" | "--output" => {
                let next = args
                    .next()
                    .ok_or_else(usage)?;
                output_path = Some(PathBuf::from(next));
            }
            value if value.starts_with('-') => {
                return Err(format!("不明なオプションです: {value}\n\n{}", usage()));
            }
            _ => {
                if input_path.is_none() {
                    input_path = Some(PathBuf::from(&arg));
                } else if output_path.is_none() {
                    output_path = Some(PathBuf::from(&arg));
                } else {
                    return Err(format!("引数が多すぎます。\n\n{}", usage()));
                }
            }
        }
    }

    let input_path = input_path.ok_or_else(usage)?;
    let output_path = output_path.unwrap_or_else(|| default_output_path(&input_path));
    Ok((input_path, output_path))
}

fn default_output_path(input_path: &Path) -> PathBuf {
    let mut output_path = input_path.to_path_buf();
    let file_stem = input_path
        .file_stem()
        .map(|stem| stem.to_os_string())
        .unwrap_or_else(|| "preview".into());
    output_path.set_file_name(file_stem);
    output_path.set_extension("preview.pdf");
    output_path
}

fn usage() -> String {
    [
        "Usage:",
        "  markdown_preview_cli <input.md> [output.pdf]",
        "  markdown_preview_cli <input.md> --output <output.pdf>",
        "",
        "Options:",
        "  -o, --output <path>  出力先PDFを指定する",
        "  -h, --help           このヘルプを表示する",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_supports_explicit_output_path() {
        let (input, output) = parse_args([
            "input.md".into(),
            "--output".into(),
            "out.pdf".into(),
        ])
        .expect("args should parse");

        assert_eq!(input, PathBuf::from("input.md"));
        assert_eq!(output, PathBuf::from("out.pdf"));
    }

    #[test]
    fn parse_args_defaults_output_path_from_input_stem() {
        let (input, output) = parse_args(["fixtures/check.md".into()]).expect("args should parse");

        assert_eq!(input, PathBuf::from("fixtures/check.md"));
        assert_eq!(output, PathBuf::from("fixtures/check.preview.pdf"));
    }
}
