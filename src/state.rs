use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Result, Write},
    path::Path,
    sync::Mutex,
};

use bincode::{Decode, Encode, config};

use crate::{config::DeviceConfig, monitoring::UPSStatus};

#[derive(Encode, Decode, Debug, Clone)]
pub struct NutjobState {
    pub ups: UPSStatus,
    pub devices: Vec<DeviceState>,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct DeviceState {
    pub friendly_name: String,
    pub online_before_shutdown: bool,
    pub online: bool,
    pub wol_sent: bool,
}

static STATE_PATH: &'static str = "/nutjob/state";

static STATE: Mutex<NutjobState> = Mutex::new(NutjobState {
    ups: UPSStatus {
        currently_on_battery: false,
        battery_percentage: 100,
        load_percentage: 0,
    },
    devices: Vec::new(),
});

pub fn read_vector(path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    return Ok(data);
}

pub fn read_state_from_file() -> NutjobState {
    let config = config::standard();

    let vector = read_vector(Path::new(STATE_PATH)).unwrap_or_default();

    let (decoded, _len): (NutjobState, usize) = bincode::decode_from_slice(&vector, config)
        .unwrap_or((
            NutjobState {
                ups: UPSStatus {
                    currently_on_battery: false,
                    battery_percentage: 100,
                    load_percentage: 0,
                },
                devices: Vec::new(),
            },
            0,
        ));

    return decoded;
}

pub fn init_state(device_configs: &Vec<DeviceConfig>) -> Result<()> {
    let mut state = read_state_from_file();

    state.devices = device_configs
        .into_iter()
        .map(|device| DeviceState {
            friendly_name: device.friendly_name.clone(),
            online_before_shutdown: false,
            online: false,
            wol_sent: false,
        })
        .collect();

    update_state(state)?;

    return save_state();
}

pub fn get_state() -> NutjobState {
    let guard = STATE.lock().unwrap();

    return guard.clone();
}

fn save_vector(path: &Path, encoded: &[u8]) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&encoded[..])?;
    return Ok(());
}

pub fn save_state() -> Result<()> {
    let config = config::standard();

    let guard = STATE.lock().unwrap();
    let encoded = bincode::encode_to_vec(guard.clone(), config).unwrap();

    return save_vector(Path::new(STATE_PATH), &encoded);
}

pub fn update_state(new_state: NutjobState) -> Result<()> {
    let mut guard = STATE.lock().unwrap();

    *guard = new_state;

    return Ok(());
}

pub fn update_device_state(new_device_state: DeviceState) -> Result<()> {
    let state = get_state();

    return update_state(NutjobState {
        ups: state.ups,
        devices: state
            .devices
            .clone()
            .into_iter()
            .map(|device| {
                if device.friendly_name == new_device_state.friendly_name {
                    new_device_state.clone()
                } else {
                    device
                }
            })
            .collect(),
    });
}

pub fn mark_device_online(friendly_name: String, online: bool) -> Result<()> {
    let state = get_state();

    let _device = state
        .devices
        .into_iter()
        .find(|device| device.friendly_name == friendly_name);

    match _device {
        Some(mut device) => {
            device.online = online;

            return update_device_state(device);
        }
        None => Err(Error::new(ErrorKind::InvalidInput, "Device not found")),
    }
}

pub fn mark_online_devices() -> Result<()> {
    let state = get_state();

    return update_state(NutjobState {
        ups: state.ups,
        devices: state
            .devices
            .clone()
            .into_iter()
            .map(|device| DeviceState {
                friendly_name: device.friendly_name,
                online_before_shutdown: device.online,
                online: device.online,
                wol_sent: device.wol_sent,
            })
            .collect(),
    });
}

pub fn update_ups_state(new_ups_state: UPSStatus) -> Result<()> {
    let state = get_state();

    return update_state(NutjobState {
        ups: new_ups_state.clone(),
        devices: state.devices.clone(),
    });
}
