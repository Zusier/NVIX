//! # NVIDIA API
//! This module contains actions related to the NVIDIA API. Not to be confused with NVIDIA's driver api.
//! Reference: <https://github.com/fyr77/EnvyUpdate/wiki/Nvidia-API>

use std::error::Error;

const BASE_LINK: &str = "https://international.download.nvidia.com/Windows";

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

pub fn new_link(driver: &Driver) -> Result<Vec<String>, Box<dyn Error>> {
    let version: &str = &driver.version;
    let platform: &str = &driver.platform.to_string();
    let channel: &str = &driver.channel.to_string();
    let edition: &str = &driver.edition.to_string();

    // Construct link with values that always exist
    let links: Vec<String> = DriverWindowsVersion::iter().map(|winver| {
        let link: String = format!("{BASE_LINK}/{version}/{version}-{platform}{winver}-64bit-international{channel}{edition}-whql.exe");
        link
    }).collect();

    Ok(links)
}

pub async fn check_link(link: &str) -> Result<(), Box<dyn Error>> {
    // Check if link exists
    let resp = reqwest::get(link).await?;
    if resp.status().is_success() {
        return Ok(());
    } else {
        return Err(resp.status().to_string().into());
    }
}

pub mod xml {
    use std::error::Error;

    use serde::Deserialize;
    #[derive(Debug, PartialEq, Eq, Clone)]
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

    pub async fn get_gpu_list() -> Result<Vec<XmlGpuEntry>, Box<dyn Error>> {
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
