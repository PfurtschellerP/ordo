//! `ordo` command line.

use std::process::ExitCode;

use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_server::config::to_hex;
use ordo_server::{Config, start};

const USAGE: &str = "\
Usage:
  ordo serve                 Run the server (configuration from environment variables)
  ordo generate-key          Print a new master key (64 hex characters)
  ordo generate-device-key   Print a new device key pair (for the seed file and test clients)
  ordo healthcheck           Exit 0 if the local server answers /healthz (for containers)
  ordo --version

Environment for `serve`:
  ORDO_SERVER_NAME       Server name, e.g. ordo.drk-musterstadt.de (required)
  ORDO_MASTER_KEY_FILE   File with the master key (required; see generate-key)
  ORDO_LISTEN            Listen address (default 0.0.0.0:8080)
  ORDO_DATA_DIR          Data directory (default /data)
  ORDO_SEED_FILE         Users and devices (TOML, optional)
  ORDO_PROTECTED         Protected compartments (default med.clinical)
  ORDO_APP_DIR           Built web app, served at / (optional)
  ORDO_POLICY_DIR        Own roles and permissions as Cedar policies, *.cedar (optional)
  ORDO_OIDC_ISSUER       Identity provider for login (OpenID Connect, optional); then also:
  ORDO_OIDC_CLIENT_ID    Client ID of ORDO at the identity provider
  ORDO_OIDC_CLIENT_SECRET_FILE  File with the client secret
  ORDO_PUBLIC_URL        Address of ORDO, e.g. https://ordo.example.org
  ORDO_ACCESS_TIMEOUT_HOURS  How long a device login lasts without re-verification
                         (default 48, from 24 to 720 = 1 to 30 days)
  ORDO_ADMINS            Administrators by user name, comma-separated, e.g. dora
                         (plan incidents, grant roles)
  ORDO_GRAVATAR          Profile pictures from Gravatar: on (default) or off
  ORDO_MAX_BLOB_MB       Largest attachment in MB (default 20, 1 to 200); attachments
                         are stored in <ORDO_DATA_DIR>/blobs, or with ORDO_S3_BUCKET in
                         an S3-compatible store (Garage):
  ORDO_S3_BUCKET         Bucket for attachments (optional)
  ORDO_S3_ENDPOINT       Address of the S3 API, e.g. http://garage:3900
  ORDO_S3_REGION         Region name (default garage)
  ORDO_S3_ACCESS_KEY_ID  Access key ID
  ORDO_S3_SECRET_ACCESS_KEY_FILE  File with the secret access key
  ORDO_LOG_FORMAT        json (default) or pretty; RUST_LOG sets the level (default info)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("serve") => serve(),
        Some("generate-key") => {
            println!("{}", to_hex(&random::<32>()));
            ExitCode::SUCCESS
        }
        Some("generate-device-key") => {
            let key = SigningKey::from_bytes(&random::<32>());
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as u64);
            let id = ordo_core::Ulid::from_parts(now, u128::from_be_bytes(random::<16>()))
                .expect("time fits");
            println!("# for the seed file:");
            println!("id = \"{id}\"");
            println!(
                "public_key = \"{}\"",
                to_hex(key.verifying_key().as_bytes())
            );
            println!("# for the device only — keep secret:");
            println!("private_key = \"{}\"", to_hex(&key.to_bytes()));
            ExitCode::SUCCESS
        }
        Some("healthcheck") => healthcheck(),
        Some("--version" | "-V") => {
            println!("ordo {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn serve() -> ExitCode {
    init_logging();
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(%error, "invalid configuration");
            return ExitCode::from(2);
        }
    };
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async {
        let running = match start(config).await {
            Ok(running) => running,
            Err(error) => {
                tracing::error!(%error, "start failed");
                return ExitCode::FAILURE;
            }
        };
        let stopper = running.stopper();
        tokio::spawn(async move {
            shutdown_signal().await;
            tracing::info!("shutting down");
            let _ = stopper.send(true);
        });
        match running.wait().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                tracing::error!(%error, "stopped with error");
                ExitCode::FAILURE
            }
        }
    })
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = ctrl_c.await;
    }
}

/// Asks the server on this machine for `/healthz`. The container image has no shell or
/// curl, so the health check is part of the binary.
fn healthcheck() -> ExitCode {
    use std::io::{Read, Write};
    let listen = std::env::var("ORDO_LISTEN").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let port = listen.rsplit(':').next().unwrap_or("8080");
    let check = || -> std::io::Result<bool> {
        let address = format!("127.0.0.1:{port}");
        let mut stream = std::net::TcpStream::connect(&address)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(3)))?;
        stream
            .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        Ok(response.starts_with("HTTP/1.1 200"))
    };
    match check() {
        Ok(true) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout);
    if std::env::var("ORDO_LOG_FORMAT").as_deref() == Ok("pretty") {
        builder.init();
    } else {
        builder.json().init();
    }
}

fn random<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).expect("the operating system provides randomness");
    bytes
}
