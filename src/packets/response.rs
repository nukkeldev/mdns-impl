use anyhow::Result;
use bitvec::vec::BitVec;

use crate::{pack::Packable, pack_chain, util::read_vec_of_t};

use super::{header::MDNSHeader, query::MDNSQuery, resource_record::MDNSResourceRecord};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSResponse {
    header: MDNSHeader,
    queries: Vec<MDNSQuery>,
    answers: Vec<MDNSResourceRecord>,
    authorities: Vec<MDNSResourceRecord>,
    additional: Vec<MDNSResourceRecord>,
}

impl Packable for MDNSResponse {
    fn pack(&self) -> BitVec<u8> {
        pack_chain![
            self.header,
            self.queries,
            self.answers,
            self.authorities,
            self.additional
        ]
    }

    fn unpack(data: &mut BitVec<u8>) -> Result<Self> {
        let header = MDNSHeader::unpack(data)?;
        let queries = read_vec_of_t(data, header.questions as usize)?;
        let answers = read_vec_of_t(data, header.answer_rrs as usize)?;
        let authorities = read_vec_of_t(data, header.authority_rrs as usize)?;
        let additional = read_vec_of_t(data, header.additional_rrs as usize)?;

        let response = MDNSResponse {
            header,
            queries,
            answers,
            authorities,
            additional,
        };

        Ok(response)
    }
}
