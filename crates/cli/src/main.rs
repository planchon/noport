use clap::Parser;
use clap::Subcommand;

use ansi_term::Colour;

mod subprocess;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct NoPort {
    #[command(subcommand)]
    command: Option<NoPortCommand>,

    // everything after -- is treated as "child process" arguments
    #[arg(last = true)]
    child_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum NoPortCommand {
    // setup the HTTPS certificate and key
    Setup,
    // start the proxy server
    Start,
}

fn main() {
    let cli = NoPort::parse();

    if let Some(command) = cli.command {
        match command {
            NoPortCommand::Setup => {
                println!("Setup command");
            }
            NoPortCommand::Start => {
                println!("Start command");
            }
        }
    }

    // run the child process
    if !cli.child_args.is_empty() {
        let main_command = cli.child_args[0].clone();
        println!(
            "\n\n{} {}\n",
            Colour::Green.paint("Starting child process"),
            Colour::Yellow.paint(main_command),
        );

        subprocess::start(cli.child_args);
    }
}
