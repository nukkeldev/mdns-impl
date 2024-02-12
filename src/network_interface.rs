use std::net::IpAddr;

use anyhow::Result;

use log::debug;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

/// Gets the index and chosen IP address of a network interface usable for mDNS queries.
pub fn get_or_select_ip_address() -> Result<(u32, IpAddr)> {
    let interfaces = NetworkInterface::show().expect("Failed to list network interfaces.");

    let usable_addresses = interfaces
        .iter()
        .enumerate()
        .flat_map(|(i, interface)| {
            interface
                .addr
                .iter()
                .map(move |a| (i, interface.index, a.ip()))
                .filter(|(_, _, ip)| {
                    !ip.is_loopback() && !ip.is_unspecified() && !ip.is_multicast()
                })
        })
        .collect::<Vec<_>>();

    debug!(
        "Usable addresses: {:#?}",
        usable_addresses
            .iter()
            .map(|(_, _, ip)| ip)
            .collect::<Vec<_>>()
    );

    match usable_addresses[..] {
        [] => Err(anyhow::anyhow!(
            "No usable network interfaces/addresses found."
        )),
        [(_, i, ip)] => Ok((i, ip)),
        _ => {
            println!("Select an IP address to use by entering the corresponding index:");
            for (i, (ux, _, ip)) in usable_addresses.iter().enumerate() {
                println!("{} [{}]: {}", i, interfaces[*ux as usize].name, ip);
            }

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            let index = input.trim().parse::<usize>().expect("Invalid input.");
            let ip = usable_addresses.get(index).expect("Invalid index.");

            debug!("Selected network interface: {:#?}", interfaces[ip.0]);
            debug!("Selected IP address: {:#?}", ip.2);

            Ok((ip.1, ip.2))
        }
    }
}
