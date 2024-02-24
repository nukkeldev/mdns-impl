use anyhow::Result;

use super::{pack::Packable, util::read_vec_of_t};

use super::{header::MDNSHeader, query::MDNSQuery, MDNSTYPE};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSPacket {
    header: MDNSHeader,
    queries: Vec<MDNSQuery>,
}

impl MDNSPacket {
    pub fn new(name: &str, qtype: MDNSTYPE) -> Self {
        let header = MDNSHeader::new();
        let query = MDNSQuery::new(name, qtype);
        MDNSPacket {
            header,
            queries: vec![query],
        }
    }
}

impl Packable for MDNSPacket {
    fn pack(&self) -> crate::Data {
        let mut out = self.header.pack();
        out.extend(self.queries.pack());
        out
    }

    fn unpack(data: &mut crate::Data) -> Result<Self> {
        let header = MDNSHeader::unpack(data)?;
        let queries = read_vec_of_t(data, header.questions as usize)?;

        let packet = MDNSPacket { header, queries };

        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use bitvec::{bitvec, field::BitField, order::Msb0, view::BitView};

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

                let mut out = bitvec![u8, Msb0; 0; 96];

                out[00..16].store_be(transaction_id);
                out[16..32].store_be(flags);
                out[32..48].store_be(questions);
                out[48..64].store_be(answer_rrs);
                out[64..80].store_be(authority_rrs);
                out[80..96].store_be(additional_rrs);

                out
            };

            // Query
            let query = {
                let qname = b"\x05_http\x04_tcp\x05local\x00";
                let len = qname.len() * 8 as usize;

                let qtype: u16 = 0x000c; // PTR
                let unicast_response: u16 = 1u16 << 15;
                let qclass: u16 = 0x0001; // IN

                let mut out = bitvec![u8, Msb0; 0; len + 16 * 2];

                out[00..len].copy_from_bitslice(&qname.view_bits::<Msb0>());
                out[len..len + 16].store_be(qtype);
                out[len + 16..len + 32].store_be(unicast_response | qclass);

                out
            };

            let mut packet = header;
            packet.extend(query);

            packet
        };

        let mut packet = MDNSPacket::new("_http._tcp.local", MDNSTYPE::PTR);
        packet.header.transaction_id = 0x1234;

        assert_eq!(packet.pack(), known_packet)
    }
}
