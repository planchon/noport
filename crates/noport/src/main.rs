use clap::Parser;
use clap::Subcommand;

use noport_lib::store::Store;

use noport_lib::setup::setup_certificate;
use noport_lib::subprocess::start;
use paris::success;
use tokio::runtime::Runtime;

use crate::start::start_background;
use crate::start::start_foreground;

mod setup;
mod start;
mod status;
mod stop;

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

    /// Set the app port to use
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
    },
    Stop,
    Status,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = NoPort::parse();
    let store = Store::new();

    if let Some(command) = cli.command {
        match command {
            // NoPortCommand::Setup => {
            //     setup_certificate();
            // }
            NoPortCommand::Stop => {
                return stop::stop_daemon(store);
            }
            NoPortCommand::Status => {
                return status::status().await;
            }
            // start the daemon proxy server
            // this part could run in sudo
            NoPortCommand::Start { foreground, tld } => {
                store.set_tld(tld)?;

                if foreground {
                    return start_foreground(store).await;
                } else {
                    return start_background(store);
                }
            }
        }
    }

    if !cli.child_args.is_empty() {
        success!("Starting the child process ({})", cli.child_args.join(" "));

        start(cli.child_args, store).await;
    }

    Ok(())
}
