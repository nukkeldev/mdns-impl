use std::collections::HashMap;

use packable_derive::Packable;

use crate::bool_u15;

use super::{fqdn::MDNSFQDN, MDNSTYPE};

#[derive(Debug, PartialEq, Eq, Clone, Packable)]
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
