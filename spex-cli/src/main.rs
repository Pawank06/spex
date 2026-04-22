mod cli;

use clap::Parser;
use cli::{Cli, Cmd};

fn main() {
    let args = Cli::parse();

    match args.cmd {
        Cmd::Send => println!("send"),
        Cmd::Recv => println!("recv"),
    }
}
