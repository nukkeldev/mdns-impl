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

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let (data, header) = MDNSHeader::unpack(data)?;
        let (data, queries) = read_vec_of_t(data, header.questions as usize)?;
        let (data, answers) = read_vec_of_t(data, header.answer_rrs as usize)?;
        let (data, authorities) = read_vec_of_t(data, header.authority_rrs as usize)?;
        let (data, additional) = read_vec_of_t(data, header.additional_rrs as usize)?;

        let response = MDNSResponse {
            header,
            queries,
            answers,
            authorities,
            additional,
        };

        debug!("Unpacked MDNSResponse: {:?}", response);

        Ok((data, response))
    }
}
