mod cli;

use clap::Parser;
use cli::{Cli, Cmd};
use tokio::net::UdpSocket;

use spex_net::config::Config;
use spex_net::{receiver, sender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Cli::parse();

    match args.cmd {
        Cmd::Send {
            bind,
            peer,
            file,
            chunk_size,
            delay_ms,
        } => {
            let socket = UdpSocket::bind(&bind).await?;
            let peer_addr = peer.parse()?;
            let cfg = Config {
                chunk_size,
                send_delay_ms: delay_ms,
            };
            sender::run(socket, peer_addr, file, cfg).await?;
        }
        Cmd::Recv { bind, peer, out } => {
            let socket = UdpSocket::bind(&bind).await?;
            let peer_addr = peer.parse()?;
            receiver::run(socket, peer_addr, out).await?;
        }
    }

    Ok(())
}
