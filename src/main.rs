use std::error::Error;

use clap::Parser;
mod nvapi;
#[cfg(test)]
mod tests;
mod tui;

/// A light-weight program to download, strip, tweak, and install a NVIDIA driver
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser, default_value = "false")]
    verbose: bool,

    #[clap(long, value_parser, default_value = "false")]
    test: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    Ok(())
}
