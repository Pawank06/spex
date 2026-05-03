use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "spex",
    version,
    about = "streaming protocol experiment",
    long_about = "Send and receive files over UDP with chunking, hashing, and retransmission."
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,

    /// Increase log verbosity (-v info, -vv debug)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Send a file to a peer
    Send {
        #[arg(long, default_value = "127.0.0.1:7001")]
        bind: String,
        #[arg(long, default_value = "127.0.0.1:7002")]
        peer: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value_t = 1024)]
        chunk_size: usize,
        #[arg(long, default_value_t = 50)]
        delay_ms: u64,
    },
    /// Receive a file from a peer
    Recv {
        #[arg(long, default_value = "127.0.0.1:7002")]
        bind: String,
        #[arg(long, default_value = "127.0.0.1:7001")]
        peer: String,
        #[arg(long)]
        out: PathBuf,
    },
}

impl Cmd {
    pub fn name(&self) -> &'static str {
        match self {
            Cmd::Send { .. } => "send",
            Cmd::Recv { .. } => "recv",
        }
    }
}
