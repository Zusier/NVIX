//! Anything related to the setup of the driver, inlcuding stripping and tweaking.

use std::path::PathBuf;
use once_cell::sync::Lazy;

static COMPONENTS: Lazy<Vec<Component>> = Lazy::new(|| {
    let mut comps: Vec<Component> = Vec::new();

    // Telemetry
    let comp = Component {
        name: "Telemetry".to_string(),
        paths: vec![
                    PathBuf::from(format!("{}\\NvTelemetry", crate::TMP_EXTRACT_DIR.display())),
                    PathBuf::from(format!("{}\\NvModuleTracker", crate::TMP_EXTRACT_DIR.display())) // May be required by GFE
        ],
        remove: None,
    };
    comps.push(comp);


    // GeForce Experience
    let comp = Component {
        name: "GeForce Experience".to_string(),
        paths: vec![
            PathBuf::from(format!("{}\\GFExperience", crate::TMP_EXTRACT_DIR.display())),
            PathBuf::from(format!("{}\\GFExperience.NvStreamSrvi", crate::TMP_EXTRACT_DIR.display())),
            PathBuf::from(format!("{}\\ShadowPlay", crate::TMP_EXTRACT_DIR.display())), // Perhaps we can split this in the future? Check if it depends on GFE
            PathBuf::from(format!("{}\\ShieldWirelessController", crate::TMP_EXTRACT_DIR.display())), // Perhaps we can split this in the future? Check if it depends on GFE
        ],
        remove: None,
    };
    comps.push(comp);

    // Update System
    let comp = Component {
        name: "Update System".to_string(),
        paths: vec![
            PathBuf::from(format!("{}\\Display.Update", crate::TMP_EXTRACT_DIR.display())), // TODO: Figure out exactly what this is
            PathBuf::from(format!("{}\\Update.Core", crate::TMP_EXTRACT_DIR.display())),  
        ],
        remove: None
    };
    comps.push(comp);

    // FrameView <https://www.nvidia.com/en-us/geforce/news/nvidia-frameview-power-and-performance-benchmarking-app-download>
    let comp = Component {
        name: "FrameView".to_string(),
        paths: vec![
            PathBuf::from(format!("{}\\Display.Update", crate::TMP_EXTRACT_DIR.display())), // TODO: Figure out exactly what this is
            PathBuf::from(format!("{}\\Update.Core", crate::TMP_EXTRACT_DIR.display())),  
        ],
        remove: None
    };
    comps.push(comp);

    // Optimus <https://www.nvidia.com/en-us/geforce/technologies/optimus/technology>
    let comp = Component {
        name: "Optimus".to_string(),
        paths: vec![
            PathBuf::from(format!("{}\\Display.Optimus", crate::TMP_EXTRACT_DIR.display())), // TODO: Figure out exactly what this is
        ],
        remove: None
    };
    comps.push(comp);

    comps
});

    pub struct Component {
        pub name: String,
        pub paths: Vec<PathBuf>, // `String` might not be the best idea, subject to change (likely PathBuf)
        pub remove: Option<bool>,
    }

    pub async fn setup() {
        let command = std::process::Command::new(crate::TMP_FILE.as_path());
    }

    pub async fn strip(components: Vec<Component>) -> std::io::Result<()> {
    for component in components {
        if component.remove == Some(false) {
            break;
        }
        component.paths.iter().for_each(|path| if path.exists() { match path.is_dir() {
            true => {
                std::fs::remove_dir_all(&path).unwrap(); // TODO: Check possible errors and handle if needed, note that errors relating to not being found are already handled above
            }
            false => {
                std::fs::remove_file(&path).unwrap();
            }
        }});
    }

    Ok(())
}
