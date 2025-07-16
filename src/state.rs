use std::{
    fs::File,
    io::{Result, Write},
    path::Path,
};

use bincode::{Decode, Encode, config};

use crate::monitoring::UPSStatus;

#[derive(Encode, Decode, Debug)]
pub struct NutjobState {
    pub ups: UPSStatus,
    pub devices: DeviceStatus,
}

#[derive(Encode, Decode, Debug)]
pub struct DeviceStatus {
    pub friendly_name: String,
    pub online_before_shutdown: bool,
    pub online: bool,
    pub wol_sent: bool,
}

fn save_vector(path: &Path, encoded: &[u8]) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&encoded[..])?;
    return Ok(());
}

pub fn save_state(state: &NutjobState) -> Result<()> {
    let config = config::standard();

    let encoded = bincode::encode_to_vec(state, config).unwrap();

    return save_vector(Path::new("/nutjob/state"), &encoded);
}
