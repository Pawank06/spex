use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "spex", version, about = "streaming protocol experiment")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    Send,
    Recv,
}
