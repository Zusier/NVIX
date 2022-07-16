use crate::dl::{self, Driver};

fn test_links(driver: &Driver) -> Vec<String> {
    let links = dl::new_link(&driver).unwrap();

    let mut valid: Vec<String> = Vec::new();

    for link in links {
        let result = dl::check_link(&link);
        if result.is_ok() {
            valid.push(link);
        }
    }
    valid
}

#[test]
fn test_link_generation_validation() {
    let driver = Driver {
        version: "516.59".to_string(),
        channel: dl::DriverChannels::GameReady,
        platform: dl::DriverPlatform::Desktop,
        edition: dl::DriverEdition::DCH,
    };

    let valid = test_links(&driver);

	assert!(valid.len() > 0);
}

#[test]
fn test_link_notebook_studio() {
    let driver = Driver {
        version: "516.59".to_string(),
        channel: dl::DriverChannels::Studio,
        platform: dl::DriverPlatform::Notebook,
        edition: dl::DriverEdition::DCH,
    };

    let valid = test_links(&driver);

	assert!(valid.len() > 0);
}

#[test]
fn test_link_old_std() {
    let driver = Driver {
        version: "441.41".to_string(),
        channel: dl::DriverChannels::GameReady,
        platform: dl::DriverPlatform::Desktop,
        edition: dl::DriverEdition::STD,
    };

    let valid = test_links(&driver);

	assert!(valid.len() > 0);
}
