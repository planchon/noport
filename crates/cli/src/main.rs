use clap::Parser;
use clap::Subcommand;

use ansi_term::Colour;
use noport_lib::store::Store;

use noport_lib::setup::setup_certificate;
use noport_lib::subprocess::start;

use crate::start::start_background;
use crate::start::start_foreground;

mod setup;
mod start;
mod stop;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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
    /// Setup HTTPS certificate and key
    Setup,
    /// Start the proxy server
    Start {
        /// Run the daemon in the foreground
        #[arg(short, long, default_value_t = false)]
        foreground: bool,
    },
    Stop,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = NoPort::parse();
    let store = Store::new();

    if let Some(command) = cli.command {
        match command {
            NoPortCommand::Setup => {
                setup_certificate();
            }
            NoPortCommand::Stop => {
                return stop::stop_daemon(store);
            }
            // start the daemon proxy server
            NoPortCommand::Start { foreground } => {
                if foreground {
                    return start_foreground(store);
                } else {
                    return start_background(store);
                }
            }
        }
    }

    // // run the child process
    if !cli.child_args.is_empty() {
        println!(
            "\n{}\n\n{}\n",
            Colour::Fixed(29).paint("Starting child process"),
            Colour::Fixed(242).paint(cli.child_args.join(" "))
        );

        start(cli.child_args);
    }

    Ok(())
}
