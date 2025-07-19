use regex::Regex;
use std::io::{Error, ErrorKind};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

/// The `validate_mac_address` function takes in a string and makes sure that it is structured like a MAC address
///
/// It returns a bool value depending upon if it matches the regex pattern
///
/// Allowed formats:
/// - f6:2e:3c:67:f1:74
/// - F6:2E:3C:67:F1:74
/// - f6-2e-3c-67-f1-74
/// - F6-2E-3C-67-F1-74
pub fn validate_mac_address(mac: &str) -> bool {
    let mac_pattern: Regex = Regex::new(r"^([0-9A-Fa-f]{2}[:\-]){5}([0-9A-Fa-f]{2})$").unwrap();

    return mac_pattern.is_match(mac);
}

/// The `resolve_mac_address` function takes in a host string (either an IPv4 address or a resolvable hostname such as `"server.local"`)
///
/// It attempts to return the MAC address as the `Ok` value in the `Result`, otherwise it will return an `Error` with a message
pub fn resolve_mac_address(host: &str) -> Result<String, Error> {
    // Run ping command to cache the host/IP
    // ARP may be unable to lookup the MAC address if this is not done
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
        return Err(ping_output.unwrap_err());
    }

    let ping_failed = ping_output.unwrap().status.code() != Some(0);

    if ping_failed {
        return Err(Error::new(
            ErrorKind::HostUnreachable,
            format!("Failed to ping {host}"),
        ));
    }

    // Lookup MAC address
    #[cfg(target_os = "windows")]
    let arp_output = Command::new("cmd")
        .stdout(Stdio::piped())
        .arg("/C")
        .raw_arg(format!(
            r#"for /f "tokens=2" %a in ('arp -a ^| findstr {host}') do @echo %a"#
        ))
        .output();

    #[cfg(not(target_os = "windows"))]
    let arp_output = Command::new("sh")
        .stdout(Stdio::piped())
        .arg("-c")
        .arg(format!("arp -n {host}"))
        .arg("| awk '/^[0-9]/ { print $3 }'")
        .output();

    if arp_output.is_err() {
        return Err(arp_output.unwrap_err());
    }

    let arp_unwraped = arp_output.unwrap();
    let arp_failed = arp_unwraped.status.code() != Some(0);

    if arp_failed {
        return Err(Error::new(
            ErrorKind::HostUnreachable,
            String::from_utf8(arp_unwraped.stderr).unwrap(),
        ));
    }

    return Ok(String::from_utf8(arp_unwraped.stdout).unwrap());
}
