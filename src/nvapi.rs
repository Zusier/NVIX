//! # NVIDIA API
//! This module contains actions related to th&e NVIDIA API. Not to be confused with NVIDIA's driver api.
//! Reference: <https://github.com/fyr77/EnvyUpdate/wiki/Nvidia-API>

use std::{error::Error, fs::File, io::Cursor};

const BASE_LINK: &str = "https://international.download.nvidia.com";
const PCI_IDS: &str = "https://raw.githubusercontent.com/pciutils/pciids/master/pci.ids";
const SEVENZIP_LINK: &str = "https://www.7-zip.org/a/7zr.exe"; // I can't have a '7' at the start of a constant? lol

static REGEX_VENDOR: Lazy<Regex> = Lazy::new(|| Regex::new("^([0-9a-f]{4})  (.*)$").unwrap());
static REGEX_DEVICE: Lazy<Regex> = Lazy::new(|| Regex::new("^\t([0-9a-f]{4})  (.*)$").unwrap());
//static REGEX_SUBDEVICE: Lazy<Regex> = Lazy::new(|| Regex::new("^\t\t([0-9a-f]{4}) (.*)$").unwrap());

pub struct Driver {
    pub version: String,
    pub channel: DriverChannels,
    pub platform: DriverPlatform,
    pub edition: DriverEdition,
}

#[derive(Default, PartialEq)]
pub enum DriverChannels {
    #[default]
    GameReady,
    Studio,
}

impl std::fmt::Display for DriverChannels {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DriverChannels::GameReady => write!(f, ""),
            DriverChannels::Studio => write!(f, "-nsd"),
        }
    }
}

impl DriverChannels {
    pub fn into_api(self) -> u8 {
        match self {
            DriverChannels::GameReady => 1,
            DriverChannels::Studio => 4,
        }
    }
}

#[derive(Default, PartialEq)]
pub enum DriverPlatform {
    #[default]
    Desktop,
    Notebook, // For mobile GPUs
}

impl std::fmt::Display for DriverPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DriverPlatform::Desktop => write!(f, "desktop"),
            DriverPlatform::Notebook => write!(f, "notebook"),
        }
    }
}

#[derive(Default, PartialEq)]
pub enum DriverEdition {
    #[default]
    DCH, // Desktop Channel, UWP
    STD, // Standard
}

impl std::fmt::Display for DriverEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DriverEdition::DCH => write!(f, "-dch"),
            DriverEdition::STD => write!(f, ""),
        }
    }
}

impl DriverEdition {
    pub fn into_api(self) -> u8 {
        match self {
            DriverEdition::DCH => 1,
            DriverEdition::STD => 0,
        }
    }
}

#[derive(Default, PartialEq)]
pub enum DriverWindowsVersion {
    #[default]
    Win11, // Allows for both 10 and 11
    Win10, // Only allows for 10
}

impl std::fmt::Display for DriverWindowsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DriverWindowsVersion::Win10 => write!(f, "-win10"),
            DriverWindowsVersion::Win11 => write!(f, "-win10-win11"),
        }
    }
}

impl DriverWindowsVersion {
    pub fn iter() -> std::slice::Iter<'static, Self> {
        [Self::Win10, Self::Win11].iter()
    }
}

pub async fn new_link(driver: &Driver) -> Result<Vec<String>, Box<dyn Error>> {
    let mut valid: Vec<String> = Vec::new();
    {
        let version: &str = &driver.version;
        let platform: &str = &driver.platform.to_string();
        let channel: &str = &driver.channel.to_string();
        let edition: &str = &driver.edition.to_string();

        // Construct link with values that always exist
        let links: Vec<String> = DriverWindowsVersion::iter().map(|winver| {
        let link: String = format!("{BASE_LINK}/Windows/{version}/{version}-{platform}{winver}-64bit-international{channel}{edition}-whql.exe");
        link
    }).collect();

        // check links
        for link in links {
            if check_link(link.as_str()).await.is_ok() {
                valid.push(link);
            }
        }
    }

    if valid.is_empty() {
        return Err("No valid links found!".into());
    }
    Ok(valid)
}

