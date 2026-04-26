use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub chunk_size: usize,
    pub send_delay_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chunk_size: 1024,
            send_delay_ms: 50,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Endpoints {
    pub bind: SocketAddr,
    pub peer: SocketAddr,
}
