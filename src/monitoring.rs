use std::convert::TryInto;
use std::io::{Error, ErrorKind};
use std::process::Command;

use rups::blocking::Connection;
use rups::{Auth, ConfigBuilder};

pub fn is_device_online(host: &str) -> bool {
    let ping_output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(format!("ping -n 1 {host}"))
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!("ping -c 1 {host}"))
            .output()
    };

    if ping_output.is_err() {
        return false;
    }

    let ping_success = ping_output.unwrap().status.code() == Some(0);

    return ping_success;
}

pub struct UPSStatus {
    pub is_on_battery: bool,
    pub battery_percentage: u8,
}

pub fn get_ups_status(
    ups_name: &str,
    host: &str,
    username: &str,
    password: &str,
) -> Result<UPSStatus, Error> {
    let rsups_config = ConfigBuilder::new()
        .with_host((host.to_string(), 3493).try_into().unwrap_or_default())
        .with_auth(Some(Auth::new(
            username.to_string(),
            Some(password.to_string()),
        )))
        .with_debug(false)
        .build();

    return Ok(UPSStatus {
        is_on_battery: false,
        battery_percentage: 100,
    });
}
