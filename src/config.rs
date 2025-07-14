use crate::mac::{resolve_mac_address, validate_mac_address};

use log::{debug, error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NutjobConfig {
    pub log_level: String,
    pub nut: NutConfig,
    pub wol: WakeOnLanConfig,
    pub devices: Vec<DeviceConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NutConfig {
    pub ups_name: String,
    pub host: String,
    pub username: String,
    pub password: String,
    pub polling_interval: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WakeOnLanConfig {
    pub min_battery_percentage: u8,
    pub restore_delay: u16,
    pub device_timeout: u16,
    pub reattempt_delay: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub friendly_name: String,
    pub host: String,
    pub mac_address: String,
}

/// `get_raw_config()` returns the deserialized, unedited version of the configuration file.
fn get_raw_config() -> Result<NutjobConfig, config::ConfigError> {
    let raw_config = config::Config::builder()
        .add_source(config::File::with_name("./config.yaml"))
        .build()
        .unwrap();

    return raw_config.try_deserialize::<NutjobConfig>();
}

/// `get_config` returns the validated configuration file, with MAC addresses resolved.
pub fn get_config() -> NutjobConfig {
    debug!(target: "Config", "Attempting to load configuration file");
    let raw_config = get_raw_config();

    if raw_config.is_err() {
        panic!("Unable to access config file! Make sure it is accessible and formatted correctly!");
    }

    let mut config = raw_config.unwrap();
    info!(target: "Config", "Loaded configuration file successfully");

    config.devices.retain_mut(|device| {
        if device.mac_address == "arp" {
            let resolved_mac_address = resolve_mac_address(&device.host);

            match resolved_mac_address {
                Ok(mac_address) => {
                    device.mac_address = mac_address;
                },
                Err(e) => {
                    error!(target: "Config", "Unable to resolve MAC address for '{}'\n\n{e}", device.friendly_name);

                    return false;
                }
            }
        }
        
        let valid_mac_address = validate_mac_address(&device.mac_address);

        if !valid_mac_address {
            error!(target: "Config", "Invalid MAC address given for '{}'\n\nMake sure the MAC address is formatted correctly.", device.friendly_name);
        }

        return valid_mac_address;
    });

    return config;
}
