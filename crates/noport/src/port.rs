use std::time::Duration;

use rand::RngExt;
use tokio::net::TcpStream;

const MAX_ATTEMPTS: u16 = 5;

pub async fn find_free_port() -> Result<i32, anyhow::Error> {
    let mut attempts = 0;

    loop {
        if attempts > MAX_ATTEMPTS {
            return Err(anyhow::anyhow!("Could not find a free port"));
        }

        let port = generate_random_port();

        if port_is_free(port).await {
            return Ok(port);
        }

        attempts += 1;
    }
}

fn generate_random_port() -> i32 {
    let mut rng = rand::rng();

    rng.random_range(10000..19999)
}

async fn port_is_free(port: i32) -> bool {
    let socket = format!("127.0.0.1:{}", port);
    let stream = TcpStream::connect(socket);

    if let Err(_) = tokio::time::timeout(Duration::from_secs(5), stream).await {
        return false;
    }

    true
}
