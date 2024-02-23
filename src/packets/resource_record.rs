use std::collections::HashMap;

use anyhow::Result;
use bitvec::{field::BitField, order::Msb0, view::BitView};
use log::debug;

use crate::{
    concat_bits,
    pack::{BoolU15, Packable},
    unpack_chain,
};

use super::{
    fqdn::{Label, MDNSFQDN},
    MDNSTYPE,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSResourceRecord {
    rr_name: MDNSFQDN,
    rr_type: MDNSTYPE,
    cache_flush_rr_class: BoolU15,
    ttl: u32,
    /// The length of the r_data field in bytes, prior to decompression.
    rd_length: u16,
    /// A hacky way to represent a possibly pointer terminated section of data.
    r_data: (Vec<u8>, Option<Label>),
}

impl MDNSResourceRecord {
    pub fn resolve(&mut self, data: &crate::BitVec, data_cache: &mut HashMap<usize, String>) {
        self.rr_name.resolve(data, data_cache, None);
        if let Some(ptr) = self.r_data.1.clone() {
            self.r_data.0.extend(
                MDNSFQDN { labels: vec![ptr] }
                    .resolve(data, data_cache, None)
                    .to_string()
                    .as_bytes()
                    .to_vec(),
            );
            self.r_data.1 = None;
        };
    }
}

impl Packable for MDNSResourceRecord {
    fn pack(&self) -> crate::BitVec {
        concat_bits![
            self.rr_name.pack(),
            self.rr_type.pack(),
            self.cache_flush_rr_class.pack(),
            self.ttl.to_be_bytes().to_vec(),
            self.rd_length.to_be_bytes().to_vec(),
            self.r_data.0.clone()
        ]
    }

    fn unpack(data: &mut crate::BitVec) -> Result<Self> {
        let (rr_name, rr_type, cache_flush_rr_class, ttl, rd_length) =
            unpack_chain!(data => MDNSFQDN, MDNSTYPE, BoolU15, u32, u16);

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

        debug!("Unpacked RR: {:?}", r_data);

        let rr = MDNSResourceRecord {
            rr_name,
            rr_type,
            cache_flush_rr_class,
            ttl,
            rd_length,
            r_data,
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
