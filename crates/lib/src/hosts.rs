use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

const START: &str = "# noport start";
const END: &str = "# noport stop";

struct HostFile {
    user_hosts: Vec<String>,
    noport_hosts: Vec<String>,
}

/// Remove all the noport related hosts
fn parse_host<'a>() -> Result<HostFile, anyhow::Error> {
    let host_path = Path::new("/etc/hosts");
    let host_file = fs::read_to_string(host_path)?;

    let mut file_acc = vec![];
    let mut noport_lines = vec![];

    let lines = host_file.split('\n').into_iter();

    let mut inside_noport = false;

    for line in lines {
        match line {
            START => {
                inside_noport = true;
            }
            END => {
                inside_noport = false;
            }
            other_lines => {
                if inside_noport {
                    noport_lines.push(other_lines.to_string());
                } else {
                    file_acc.push(other_lines.to_string());
                }
            }
        }
    }

    return Ok(HostFile {
        user_hosts: file_acc,
        noport_hosts: noport_lines,
    });
}

pub fn write_host(new_host: String) -> Result<(), anyhow::Error> {
    let mut host = parse_host()?;

    host.user_hosts.push(new_host);

    let host_path = Path::new("/etc/hosts");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(host_path)?;

    let new_host_file = format!(
        "{}\n\n{}{}{}",
        host.user_hosts.join("\n"),
        START,
        host.noport_hosts.join("\n"),
        END
    );

    file.write(new_host_file.as_bytes())?;
    file.flush()?;

    Ok(())
}

pub fn clear_host() -> Result<(), anyhow::Error> {
    let host = parse_host()?;

    let host_path = Path::new("/etc/hosts");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(host_path)?;

    let new_host_file = format!("{}\n\n{}{}", host.user_hosts.join("\n"), START, END);

    file.write(new_host_file.as_bytes())?;
    file.flush()?;

    Ok(())
}
