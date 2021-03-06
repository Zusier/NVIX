use std::{error::Error, io::Result, io::Write, path::PathBuf};

use clap::Parser;
use crossterm::style::Stylize;
use once_cell::sync::Lazy;

use crate::nvapi::{xml::XmlGpuEntry, DriverChannels, DriverEdition, DriverPlatform};
mod nvapi;
mod setup;
#[cfg(test)]
mod tests;
#[cfg(feature = "tui")]
mod tui;

static TMP_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = PathBuf::from(std::env::temp_dir());
    path.push("tmp_nvidia.exe");
    path
});
static TMP_EXTRACT_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = PathBuf::from(std::env::temp_dir());
    path.push("NVIX");
    path
});
static TMP_SEVENZIP_FILE: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = PathBuf::from(std::env::temp_dir());
    path.push("tmp_7z.exe");
    path
});

/// A light-weight program to download, strip, tweak, and install a NVIDIA driver
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser, default_value = "false")]
    verbose: bool,
}

#[tokio::main]
/// Planned flow of app:
/// Detect GPU, if not found, prompt user to select one manually.
/// Prompt for latest driver or specific driver version (user can view available versions if needed)
/// A few prompts about the driver (Notebook, DCH, etc)
/// (future) Strip driver?
/// (future) Tweak driver?
/// install driver or.. (future) package into installer
/// (future) export config file for next time?
async fn main() -> Result<()> {
    let args = Args::parse();

    interactive_mode().await;
    Ok(())
}

async fn interactive_mode() {
    #[cfg(target_os = "windows")]
    let gpu = nvapi::detect_gpu().await;
    #[cfg(target_os = "windows")]
    let gpu: XmlGpuEntry = match gpu {
        Ok(gpu) => {
            println!("Detected GPU: {}", gpu.clone().green());
            if choice("Is this correct?") {
                let list = nvapi::xml::get_gpu_list().await;
                let gpu: XmlGpuEntry = list
                    .unwrap()
                    .iter()
                    .find(|g| g.name == gpu)
                    .unwrap()
                    .clone();
                gpu
            } else {
                tui::gpu_selector()
                    .await
                    .unwrap()
                    .expect("GPU not selected, ui closed.")
            }
        }
        Err(_) => {
            println!("Detected GPU: {}", "Unknown".red());
            println!("No GPU detected, please specify a GPU manually...");
            std::thread::sleep(std::time::Duration::from_secs(2));
            tui::gpu_selector()
                .await
                .unwrap()
                .expect("GPU not selected, ui closed.")
        }
    };
    #[cfg(not(target_os = "windows"))]
    let gpu = tui::gpu_selector()
        .await
        .unwrap()
        .expect("GPU not selected, ui closed.");
    clear();
    println!("GPU Selected: {}", gpu.name.as_str().green()); // Not sure why adding terminal color requires a borrow but ok.
    let latest = choice("Use the latest driver or choose manually?");
    let channel = if choice("Use Game Ready or Studio driver?") {
        DriverChannels::GameReady
    } else {
        DriverChannels::Studio
    };
    let platform = if choice("Desktop or Mobile GPU?") {
        DriverPlatform::Desktop
    } else {
        DriverPlatform::Notebook
    };
    let edition = if choice("Use DCH (preferred) or Standard?") {
        DriverEdition::DCH
    } else {
        DriverEdition::STD
    };
    if latest {
        let driver = nvapi::Driver {
            version: "".to_string(),
            channel,
            platform,
            edition,
        };
        nvapi::download(nvapi::get_latest_driver_link(gpu, driver).await.unwrap())
            .await
            .unwrap();
    } else {
        println!("Enter your driver version: ");
        let mut input = String::new();
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        let driver = nvapi::Driver {
            version: input,
            channel,
            platform,
            edition,
        };
        nvapi::download(nvapi::new_link(&driver).await.unwrap()[0].clone())
            .await
            .unwrap(); // TODO: if no valid links exist.. use latest driver as fallback.
    }
    nvapi::extract().await.expect("Well shit");
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
    clear_ln();
    choice(prompt)
}

#[inline(always)]
/// Clears the terminal and resets cursor to top left
fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}

#[inline(always)]
/// Clears the current line
fn clear_ln() {
    print!("\x1b[1A\x1b[K");
}
