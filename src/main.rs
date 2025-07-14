mod config;
use config::get_config;
mod monitoring;
use monitoring::{get_ups_status, is_device_online};
mod wakeonlan;
// use wakeonlan::wakeonlan;
mod mac;

use log::{debug, error, info};

use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::monitoring::UPSStatus;

fn main() {
    simple_logger::init().unwrap();

    let config = get_config();

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
            debug!(target: "UPS", "UPS Status: {} | UPS Load: {ups_load_percentage}% | UPS Battery: {ups_battery_percentage}%", if ups_is_on_battery { "ON BATTERY" } else { "ONLINE" })
        }

        sleep(next_time - Instant::now());
        next_time += interval;
    }
}
