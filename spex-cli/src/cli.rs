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
    },
    Recv,
}
