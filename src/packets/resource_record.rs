use std::collections::HashMap;

use anyhow::Result;
use bitvec::{field::BitField, order::Msb0, view::BitView};

use crate::{bool_u15, concat_packable_bits, unpack_chain};

use super::{
    fqdn::{Label, MDNSFQDN},
    pack::Packable,
    MDNSTYPE,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSResourceRecord {
    pub rr_name: MDNSFQDN,
    pub rr_type: MDNSTYPE,
    pub cache_flush_rr_class: bool_u15,
    pub ttl: u32,
    /// The length of the r_data field in bytes, prior to decompression.
    pub rd_length: u16,
    /// A hacky way to represent a possibly pointer terminated section of data.
    pub r_data: Vec<u8>,
    pub r_data_pointer: Option<Label>,
}

impl MDNSResourceRecord {
    pub fn resolve(&mut self, data: &crate::Data, data_cache: &mut HashMap<usize, String>) {
        self.rr_name.resolve(data, data_cache, None);
        if let Some(ptr) = self.r_data_pointer.clone() {
            self.r_data.extend(
                MDNSFQDN { labels: vec![ptr] }
                    .resolve(data, data_cache, None)
                    .to_string()
                    .as_bytes()
                    .to_vec(),
            );
            self.r_data_pointer = None;
        };
    }
}

impl Packable for MDNSResourceRecord {
    fn pack(&self) -> crate::Data {
        concat_packable_bits![
            self.rr_name,
            self.rr_type,
            self.cache_flush_rr_class,
            self.ttl,
            self.rd_length,
            self.r_data
        ]
    }

    fn unpack(data: &mut crate::Data) -> Result<Self> {
        let (rr_name, rr_type, cache_flush_rr_class, ttl, rd_length) =
            unpack_chain!(data => MDNSFQDN, MDNSTYPE, u16, u32, u16);

        let mut r_data = (
            data.drain(..rd_length as usize * 8)
                .as_bitslice()
                .chunks(8)
                .map(|c| c.load_be())
                .collect::<Vec<u8>>(),
            None,
        );

        if [
            MDNSTYPE::NS,
            MDNSTYPE::CNAME,
            MDNSTYPE::PTR,
            MDNSTYPE::DNAME,
            MDNSTYPE::SOA,
            MDNSTYPE::MX,
            MDNSTYPE::AFSDB,
            MDNSTYPE::KX,
            MDNSTYPE::RP,
            MDNSTYPE::SRV,
            MDNSTYPE::NSEC,
        ]
        .contains(&rr_type)
        {
            let has_pointer = r_data.0[rd_length as usize - 2] & 0b1100_0000 == 0b1100_0000;

            if has_pointer {
                r_data.1 = Some(Label::Pointer(
                    r_data.0[rd_length as usize - 2..rd_length as usize].view_bits::<Msb0>()[2..]
                        .load_be::<u16>(),
                ));
                r_data.0.truncate(rd_length as usize - 2);
            }
        }

        let rr = MDNSResourceRecord {
            rr_name,
            rr_type,
            cache_flush_rr_class,
            ttl,
            rd_length,
            r_data: r_data.0,
            r_data_pointer: r_data.1,
        };

        Ok(rr)
    }
}

/* https://www.rfc-editor.org/rfc/rfc6762.html#section-18.14
    In addition to compressing the *names* of resource records, names
    that appear within the *rdata* of the following rrtypes SHOULD also
    be compressed in all Multicast DNS messages:

        NS, CNAME, PTR, DNAME, SOA, MX, AFSDB, RT, KX, RP, PX, SRV, NSEC

    Until future IETF Standards Action [RFC5226] specifying that names in
    the rdata of other types should be compressed, names that appear
    within the rdata of any type not listed above MUST NOT be compressed.
*/