pub async fn check_link(link: &str) -> Result<(), Box<dyn Error>> {
    // Check if link exists
    let resp = reqwest::get(link).await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(resp.status().to_string().into())
    }
}

use once_cell::sync::Lazy;
use regex::Regex;
use tokio::join;

use crate::ResultDynError;

use self::xml::XmlGpuEntry;

pub async fn detect_gpu() -> Result<String, Box<dyn Error>> {
    let mut vendor_id = String::new();

    // get pci device id list
    // Hierarchy of pci device id list:
    // Vendor or Generic Device Type (NVIDIA, or Display Adapter) ->
    // Device -> (Optional) SubDevices, Revisions or Vendors (3090 -> 3090 founders edition)
    let (pci_ids, device_id) = join!(reqwest::get(PCI_IDS), crate::nvapi::get_gpu_id());
    let pci_ids = pci_ids?.text().await?;
    let device_id = device_id?;

    for line in pci_ids.lines() {
        // Comments
        if line.starts_with('#') {
            continue;
        }

        // Vendors
        for capture in REGEX_VENDOR.captures_iter(line) {
            vendor_id = capture[1].to_string();
        }

        // Only check for NVIDIA devices
        if vendor_id == "10de" {
            // Devices
            for capture in REGEX_DEVICE.captures_iter(line) {
                if device_id == capture[1] {
                    // remove brackets and irrelevant ids from device name
                    let name: String = capture[2]
                        .split('[')
                        .last()
                        .unwrap()
                        .split(']')
                        .next()
                        .unwrap()
                        .to_string();
                    return Ok(name);
                }
            }
            // SubDevices
            // Commenting for now, until I can get some sample ids to test with
            /*for capture in REGEX_SUBDEVICE.captures_iter(line) {
                if id == capture[1].to_string() {
                    println!("{}", capture[2].to_string());
                    return Ok(capture[2].to_string());
                }
            }*/
        }
    }
    Err("No matching device found".into())
}

pub mod xml {
    use serde::Deserialize;

    use crate::ResultDynError;

    #[derive(Clone, PartialEq)]
    pub struct XmlGpuEntry {
        pub name: String, // e.g. "GeForce RTX 3090 Ti"
        pub series: u16,  // e.g. "120", the ParentID in the XML file
        pub id: u16,      // e.g. "985"
    }

    #[derive(Debug, Deserialize)]
    pub struct LookupValueSearch {
        #[serde(rename = "LookupValues")]
        pub lookupvalues: LookupValues,
    }

    #[derive(Debug, Deserialize)]
    pub struct LookupValues {
        #[serde(rename = "LookupValue")]
        pub lookupvalue: Vec<LookupValue>,
    }

    #[derive(Debug, Deserialize)]
    pub struct LookupValue {
        #[serde(rename = "ParentID")]
        pub parentid: u16,
        #[serde(rename = "Name")]
        pub name: Name,
        #[serde(rename = "Value")]
        pub value: Value,
    }

    #[derive(Debug, Deserialize)]
    pub struct Name {
        #[serde(rename = "$value")]
        pub value: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Value {
        #[serde(rename = "$value")]
        pub value: u16,
    }

