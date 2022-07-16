use crate::dl::{self, Driver};

#[test]
fn test_link_generation_validation() {
    let driver = Driver {
        version: "516.59".to_string(),
        channel: dl::DriverChannels::GameReady,
        platform: dl::DriverPlatform::Desktop,
        edition: dl::DriverEdition::DCH,
    };
    let links = dl::new_link(&driver).unwrap();

    let mut valid: Vec<String> = Vec::new();

    for link in links {
        let result = dl::check_link(&link);
        if result.is_ok() {
            valid.push(link);
        }
    }
	assert!(valid.len() > 0);
}