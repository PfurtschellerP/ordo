//! `ordo-profiles`: maintainers' tool for regional profiles.
//!
//! ```text
//! ordo-profiles check    [--root profiles] [--register DOC --parent ID]
//! ordo-profiles register  --register DOC --parent ID [--root profiles]
//! ordo-profiles watch    [--root profiles] [--report FILE]
//! ordo-profiles accept   [--root profiles] (--all | ID…)
//! ```
//!
//! `check` validates the manifests and that the register is current (offline, in CI).
//! `register` regenerates the register table. `watch` fetches every source and reports
//! changed or unreachable ones (exit code 2), for the weekly job. `accept` records, after a
//! maintainer reviewed the changes, today's date and the current fingerprint.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use ordo_profiles::{
    MANUAL_DAYS, Profile, Watch, fingerprint, load, manual_due, register_table, splice_register,
    today, validate,
};

struct Args {
    command: String,
    root: PathBuf,
    register: Option<PathBuf>,
    parent: Option<String>,
    report: Option<PathBuf>,
    all: bool,
    ids: Vec<String>,
}

fn args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let command = it
        .next()
        .ok_or("command missing: check, register, watch or accept")?;
    let mut a = Args {
        command,
        root: PathBuf::from("profiles"),
        register: None,
        parent: None,
        report: None,
        all: false,
        ids: Vec::new(),
    };
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--root" => a.root = PathBuf::from(value()?),
            "--register" => a.register = Some(PathBuf::from(value()?)),
            "--parent" => a.parent = Some(value()?),
            "--report" => a.report = Some(PathBuf::from(value()?)),
            "--all" => a.all = true,
            other if !other.starts_with('-') => a.ids.push(other.to_owned()),
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(a)
}

fn main() -> ExitCode {
    let result = args().and_then(|a| match a.command.as_str() {
        "check" => check(&a),
        "register" => register(&a),
        "watch" => watch(&a),
        "accept" => accept(&a),
        other => Err(format!("unknown command {other}")),
    });
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn check(a: &Args) -> Result<ExitCode, String> {
    let profiles = load(&a.root)?;
    let problems = validate(&profiles);
    for p in &problems {
        eprintln!("{p}");
    }
    if !problems.is_empty() {
        return Ok(ExitCode::FAILURE);
    }
    if let (Some(doc), Some(parent)) = (&a.register, &a.parent) {
        let text = read(doc)?;
        let new = splice_register(&text, &register_table(&profiles, parent))
            .ok_or(format!("{}: register markers missing", doc.display()))?;
        if new != text {
            eprintln!(
                "{} is out of date: run `cargo run -p ordo-profiles -- register --register {} --parent {parent}`",
                doc.display(),
                doc.display()
            );
            return Ok(ExitCode::FAILURE);
        }
    }
    println!("{} profiles valid", profiles.len());
    Ok(ExitCode::SUCCESS)
}

fn register(a: &Args) -> Result<ExitCode, String> {
    let (Some(doc), Some(parent)) = (&a.register, &a.parent) else {
        return Err("register needs --register DOC and --parent ID".into());
    };
    let profiles = load(&a.root)?;
    let text = read(doc)?;
    let new = splice_register(&text, &register_table(&profiles, parent))
        .ok_or(format!("{}: register markers missing", doc.display()))?;
    std::fs::write(doc, new).map_err(|e| e.to_string())?;
    Ok(ExitCode::SUCCESS)
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// What fetching one source gave.
enum Seen {
    Fingerprint(String),
    Failed(String),
}

fn client() -> Result<reqwest::Client, String> {
    // TLS with ring, like the server.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .user_agent(concat!(
            "ordo-profiles/",
            env!("CARGO_PKG_VERSION"),
            " (source check of the ORDO project)"
        ))
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())
}

async fn fetch(client: &reqwest::Client, source: &ordo_profiles::Source) -> Seen {
    let response = match client.get(&source.url).send().await {
        Ok(r) => r,
        Err(e) => return Seen::Failed(e.without_url().to_string()),
    };
    if !response.status().is_success() {
        return Seen::Failed(format!("HTTP {}", response.status()));
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    match response.bytes().await {
        Ok(body) => Seen::Fingerprint(fingerprint(Watch::of(source, &content_type), &body)),
        Err(e) => Seen::Failed(e.without_url().to_string()),
    }
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}

fn watch(a: &Args) -> Result<ExitCode, String> {
    let profiles = load(&a.root)?;
    let client = client()?;
    let mut findings = Vec::new();
    let mut count = 0;
    runtime()?.block_on(async {
        for (id, Profile { manifest, .. }) in &profiles {
            for s in &manifest.source {
                count += 1;
                if s.watch.as_deref() == Some("manual") {
                    if manual_due(s.checked, time::OffsetDateTime::now_utc().date()) {
                        findings.push(format!(
                            "- `{id}` — [{}]({}): check by hand (last checked {}; due every {MANUAL_DAYS} days)",
                            s.title,
                            s.url,
                            s.checked.map(|d| d.to_string()).unwrap_or_else(|| "never".into())
                        ));
                    }
                    continue;
                }
                let line = match (fetch(&client, s).await, &s.fingerprint) {
                    (Seen::Failed(e), _) => Some(format!("cannot be fetched ({e})")),
                    (Seen::Fingerprint(_), None) => Some("not confirmed yet".to_owned()),
                    (Seen::Fingerprint(now), Some(then)) if &now != then => Some(format!(
                        "changed since {}",
                        s.checked.map(|d| d.to_string()).unwrap_or_default()
                    )),
                    _ => None,
                };
                if let Some(line) = line {
                    findings.push(format!("- `{id}` — [{}]({}): {line}", s.title, s.url));
                }
            }
        }
    });
    println!("{count} sources fetched, {} to review", findings.len());
    let report = if findings.is_empty() {
        String::new()
    } else {
        format!(
            "These profile sources changed or could not be fetched. Review each one, update \
             the profile if the rules changed, then record the review with \
             `cargo run -p ordo-profiles -- accept <profile>`.\n\n{}\n",
            findings.join("\n")
        )
    };
    print!("{report}");
    if let Some(path) = &a.report {
        std::fs::write(path, &report).map_err(|e| e.to_string())?;
    }
    Ok(if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    })
}

fn accept(a: &Args) -> Result<ExitCode, String> {
    if !a.all && a.ids.is_empty() {
        return Err("accept needs --all or profile IDs".into());
    }
    let profiles = load(&a.root)?;
    let client = client()?;
    let date = today();
    let rt = runtime()?;
    for (id, profile) in &profiles {
        if !a.all && !a.ids.contains(id) {
            continue;
        }
        let text = read(&profile.path)?;
        let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e| format!("{id}: {e}"))?;
        let Some(tables) = doc
            .get_mut("source")
            .and_then(|s| s.as_array_of_tables_mut())
        else {
            continue;
        };
        for (table, source) in tables.iter_mut().zip(&profile.manifest.source) {
            if source.watch.as_deref() == Some("manual") {
                table["checked"] = toml_edit::value(date);
                table["fingerprint"] = toml_edit::value("manual");
                continue;
            }
            match rt.block_on(fetch(&client, source)) {
                Seen::Fingerprint(f) => {
                    table["checked"] = toml_edit::value(date);
                    table["fingerprint"] = toml_edit::value(f);
                }
                Seen::Failed(e) => eprintln!("{id}: {}: {e} (not recorded)", source.url),
            }
        }
        std::fs::write(&profile.path, doc.to_string()).map_err(|e| e.to_string())?;
        println!("{id}: recorded");
    }
    Ok(ExitCode::SUCCESS)
}
