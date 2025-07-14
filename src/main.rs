mod config;
use config::get_config;
mod monitoring;
use monitoring::{get_ups_status, is_device_online};
mod wakeonlan;
// use wakeonlan::wakeonlan;
mod mac;

use log::{LevelFilter, debug, error, info};

use std::io::{Error, ErrorKind};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::monitoring::UPSStatus;

fn string_to_level_filter(log_level: &String) -> Result<LevelFilter, Error> {
    return match log_level.to_lowercase().as_str() {
        "off" => Ok(LevelFilter::Off),
        "trace" => Ok(LevelFilter::Trace),
        "debug" => Ok(LevelFilter::Debug),
        "info" => Ok(LevelFilter::Info),
        "warn" => Ok(LevelFilter::Warn),
        "error" => Ok(LevelFilter::Error),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid log level provided",
        )),
    };
}

fn main() {
    simple_logger::init().unwrap();

    let config = get_config();

    log::set_max_level(string_to_level_filter(&config.log_level).unwrap_or(LevelFilter::Trace));

    let interval = Duration::from_secs(config.nut.polling_interval.into());
    let mut next_time = Instant::now() + interval;

    loop {
        let ups_status = get_ups_status(
            &config.nut.ups_name,
            &config.nut.host,
            &config.nut.username,
            &config.nut.password,
        );

        if ups_status.is_err() {
            error!(target: "UPS", "Unable to establish connection to {}@{}", config.nut.ups_name, config.nut.host)
        } else {
            let UPSStatus {
                is_on_battery: ups_is_on_battery,
                battery_percentage: ups_battery_percentage,
                load_percentage: ups_load_percentage,
            } = ups_status.unwrap();
            debug!(target: "UPS", "UPS Status: {} | UPS Load: {ups_load_percentage}% | UPS Battery: {ups_battery_percentage}%", if ups_is_on_battery { "ON BAT" } else { "ONLINE" })
        }

        sleep(next_time - Instant::now());
        next_time += interval;
    }
}
