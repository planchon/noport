use ansi_term::Colour;
use noport_lib::store::Store;

pub fn status(store: Store) -> Result<(), anyhow::Error> {
    if let Ok(pid) = store.get_daemon_process_id() {
        println!(
            "{}\n{} {}",
            Colour::Fixed(29).paint("NoPort daemon is running"),
            Colour::Fixed(244).italic().paint("Running on PID:"),
            pid
        )
    } else {
        println!("{}", Colour::Fixed(29).paint("NoPort is not running"));
    }

    Ok(())
}
