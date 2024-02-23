use std::collections::HashMap;

use anyhow::Result;

use crate::{
    pack::{BoolU15, Packable},
    unpack_chain,
};

use super::{fqdn::MDNSFQDN, MDNSTYPE};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSQuery {
    pub qname: MDNSFQDN,
    pub qtype: MDNSTYPE,
    pub qu_qclass: BoolU15,
}

impl MDNSQuery {
    pub fn new(name: &'static str, qtype: MDNSTYPE) -> Self {
        let qname = MDNSFQDN::new(name);
        let qu_qclass = BoolU15::new(true, 1);
        MDNSQuery {
            qname,
            qtype,
            qu_qclass,
        }
    }

    pub fn get_name(&self) -> String {
        self.qname.to_string()
    }

    pub fn get_type(&self) -> MDNSTYPE {
        self.qtype
    }

    pub fn get_class(&self) -> u16 {
        self.qu_qclass.get_u15()
    }

    pub fn get_qu(&self) -> bool {
        self.qu_qclass.get_bool()
    }

    pub fn resolve(&mut self, data: &crate::BitVec, data_cache: &mut HashMap<usize, String>) {
        self.qname.resolve(data, data_cache, None);
    }
}

impl Packable for MDNSQuery {
    fn pack(&self) -> crate::BitVec {
        let mut data = crate::BitVec::new();
        data.append(&mut self.qname.pack());
        data.append(&mut self.qtype.pack());
        data.append(&mut self.qu_qclass.pack());
        data
    }

    fn unpack(data: &mut crate::BitVec) -> Result<Self> {
        let (qname, qtype, qu_qclass) = unpack_chain!(data => MDNSFQDN, MDNSTYPE, BoolU15);

        let query = MDNSQuery {
            qname,
            qtype,
            qu_qclass,
        };

        Ok(query)
    }
}
