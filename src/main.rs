mod config;
use config::get_config;
mod monitoring;
mod state;
use monitoring::{get_ups_status, is_device_online};
mod wakeonlan;
use wakeonlan::wakeonlan;
mod mac;

use log::{LevelFilter, debug, error, info, warn};

use std::io::{Error, ErrorKind};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

use crate::monitoring::UPSStatus;
use crate::state::{
    DeviceState, NutjobState, get_state, init_state, mark_device_online, mark_online_devices,
    save_state, update_ups_state,
};

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

    if init_state(&config.devices).is_err() {
        panic!("Unable to initialize state management!");
    }

    let initial_state = get_state();

    // Check if UPS was on battery prior to FSD
    let mut on_battery = false;
    let mut restoring = false;
    let mut restoration_start: Option<SystemTime> = None;
    if initial_state.ups.currently_on_battery {
        info!("Service is restoring from a UPS outage");
        restoring = true;
    }

    let interval = Duration::from_secs(config.nut.polling_interval.into());
    let mut next_time = Instant::now() + interval;

    loop {
        let _ups_status = get_ups_status(
            &config.nut.ups_name,
            &config.nut.host,
            &config.nut.username,
            &config.nut.password,
        );

        if _ups_status.is_err() {
            error!(target: "UPS", "Unable to establish connection to {}@{}", config.nut.ups_name, config.nut.host);
            continue;
        }

        let ups_status = _ups_status.unwrap();

        let UPSStatus {
            currently_on_battery: ups_currently_on_battery,
            battery_percentage: ups_battery_percentage,
            load_percentage: ups_load_percentage,
        } = ups_status;
        let _ = update_ups_state(ups_status);

        debug!(target: "UPS", "UPS Status: {} | UPS Load: {ups_load_percentage}% | UPS Battery: {ups_battery_percentage}%", if ups_currently_on_battery { "ON BAT" } else { "ONLINE" });

        // Check if devices are online
        for device in &config.devices {
            let _ =
                mark_device_online(device.friendly_name.clone(), is_device_online(&device.host));
        }

        if ups_currently_on_battery {
            let _ = mark_online_devices();

            if !on_battery {
                info!(target: "UPS", "UPS switched to battery power");
            }

            on_battery = true;
        } else if restoring || (on_battery && !ups_currently_on_battery) {
            restoring = true;
            on_battery = false;

            if restoration_start.is_none() {
                restoration_start = Some(SystemTime::now());
                info!(target: "UPS", "UPS switched to AC power");
            }

            let restoration_time_elapsed = restoration_start.unwrap().elapsed().unwrap();

            if restoration_time_elapsed < Duration::from_secs(config.wol.restore_delay.into()) {
                warn!(
                    "Waiting {} more seconds before waking devices",
                    (config.wol.restore_delay as u64) - restoration_time_elapsed.as_secs()
                );
            } else if ups_battery_percentage < config.wol.min_battery_percentage {
                warn!(
                    "Waiting for battery to reach minimum percentage before waking devices ({}%/{}%)",
                    ups_battery_percentage, config.wol.min_battery_percentage
                );
            } else {
            }
        }

        let _ = save_state();

        sleep(next_time - Instant::now());
        next_time += interval;
    }
}
