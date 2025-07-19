use log::{debug, error, info};
use std::io::Result;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub fn wakeonlan(mac: &str, friendly_name: &str) -> Result<()> {
    let mac_address = wol::MacAddr6::from_str(mac).unwrap();

    info!(target: "WoL", "Attempting to wake {friendly_name}");
    debug!(target: "WoL", "Bounded {mac} with {friendly_name}");
    let wol_result = wol::send_magic_packet(mac_address, None, (Ipv4Addr::BROADCAST, 9).into());

    match wol_result {
        Ok(()) => {
            info!(target: "WoL", "Sent packet to {friendly_name} successfully");
            return Ok(());
        }
        Err(e) => {
            error!(target: "WoL", "Failed to send packet to {friendly_name}: {e}");
            return Err(e);
        }
    }
}
