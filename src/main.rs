use std::{io::Result, io::Write, path::PathBuf};

use clap::Parser;
use nvapi::{xml::get_gpu_list, detect_gpu};
use once_cell::sync::Lazy;
use slint::{SharedString, ModelRc};

use crate::nvapi::{xml::XmlGpuEntry, DriverChannels, DriverEdition, DriverPlatform};
mod nvapi;
mod setup;
#[cfg(test)]
mod tests;
mod ui;

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
slint::include_modules!();

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

    let orig: Vec<XmlGpuEntry> = get_gpu_list().await.unwrap();
    let list: slint::ModelRc<SharedString> = xml_vec_to_slint_vec(&orig.clone(), None);

    let ui = AppWindow::new();
    let ui_weak = ui.as_weak();
    let ui_weak_pages = ui_weak.clone();
    ui.set_list(list);
    ui.on_search(move |search| {
        let ui = ui_weak.upgrade().unwrap();
        ui.set_list(xml_vec_to_slint_vec(&orig, Some(search.clone().as_str())));
    });
    ui.on_page_move(move |page| {
        let ui = ui_weak_pages.upgrade().unwrap();
        println!("On page: {}", page);
        match page {
            1 => {
                println!("{}", ui.get_selection());
            }
            _ => {} 
        }
    });
    ui.run();
    Ok(())
}

/// I fucking love strong types!
fn xml_vec_to_slint_vec(xml: &Vec<XmlGpuEntry>, filter: Option<&str>) -> ModelRc<SharedString> {
        let list: Vec<slint::SharedString> = match filter {
            Some(filter) => {
                xml.iter().filter(|x| x.name.contains(&*filter)).map(|x| slint::SharedString::from(&x.name)).collect()
            }
            None => {
                xml.iter().map(|x| slint::SharedString::from(&x.name)).collect()
            }
        };
        let list: slint::VecModel<slint::SharedString> = list.into();
        let list: slint::ModelRc<slint::SharedString> = slint::ModelRc::new(list);

        list
}
