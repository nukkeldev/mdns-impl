use anyhow::Result;
use log::debug;

use crate::{
    concat_slices,
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
    fn pack(&self) -> Vec<u8> {
        concat_slices![
            self.rr_name.pack(),
            self.rr_type.pack(),
            self.cache_flush_rr_class.pack(),
            self.ttl.to_be_bytes().to_vec(),
            self.rd_length.to_be_bytes().to_vec(),
            self.r_data.clone()
        ]
    }

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let (data, (rr_name, rr_type, cache_flush_rr_class, ttl, rd_length)) =
            unpack_chain!(data => MDNSFQDN, MDNSTYPE, BoolU15, u32, u16);

        let r_data = data[..rd_length as usize].to_vec();

        let rr = MDNSResourceRecord {
            rr_name,
            rr_type,
            cache_flush_rr_class,
            ttl,
            rd_length,
            r_data,
        };

        debug!("Unpacked MDNSResourceRecord: {rr:#?}");

        Ok((&data[rd_length as usize..], rr))
    }
}
