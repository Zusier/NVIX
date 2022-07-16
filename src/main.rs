use clap::Parser;

mod dl;

/// A light-weight program to download, strip, tweak, and install a NVIDIA driver
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser, default_value = "false")]
    verbose: bool,

    #[clap(long, value_parser, default_value = "false")]
    test: bool,
}

fn main() {
    let args = Args::parse();
}