use std::{error::Error, io::Write};

use clap::Parser;
mod nvapi;
#[cfg(test)]
mod tests;
#[cfg(feature = "tui")]
mod tui;

/// A light-weight program to download, strip, tweak, and install a NVIDIA driver
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser, default_value = "false")]
    verbose: bool,

    #[clap(long, value_parser, default_value = "false")]
    test: bool,

    #[clap(long, short)]
    interactive: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.interactive {
        interactive_mode().await;
    }
    Ok(())
}


async fn interactive_mode() {
    let gpu_orig = nvapi::detect_gpu().await;
    let gpu: String = match gpu_orig {
        Ok(gpu_orig) =>  {
            println!("Detected GPU: {gpu_orig}");
            if choice("Is this correct?") {
                // query.. driver
            }
            tui::gpu_selector().await.unwrap().unwrap().name
        }
        Err(_) => {
            println!("Detected GPU: Unkonwn");
            println!("No GPU detected, please specify a GPU manually...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            tui::gpu_selector().await.unwrap().unwrap().name
        }
    };
    println!("GPU Selected: {gpu}");
}

/// Prints prompt with a y/n amswer, if it is invalid it will simply clear the prompt and recurse
fn choice(prompt: &str) -> bool {
    print!("{} [y/n] ", prompt);
    let mut input = String::new();
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    input = input.trim().to_string();
    if input.to_lowercase() == "y" {
        return true;
    } else if input.to_lowercase() == "n" {
        return false;
    }
    // Invalid, repeat
    print!("\x1b[1A\x1b[K"); // Clears the line
    choice(prompt)
}


macro_rules! clear {
    () => {
        print!("{}[2J", 27 as char);
    };
}