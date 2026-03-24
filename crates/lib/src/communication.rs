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
