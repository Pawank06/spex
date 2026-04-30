use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub chunk_size: usize,
    pub send_delay_ms: u64,
    pub retry_after_ms: u64,
    pub max_retries: u32,
    pub idle_timeout_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chunk_size: 1024,
            send_delay_ms: 50,
            retry_after_ms: 200,
            max_retries: 5,
            idle_timeout_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Endpoints {
    pub bind: SocketAddr,
    pub peer: SocketAddr,
}
