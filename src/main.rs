use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    time::Duration,
};

use anyhow::Result;
use log::debug;
use mdns_browser::{
    network_interface::get_or_select_ip_address,
    pack::Packable,
    packets::{packet::MDNSPacket, query::MDNSQuery, response::MDNSResponse, MDNSTYPE},
};

const MDNS_PORT: u16 = 5353;
const MDNS_MULTICAST_IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_MULTICAST_IPV6: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x00fb);
const MDNS_MULTICAST_SOCKETV4: SocketAddr =
    SocketAddr::new(IpAddr::V4(MDNS_MULTICAST_IPV4), MDNS_PORT);
const MDNS_MULTICAST_SOCKETV6: SocketAddr =
    SocketAddr::new(IpAddr::V6(MDNS_MULTICAST_IPV6), MDNS_PORT);

const QUERY_WRITE_TIMEOUT: Duration = Duration::from_secs(20);
const RESPONSE_READ_TIMEOUT: Duration = Duration::from_secs(20);

fn configured_mdns_socket(source: (u32, IpAddr)) -> Result<UdpSocket> {
    let socket = UdpSocket::bind((source.1, 0))?;

    socket.set_read_timeout(Some(RESPONSE_READ_TIMEOUT))?;
    socket.set_write_timeout(Some(QUERY_WRITE_TIMEOUT))?;

    match source.1 {
        IpAddr::V4(v4) => socket.join_multicast_v4(&MDNS_MULTICAST_IPV4, &v4),
        IpAddr::V6(_) => socket.join_multicast_v6(&MDNS_MULTICAST_IPV6, source.0),
    }
    .expect("Failed to join multicast group.");

    Ok(socket)
}

fn oneshot_mdns_query(source: (u32, IpAddr)) -> Result<()> {
    let is_ipv6 = source.1.is_ipv6();
    let socket = configured_mdns_socket(source).expect("Failed to configure mDNS socket.");
    let target_address: SocketAddr = if is_ipv6 {
        MDNS_MULTICAST_SOCKETV6
    } else {
        MDNS_MULTICAST_SOCKETV4
    };

    let packet = MDNSPacket::new("_http._tcp.local", MDNSTYPE::PTR);

    debug!(
        "Sending mDNS query from {} to {}",
        socket.local_addr()?,
        target_address
    );

    // Send the packet.
    socket.send_to(&packet.pack(), target_address)?;

    // // Receive the response.
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((num_bytes, src_addr)) => {
            debug!("Received {} bytes from {}", num_bytes, src_addr);
            let (data, response) =
                MDNSResponse::unpack(&buf[..num_bytes]).expect("Failed to unpack response.");
            debug!("Response: {:#?}", response);
        }
        Err(e) => {
            eprintln!("Failed to receive response: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let ip = get_or_select_ip_address().expect("Failed to get IP address!");
    oneshot_mdns_query(ip)?;

    Ok(())
}
