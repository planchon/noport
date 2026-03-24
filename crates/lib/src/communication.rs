use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NoPortCommunication {
    CreateHost {
        domain: String,
        port: u16,
        path: String,
    },
    RemoveHost {
        domain: String,
    },
    Status,
    Stop,
    Ok,
}
