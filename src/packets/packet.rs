use log::debug;

use crate::{pack::Packable, pack_chain, util::read_vec_of_t};

use super::{header::MDNSHeader, query::MDNSQuery, MDNSTYPE};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSPacket {
    header: MDNSHeader,
    queries: Vec<MDNSQuery>,
}

impl MDNSPacket {
    pub fn new(name: &'static str, qtype: MDNSTYPE) -> Self {
        let header = MDNSHeader::new();
        let query = MDNSQuery::new(name, qtype);
        MDNSPacket {
            header,
            queries: vec![query],
        }
    }
}

impl Packable for MDNSPacket {
    fn pack(&self) -> Vec<u8> {
        pack_chain!(self.header, self.queries)
    }

    fn unpack(data: &[u8], offset: usize) -> anyhow::Result<(usize, Self)> {
        let (offset, header) = MDNSHeader::unpack(data, offset)?;
        let (offset, queries) = read_vec_of_t(data, offset, header.questions as usize)?;

        let packet = MDNSPacket { header, queries };

        debug!("Unpacked MDNSPacket: {packet:#?}");

        Ok((offset, packet))
    }
}

#[cfg(test)]
mod tests {
    use crate::{concat_slices, concat_slices_to_bytes};

    use super::*;

    #[test]
    fn test_mdns_packet() {
        let known_packet = {
            // Header
            let header = {
                let transaction_id: u16 = 0x1234;
                let flags: u16 = 0b0000_0000_0000_0000;
                let questions: u16 = 0x0001;
                let answer_rrs: u16 = 0x0000;
                let authority_rrs: u16 = 0x0000;
                let additional_rrs: u16 = 0x0000;

                concat_slices_to_bytes![
                    transaction_id,
                    flags,
                    questions,
                    answer_rrs,
                    authority_rrs,
                    additional_rrs
                ]
            };

            // Query
            let query = {
                let qname = b"\x05_http\x04_tcp\x05local\x00";
                let qtype: u16 = 0x000c; // PTR
                let unicast_response: u16 = 1u16 << 15;
                let qclass: u16 = 0x0001; // IN

                concat_slices![
                    qname,
                    qtype.to_be_bytes(),
                    (unicast_response | qclass).to_be_bytes()
                ]
            };
            concat_slices![header, query]
        };

        let mut packet = MDNSPacket::new("_http._tcp.local", MDNSTYPE::PTR);
        packet.header.transaction_id = 0x1234;

        assert_eq!(packet.pack(), known_packet)
    }
}
