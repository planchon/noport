use std::process::exit;

use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use clap::Subcommand;

use noport_lib::cert;
use noport_lib::store::Store;
use tracing::info;

use crate::start::start_background;
use crate::start::start_foreground;
use crate::subprocess::rerun_as_sudo;
use crate::subprocess::start_subcommand;

mod start;
mod status;
mod stop;
mod subprocess;

#[derive(Parser)]
#[command(
    author = "Paul Planchon",
    version = "0.1.0",
    name = "noport",
    about = "Remove all port from your dev workflow",
    arg_required_else_help = true
)]
struct NoPort {
    #[command(subcommand)]
    command: Option<NoPortCommand>,

    /// Change the used subdomain
    #[arg(short, long)]
    domain: Option<String>,

    /// Force the port of the child app (your app)
    #[arg(short, long)]
    app_port: Option<u16>,

    /// Use the git branch name as subdomain
    #[arg(long, default_value_t = false)]
    git_branch: bool,

    /// Use the git worktree name as subdomain
    #[arg(long, default_value_t = false)]
    git_worktree: bool,

    /// Child process arguments (your command)
    #[arg(last = true)]
    child_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum NoPortCommand {
    /// Start the proxy server
    Start {
        /// Run the daemon in the foreground
        #[arg(short, long, default_value_t = false)]
        foreground: bool,

        /// Change the TLD (default is .localhost)
        /// You can use .test, .lan and .home without any problems
        /// all other TLDs can lead to problems
        #[arg(short, long, default_value = "localhost")]
        tld: String,

        #[arg(long, default_value_t = false)]
        https: bool,

        /// Port used by the proxy
        #[arg(short, long, default_value_t = 2828)]
        port: u16,
    },
    /// Stop the daemon
    Stop,
    /// Status of the daemon
    Status,
    /// Setup the CA certificate for local HTTPS
    Setup,
    /// Trust globally on your machine the CA root certificate
    Trust,
    /// Remove and untrust the CA root certificate
    Nuke,
}

fn need_sudo(cli: &NoPort) -> bool {
    if nix::unistd::Uid::current().is_root() {
        return false;
    }

    if let Some(command) = &cli.command {
        match command {
            NoPortCommand::Start {
                foreground: _,
                https,
                tld,
                port,
            } => {
                if (*port < 1024) || tld != "localhost" || *https {
                    return true;
                }
                return false;
            }
            _ => {
                return false;
            }
        }
    }

    false
}

#[tokio::main]
async fn run() -> Result<()> {
    let cli = NoPort::parse();

    if need_sudo(&cli) {
        rerun_as_sudo();
        exit(1);
    }

    if let Some(command) = cli.command {
        match command {
            NoPortCommand::Stop => {
                return stop::stop_daemon().await;
            }
            NoPortCommand::Status => {
                return status::status().await;
            }
            NoPortCommand::Start {
                foreground,
                tld,
                port,
                https,
            } => {
                let mut store = Store::new();
                store.set_tld(tld)?;

                if foreground {
                    return start_foreground(store, port, https).await;
                } else {
                    return start_background().await;
                }
            }
            NoPortCommand::Setup => {
                cert::setup_ca()?;
            }
            NoPortCommand::Trust => {
                cert::trust_certificate()?;
            }
            NoPortCommand::Nuke => {
                bail!("not implemented yet");
            }
        }
    }

    if !cli.child_args.is_empty() {
        info!("Starting the child process ({})", cli.child_args.join(" "));

        start_subcommand(cli.child_args).await;
    }

    Ok(())
}

pub fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    run()
}
