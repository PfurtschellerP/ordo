//! `ordo-tokens` command-line tool.

use std::path::PathBuf;
use std::process::ExitCode;

use ordo_tokens::color::format_ratio;
use ordo_tokens::{DesignSystem, ThemeReport, css, report};

const USAGE: &str = "\
Usage:
  ordo-tokens check [--design DIR]
      Check all token files and the contrast of every theme.
  ordo-tokens build --out DIR [--design DIR]
      Check, then write ordo-tokens.css and contrast-report.md to DIR.
  ordo-tokens check-theme FILE --base THEME [--design DIR]
      Check an organisation theme (DTCG JSON) on top of THEME.

DIR defaults to ./design.";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(Failure::Usage(message)) => {
            eprintln!("{message}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Err(Failure::Error(error)) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

enum Failure {
    Usage(String),
    Error(ordo_tokens::Error),
}

impl From<ordo_tokens::Error> for Failure {
    fn from(error: ordo_tokens::Error) -> Self {
        Self::Error(error)
    }
}

struct Args {
    command: String,
    positional: Vec<String>,
    design: PathBuf,
    out: Option<PathBuf>,
    base: Option<String>,
}

fn parse(args: Vec<String>) -> Result<Args, Failure> {
    let mut iter = args.into_iter();
    let command = iter
        .next()
        .ok_or_else(|| Failure::Usage("missing command".into()))?;
    let mut parsed = Args {
        command,
        positional: Vec::new(),
        design: PathBuf::from("design"),
        out: None,
        base: None,
    };
    while let Some(arg) = iter.next() {
        let mut value = |name: &str| {
            iter.next()
                .ok_or_else(|| Failure::Usage(format!("{name} needs a value")))
        };
        match arg.as_str() {
            "--design" => parsed.design = value("--design")?.into(),
            "--out" => parsed.out = Some(value("--out")?.into()),
            "--base" => parsed.base = Some(value("--base")?),
            "-h" | "--help" => return Err(Failure::Usage("ORDO design token tool".into())),
            flag if flag.starts_with("--") => {
                return Err(Failure::Usage(format!("unknown option {flag}")));
            }
            _ => parsed.positional.push(arg),
        }
    }
    Ok(parsed)
}

fn run(args: Vec<String>) -> Result<bool, Failure> {
    let args = parse(args)?;
    let system = DesignSystem::load(&args.design)?;
    match args.command.as_str() {
        "check" => {
            let report = system.check()?;
            report.themes.iter().for_each(print_summary);
            Ok(report.passes())
        }
        "build" => {
            let out = args
                .out
                .ok_or_else(|| Failure::Usage("build needs --out DIR".into()))?;
            let report = system.check()?;
            report.themes.iter().for_each(print_summary);
            if !report.passes() {
                eprintln!("not writing output: contrast checks failed");
                return Ok(false);
            }
            let write = |name: &str, content: String| {
                std::fs::create_dir_all(&out)
                    .and_then(|()| std::fs::write(out.join(name), content))
                    .map_err(|source| {
                        Failure::Error(ordo_tokens::Error::Io {
                            file: out.join(name).display().to_string(),
                            source,
                        })
                    })
            };
            write("ordo-tokens.css", css::generate(&system)?)?;
            write("contrast-report.md", report::markdown(&report))?;
            println!("wrote {}", out.display());
            Ok(true)
        }
        "check-theme" => {
            let [file] = args.positional.as_slice() else {
                return Err(Failure::Usage("check-theme needs exactly one FILE".into()));
            };
            let base = args
                .base
                .ok_or_else(|| Failure::Usage("check-theme needs --base THEME".into()))?;
            let text = std::fs::read_to_string(file).map_err(|source| {
                Failure::Error(ordo_tokens::Error::Io {
                    file: file.clone(),
                    source,
                })
            })?;
            let report = system.check_overlay(&base, &text, file)?;
            print_summary(&report);
            Ok(report.passes())
        }
        other => Err(Failure::Usage(format!("unknown command {other}"))),
    }
}

fn print_summary(theme: &ThemeReport) {
    let failures: Vec<_> = theme.failures().collect();
    if failures.is_empty() {
        let tightest = theme
            .tightest()
            .map(|r| {
                format!(
                    "; closest to its minimum: {} at {} (needs {})",
                    r.pair.purpose,
                    format_ratio(r.ratio),
                    format_ratio(r.pair.min)
                )
            })
            .unwrap_or_default();
        println!(
            "{}: all {} checks pass{tightest}",
            theme.theme,
            theme.results.len()
        );
    } else {
        println!(
            "{}: {} of {} checks FAIL",
            theme.theme,
            failures.len(),
            theme.results.len()
        );
        for r in failures {
            println!(
                "  {} — {} on {}: {} < {} (WCAG {})",
                r.pair.purpose,
                r.pair.fg,
                r.pair.bg,
                format_ratio(r.ratio),
                format_ratio(r.pair.min),
                r.pair.criterion
            );
        }
    }
}
