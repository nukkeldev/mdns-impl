use anyhow::Result;
use bitvec::vec::BitVec;

use crate::{
    concat_bits,
    pack::{BoolU15, Packable},
    unpack_chain,
};

use super::{fqdn::MDNSFQDN, MDNSTYPE};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSResourceRecord {
    rr_name: MDNSFQDN,
    rr_type: MDNSTYPE,
    cache_flush_rr_class: BoolU15,
    ttl: u32,
    rd_length: u16,
    r_data: Vec<u8>,
}

impl Packable for MDNSResourceRecord {
    fn pack(&self) -> BitVec<u8> {
        concat_bits![
            self.rr_name.pack(),
            self.rr_type.pack(),
            self.cache_flush_rr_class.pack(),
            self.ttl.to_be_bytes().to_vec(),
            self.rd_length.to_be_bytes().to_vec(),
            self.r_data.clone()
        ]
    }

    fn unpack(data: &mut BitVec<u8>) -> Result<Self> {
        let (rr_name, rr_type, cache_flush_rr_class, ttl, rd_length) =
            unpack_chain!(data => MDNSFQDN, MDNSTYPE, BoolU15, u32, u16);

        let r_data = data
            .drain(..rd_length as usize * 8)
            .as_bitslice()
            .to_bitvec()
            .into_vec();

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
