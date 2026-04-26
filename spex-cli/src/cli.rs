use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "spex", version, about = "streaming protocol experiment")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    Send {
        #[arg(long, default_value = "127.0.0.1:7001")]
        bind: String,
        #[arg(long, default_value = "127.0.0.1:7002")]
        peer: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value_t = 1024)]
        chunk_size: usize,
    },
    Recv {
        #[arg(long, default_value = "127.0.0.1:7002")]
        bind: String,
        #[arg(long, default_value = "127.0.0.1:7001")]
        peer: String,
        #[arg(long)]
        out: PathBuf,
    },
}
