use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NoPortCommunication {
    CreateHost {
        domain: String,
        port: i32,
        path: String,
    },
    RemoveHost {
        domain: String,
    },
    Status,
    Stop,
    Ok,
}

pub fn get_socket<'a>() -> &'a str {
    let uid = nix::unistd::Uid::current();

    if uid.is_root() {
        "/var/run/noport.socket"
    } else {
        "/tmp/noport.socket"
    }
}

pub fn find_socket<'a>() -> Result<&'a str, anyhow::Error> {
    if fs::exists("/tmp/noport.socket")? {
        return Ok("/tmp/noport.socket");
    }
    if fs::exists("/var/run/noport.socket")? {
        return Ok("/var/run/noport.socket");
    }

    Err(anyhow::Error::msg("Could not find a socket"))
}
