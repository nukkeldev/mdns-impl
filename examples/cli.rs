use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    time::Duration,
};

use anyhow::Result;
use bitvec::view::BitView;
use clap::*;
use mdns_impl::packets::{pack::Packable, packet::MDNSPacket, response::MDNSResponse, MDNSTYPE};

// MDNS Constants
const MDNS_PORT: u16 = 5353;
const MDNS_MULTICAST_IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_MULTICAST_IPV6: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x00fb);
const MDNS_MULTICAST_SOCKETV4: SocketAddr =
    SocketAddr::new(IpAddr::V4(MDNS_MULTICAST_IPV4), MDNS_PORT);
const MDNS_MULTICAST_SOCKETV6: SocketAddr =
    SocketAddr::new(IpAddr::V6(MDNS_MULTICAST_IPV6), MDNS_PORT);

const DEFAULT_RESPONSE_READ_TIMEOUT: f32 = 3.0;

fn configured_mdns_socket(source: (u32, IpAddr), timeout: Duration) -> Result<UdpSocket> {
    let socket = UdpSocket::bind((source.1, 0))?;

    socket.set_read_timeout(Some(timeout))?;

    match source.1 {
        IpAddr::V4(v4) => socket.join_multicast_v4(&MDNS_MULTICAST_IPV4, &v4),
        IpAddr::V6(_) => socket.join_multicast_v6(&MDNS_MULTICAST_IPV6, source.0),
    }
    .expect("Failed to join multicast group.");

    Ok(socket)
}

fn mdns_query(source: (u32, IpAddr), service_type: &str, timeout: Duration) -> Result<()> {
    let is_ipv6 = source.1.is_ipv6();
    let socket = configured_mdns_socket(source, timeout).expect("Failed to configure mDNS socket.");
    let target_address: SocketAddr = if is_ipv6 {
        MDNS_MULTICAST_SOCKETV6
    } else {
        MDNS_MULTICAST_SOCKETV4
    };

    let packet = MDNSPacket::new(service_type, MDNSTYPE::PTR);

    // Send the packet.
    socket.send_to(&packet.pack().into_vec(), target_address)?;

    // // Receive the responses.
    let mut buf = [0; 1024];
    let mut responses = vec![];

    while let Ok((num_bytes, src)) = socket.recv_from(&mut buf) {
        let mut data = buf[..num_bytes].view_bits().to_bitvec();
        let response = MDNSResponse::unpack(&mut data).expect("Failed to unpack response.");

        print!("Recived a response from {src}");

        match response.get_resource_record_of_type(MDNSTYPE::SRV) {
            Ok(rr) => println!(": '{}'", rr.rr_name.to_string()),
            Err(_) => println!(" with no PTR record."),
        }

        responses.push((src, response));
    }

    if responses.is_empty() {
        println!("No devices publishing '{service_type}' found!");
    }

    Ok(())
}

// CLI Parser

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct MDNSCli {
    /// What type of service to search for, for instance "_http._tcp.local".
    service_type: String,
    /// Max duration to wait for when recieving new .
    #[arg(short, long, default_value_t = DEFAULT_RESPONSE_READ_TIMEOUT)]
    timeout: f32,
}

// ...

fn main() -> Result<()> {
    pretty_env_logger::init();

    let cli = MDNSCli::parse();

    let adapter = get_or_select_ip_address()?;

    mdns_query(
        adapter,
        &cli.service_type,
        Duration::from_secs_f32(cli.timeout),
    )
}

// Network Interface Selection

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
                    !ip.is_loopback() && !ip.is_unspecified() && !ip.is_multicast() && ip.is_ipv4()
                })
        })
        .collect::<Vec<_>>();

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

            Ok((ip.1, ip.2))
        }
    }
}
