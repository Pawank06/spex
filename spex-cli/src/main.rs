mod cli;

use clap::Parser;
use cli::{Cli, Cmd};

fn main() {
    let args = Cli::parse();

    match args.cmd {
        Cmd::Send { bind, peer } => println!("send {bind} -> {peer}"),
        Cmd::Recv { bind, peer } => println!("recv {bind} <- {peer}"),
    }
}
