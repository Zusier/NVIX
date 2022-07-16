///! All code related to downloading: fetching lists, downloading binaries, etc.

const BASE_LINK: &str = "https://international.download.nvidia.com/Windows/";

pub struct Driver {
    pub version: String,
    pub channel: DriverChannels,
    pub platform: DriverPlatform,
    pub edition: DriverEdition,
    pub winver: DriverWindowsVersion,
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
            DriverChannels::Studio => write!(f, "nsd"),
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
            DriverEdition::DCH => write!(f, "dch"),
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
            DriverWindowsVersion::Win10 => write!(f, "win10"),
            DriverWindowsVersion::Win11 => write!(f, "win10-win11"),
        }
    }
}

pub fn new_link(driver: &Driver) -> Result<String, String> {
    if driver.winver == DriverWindowsVersion::Win11 && driver.edition == DriverEdition::STD {
        return Err(
            "Windows 11 is not compatible with a non-dch version! Please use DCH.".to_string(),
        );
    }

    let version: &str = &driver.version;
    let platform: &str = &driver.platform.to_string();
    let winver: &str = &driver.winver.to_string();

    // Construct link with values that always exist
    let mut link: String =
        format!("{BASE_LINK}{version}/{version}-{platform}-{winver}-64bit-international");

    if driver.channel == DriverChannels::Studio {
        link.push_str("-nsd");
    }
    if driver.edition == DriverEdition::DCH {
        link.push_str("-dch");
    }
    link.push_str("-whql.exe");
    Ok(link)
}
