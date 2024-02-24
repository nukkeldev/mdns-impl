use std::collections::HashMap;

use anyhow::Result;

use crate::{bool_u15, concat_packable_bits, unpack_chain};

use super::{fqdn::MDNSFQDN, pack::Packable, MDNSTYPE};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSQuery {
    pub qname: MDNSFQDN,
    pub qtype: MDNSTYPE,
    pub qu_qclass: bool_u15,
}

impl MDNSQuery {
    pub fn new(name: &str, qtype: MDNSTYPE) -> Self {
        let qname = MDNSFQDN::new(name);
        let qu_qclass = (1 << 15) | 1;
        MDNSQuery {
            qname,
            qtype,
            qu_qclass,
        }
    }

    pub fn resolve(&mut self, data: &crate::Data, data_cache: &mut HashMap<usize, String>) {
        self.qname.resolve(data, data_cache, None);
    }
}

impl Packable for MDNSQuery {
    fn pack(&self) -> crate::Data {
        concat_packable_bits![self.qname, self.qtype, self.qu_qclass]
    }

    fn unpack(data: &mut crate::Data) -> Result<Self> {
        let (qname, qtype, qu_qclass) = unpack_chain!(data => MDNSFQDN, MDNSTYPE, u16);

        let query = MDNSQuery {
            qname,
            qtype,
            qu_qclass,
        };

        Ok(query)
    }
}