    pub async fn get_gpu_list() -> ResultDynError<Vec<XmlGpuEntry>> {
        let xml =
            reqwest::get("https://www.nvidia.com/Download/API/lookupValueSearch.aspx?TypeID=3");
        let deserialized: LookupValueSearch = quick_xml::de::from_str(&xml.await?.text().await?)?;

        let mut gpu_entries: Vec<XmlGpuEntry> = Vec::new();
        for lookupvalue in deserialized.lookupvalues.lookupvalue.iter() {
            let gpu_entry = XmlGpuEntry {
                name: lookupvalue.name.value.clone(),
                series: lookupvalue.parentid,
                id: lookupvalue.value.value,
            };
            gpu_entries.push(gpu_entry);
        }
        Ok(gpu_entries)
    }
}

#[cfg(feature = "wmi")]
pub async fn get_gpu_id() -> ResultDynError<String> {
    use serde::Deserialize;

    let com_connection: wmi::COMLibrary = wmi::COMLibrary::new()?;
    let wmi_connection: wmi::WMIConnection = wmi::WMIConnection::new(com_connection)?;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct Win32_PnPSignedDriver {
        hardware_id: Option<String>,
        device_class: Option<String>,
        device_name: Option<String>,
    }

    let results: Vec<Win32_PnPSignedDriver> = wmi_connection
        .raw_query("SELECT HardwareID, DeviceClass, DeviceName FROM Win32_PnPSignedDriver")
        .unwrap();
    for driver in results {
        // only try and match hardware_id if the device is a GPU
        if driver.device_class == Some("DISPLAY".to_string())
            || driver.device_name == Some("3D Video Controller".to_string())
        {
            if let Some(mut hwid) = driver.hardware_id {
                hwid = hwid.split("DEV_").last().unwrap().to_string();
                let parts: Vec<&str> = hwid.split('&').collect();
                return Ok(parts[0].to_string().to_ascii_lowercase()); // WMI returns an uppercase hwid
            }
        }
    }

    Err("No matching device found".into())
}

#[cfg(feature = "reg")]
pub async fn get_gpu_id() -> Result<String, Box<dyn Error>> {
    let mut device_id = String::new();

    // get device id from registry (if any)
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}", // path for display adapters
    )?;
    let subkeys = key.enum_keys();
    for subkey in subkeys {
        let subkey = subkey.unwrap();
        if subkey.len() == 4 {
            // subkeys for devices are 4 characters long, e.g. "0000" or "0001"
            let subkey = key.open_subkey(subkey)?;
            device_id = subkey.get_value("MatchingDeviceId")?;
            device_id = device_id.split("dev_").last().unwrap().to_string();
            return Ok(device_id);
        }
    }
    Err("No matching device found".into())
}

pub async fn get_latest_driver_link(gpu: XmlGpuEntry, driver: Driver) -> String {
    let psid = gpu.series;
    let pfid = gpu.id;
    let dtcid = driver.edition.into_api(); // 1=dch, 0=std
    let whql = driver.channel.into_api(); // 1 = Game Ready, 4 = Studio

    let link: String = format!("https://www.nvidia.com/Download/processDriver.aspx?psid={psid}&pfid={pfid}&osid=57&lid=1&whql={whql}&dtcid={dtcid}");
    let link = reqwest::get(link).await.unwrap().text().await.unwrap();
    parse_driver_page(link).await
}

/// returns direct link to download
pub async fn parse_driver_page(link: String) -> String {
    let html = reqwest::get(link).await.unwrap().text().await.unwrap();
    let html = html
        .split("?url=")
        .last()
        .unwrap()
        .split('&')
        .next()
        .unwrap();
    format!("{BASE_LINK}{html}")
}

pub async fn download(link: String) -> ResultDynError<()> {
    println!("Downloading driver! Please wait...");
    // TODO: Progress bar?
    let resp = reqwest::get(link).await?;
    let mut file = File::create(crate::TMP_FILE.as_path())?;
    let mut content = Cursor::new(resp.bytes().await?);
    std::io::copy(&mut content, &mut file).unwrap();
    Ok(())
}

// note: I have tried every rust archive library and other workarounds in order to not use external dependencies without luck.. feel free to suggest a different way!
pub async fn extract() -> ResultDynError<()> {
    println!("Extracting driver! Please wait...");

    // download 7z
    {
        let resp = reqwest::get(SEVENZIP_LINK).await?;
        let mut file = File::create(crate::TMP_SEVENZIP_FILE.as_path())?;
        let mut content = Cursor::new(resp.bytes().await?);
        std::io::copy(&mut content, &mut file).unwrap();
    }

    // extract the setup
    let mut command = std::process::Command::new(crate::TMP_SEVENZIP_FILE.as_path());
    command
        .arg("x")
        .arg("-aoa") // overwrite, no prompt
        .arg("-bb0")
        .arg("-bso0")
        .arg("-bse1")
        .arg("-bsp1")
        .arg(crate::TMP_FILE.clone())
        .arg(format!("-o{}", crate::TMP_EXTRACT_DIR.as_path().display()));
    command.spawn().unwrap().wait().unwrap(); // TODO: handle unwraps
    Ok(())
}
