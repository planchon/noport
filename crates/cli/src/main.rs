use clap::Parser;
use clap::Subcommand;

use ansi_term::Colour;
use tokio::runtime::Runtime;

use noport_lib::setup::setup_certificate;
use noport_lib::subprocess::start;

mod setup;

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
    Start,
}

fn main() {
    let cli = NoPort::parse();

    let use_git_branch = cli.git_branch;
    let use_git_worktree = cli.git_worktree;

    let override_domain = cli.domain;
    let override_app_port = cli.app_port;

    if let Some(command) = cli.command {
        match command {
            NoPortCommand::Setup => {
                setup_certificate();
            }
            NoPortCommand::Start => {
                let runtime = Runtime::new().unwrap();
                println!(
                    "\n{} {}\n",
                    Colour::Fixed(29).paint("Starting the daemon proxy server"),
                    Colour::Fixed(31).paint("(:2828)")
                );
                let result = runtime.block_on(daemon::daemon::start_deamon(None));

                if let Err(e) = result {
                    println!("{}", Colour::Red.paint(e.to_string()));
                }

                println!("{}", Colour::Fixed(50).paint("Proxy server started"));
            }
        }
    }

    // run the child process
    if !cli.child_args.is_empty() {
        println!(
            "\n{}\n\n{}\n",
            Colour::Fixed(29).paint("Starting child process"),
            Colour::Fixed(242).paint(cli.child_args.join(" "))
        );

        start(cli.child_args);
    }
}
