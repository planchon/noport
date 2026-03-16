use clap::Parser;
use clap::Subcommand;

use ansi_term::Colour;
use tokio::runtime::Runtime;

mod setup;
mod subprocess;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct NoPort {
    #[command(subcommand)]
    command: Option<NoPortCommand>,

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

    if let Some(command) = cli.command {
        match command {
            NoPortCommand::Setup => {
                setup::setup();
            }
            NoPortCommand::Start => {
                let runtime = Runtime::new().unwrap();
                println!(
                    "\n{} {}\n",
                    Colour::Fixed(29).paint("Starting proxy server"),
                    Colour::Fixed(31).paint("(:2828)")
                );
                let result = runtime.block_on(daemon::start_deamon(None));

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

        subprocess::start(cli.child_args);
    }
}
