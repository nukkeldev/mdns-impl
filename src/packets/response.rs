use anyhow::Result;
use log::debug;

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
    fn pack(&self) -> Vec<u8> {
        pack_chain![
            self.header,
            self.queries,
            self.answers,
            self.authorities,
            self.additional
        ]
    }

    fn unpack(data: &[u8], offset: usize) -> Result<(usize, Self)> {
        let (offset, header) = MDNSHeader::unpack(data, offset)?;
        let (offset, queries) = read_vec_of_t(data, offset, header.questions as usize)?;
        let (offset, answers) = read_vec_of_t(data, offset, header.answer_rrs as usize)?;
        let (offset, authorities) = read_vec_of_t(data, offset, header.authority_rrs as usize)?;
        let (offset, additional) = read_vec_of_t(data, offset, header.additional_rrs as usize)?;

        let response = MDNSResponse {
            header,
            queries,
            answers,
            authorities,
            additional,
        };

        debug!("Unpacked MDNSResponse: {:?}", response);

        Ok((offset, response))
    }
}
