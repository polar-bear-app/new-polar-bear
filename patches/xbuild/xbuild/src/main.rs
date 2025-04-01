use anyhow::Result;
use app_store_connect::certs_api::CertificateType;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xbuild::{cargo::config::LocalizedConfig, command, BuildArgs, BuildEnv};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
    tracing_log::LogTracer::init().ok();
    let env = std::env::var("XBUILD_LOG").unwrap_or_else(|_| "error".into());
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_span_events(FmtSpan::ACTIVE | FmtSpan::CLOSE)
        .with_env_filter(EnvFilter::new(env))
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();
    log_panics::init();
    let args = Args::parse();
    args.command.run()
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a new rust project
    New {
        /// Project name
        name: String,
    },
    /// Show information about the installed tooling
    Doctor,
    /// List all connected devices
    Devices,
    /// Build an executable app or install bundle
    Build {
        #[clap(flatten)]
        args: BuildArgs,
    },
    /// Run app on an attached device
    Run {
        #[clap(flatten)]
        args: BuildArgs,
    },
    /// Launch app in a debugger on an attached device
    Lldb {
        #[clap(flatten)]
        args: BuildArgs,
    },
    /// Build and run tests on an attached device
    Test {
        #[clap(flatten)]
        args: BuildArgs,
    },
    /// Generates a PEM encoded RSA2048 signing key
    GenerateKey {
        /// Path to unified api key.
        #[clap(long)]
        api_key: PathBuf,
        #[clap(long)]
        r#type: CertificateType,
        /// Path to write a new PEM encoded RSA2048 signing key
        pem: PathBuf,
    },
    CreateAppleApiKey {
        /// Issuer id.
        #[clap(long)]
        issuer_id: String,
        /// Key id.
        #[clap(long)]
        key_id: String,
        /// Path to private key.
        private_key: PathBuf,
        /// Path to write a unified api key.
        api_key: PathBuf,
    },
}

/// Setup a partial build environment (e.g. read `[env]` from `.cargo/config.toml`) when there is
/// no crate/manifest selected. Pretend `$PWD` is the workspace.
///
/// Only necessary for apps that don't call [`BuildEnv::new()`],
fn partial_build_env() -> Result<()> {
    let config = LocalizedConfig::find_cargo_config_for_workspace(".")?;
    if let Some(config) = &config {
        config.set_env_vars()?;
    }
    Ok(())
}

impl Commands {
    pub fn run(self) -> Result<()> {
        match self {
            Self::New { name } => command::new(&name)?,
            Self::Doctor => {
                partial_build_env()?;
                command::doctor()
            }
            Self::Devices => {
                partial_build_env()?;
                command::devices()?
            }
            Self::Build { args } => {
                let env = BuildEnv::new(args)?;
                command::build(&env)?;
            }
            Self::Run { args } => {
                let env = BuildEnv::new(args)?;
                command::build(&env)?;
                command::run(&env)?;
            }
            Self::Test { args } => {
                let env = BuildEnv::new(args)?;
                command::test(&env)?;
            }
            Self::Lldb { args } => {
                let env = BuildEnv::new(args)?;
                command::build(&env)?;
                command::lldb(&env)?;
            }
            Self::GenerateKey {
                api_key,
                r#type,
                pem,
            } => {
                app_store_connect::certs_api::generate_signing_certificate(&api_key, r#type, &pem)?
            }
            Self::CreateAppleApiKey {
                issuer_id,
                key_id,
                private_key,
                api_key,
            } => {
                command::create_apple_api_key(&issuer_id, &key_id, &private_key, &api_key)?;
            }
        }
        Ok(())
    }
}
