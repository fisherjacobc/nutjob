use std::convert::TryInto;
use std::process::Command;

use bincode::{Decode, Encode};
use rups::blocking::Connection;
use rups::{Auth, ConfigBuilder};

/// The `is_device_online` function checks to see if a device is "online" by pinging the device. If the exit code is 0 then it returns true, otherwise it returns false.
pub fn is_device_online(host: &str) -> bool {
    #[cfg(target_os = "windows")]
    let ping_output = Command::new("cmd")
        .arg("/C")
        .arg(format!("ping -n 1 {host}"))
        .output();

    #[cfg(not(target_os = "windows"))]
    let ping_output = Command::new("sh")
        .arg("-c")
        .arg(format!("ping -c 1 {host}"))
        .output();

    if ping_output.is_err() {
        return false;
    }

    let ping_success = ping_output.unwrap().status.code() == Some(0);

    return ping_success;
}

#[derive(Encode, Decode, Debug)]
pub struct UPSStatus {
    pub is_on_battery: bool,
    pub battery_percentage: u8,
    pub load_percentage: u8,
}

/// The `get_ups_status` function queries specific information (see [`UPSStatus`]) from the NUT server specified in the config file
pub fn get_ups_status(
    ups_name: &str,
    host: &str,
    username: &str,
    password: &str,
) -> Result<UPSStatus, Box<dyn std::error::Error>> {
    let rsups_config = ConfigBuilder::new()
        .with_host((host.to_string(), 3493).try_into().unwrap_or_default())
        .with_auth(Some(Auth::new(
            username.to_string(),
            Some(password.to_string()),
        )))
        .with_debug(false)
        .build();

    let mut connection = Connection::new(&rsups_config)?;

    let mut is_on_battery = false;
    let mut battery_percentage = 100;
    let mut load_percentage = 0;

    for var in connection.list_vars(ups_name)? {
        if var.name() == "ups.status" {
            is_on_battery = !var.value().contains("OL");
        } else if var.name() == "battery.charge" {
            battery_percentage = var.value().parse::<u8>().unwrap();
        } else if var.name() == "ups.load" {
            load_percentage = var.value().parse::<u8>().unwrap();
        }
    }

    return Ok(UPSStatus {
        is_on_battery: is_on_battery,
        battery_percentage: battery_percentage,
        load_percentage: load_percentage,
    });
}
