mod capturing_writer;
mod curl;
mod db;
mod history;

use std::env;

pub type BoxError = Box<dyn std::error::Error>;

const USAGE: &str = "
USAGE:
    curl-history <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -v, --version    Prints version information

SUBCOMMANDS:
    curl
    history";

#[async_std::main]
async fn main() {
    match env::args().nth(1).as_deref() {
        Some("curl") => curl::run_and_save_to_history(env::args_os().skip(2).collect()).await,
        Some("history") => history::search(env::args_os().skip(2).collect()).await,
        Some("-v") | Some("--version") => println!("{}", env!("CARGO_PKG_VERSION")),
        _ => println!(
            "{} {}\n{}",
            env!("CARGO_BIN_NAME"),
            env!("CARGO_PKG_VERSION"),
            USAGE
        ),
    }
}
