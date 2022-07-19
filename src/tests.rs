use crate::nvapi::{self, Driver};

// Allow for async to be used in tests
macro_rules! bo {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

fn test_links(driver: &Driver) -> Vec<String> {
    let links = nvapi::new_link(&driver).unwrap();

    let mut valid: Vec<String> = Vec::new();

    for link in links {
        let result = bo!(nvapi::check_link(&link));
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
        channel: nvapi::DriverChannels::GameReady,
        platform: nvapi::DriverPlatform::Desktop,
        edition: nvapi::DriverEdition::DCH,
    };

    let valid = test_links(&driver);

    assert!(valid.len() > 0);
}

#[test]
fn test_link_notebook_studio() {
    let driver = Driver {
        version: "516.59".to_string(),
        channel: nvapi::DriverChannels::Studio,
        platform: nvapi::DriverPlatform::Notebook,
        edition: nvapi::DriverEdition::DCH,
    };

    let valid = test_links(&driver);

    assert!(valid.len() > 0);
}

#[test]
fn test_link_old_std() {
    let driver = Driver {
        version: "441.41".to_string(),
        channel: nvapi::DriverChannels::GameReady,
        platform: nvapi::DriverPlatform::Desktop,
        edition: nvapi::DriverEdition::STD,
    };

    let valid = test_links(&driver);

    assert!(valid.len() > 0);
}

#[test]
fn test_gpu_list() {
    let gpus = bo!(nvapi::xml::get_gpu_list()).unwrap();

    assert!(gpus.len() > 0);
    for gpu in gpus {
        if gpu.name.contains("GeForce RTX 3090 Ti") {
            assert_eq!(gpu.series, 120);
            assert_eq!(gpu.id, 985);
        }
    }
}
