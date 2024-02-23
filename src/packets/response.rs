use std::{collections::HashMap, fmt::Debug};

use anyhow::Result;

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

impl MDNSResponse {
    pub fn new(
        data: crate::BitVec,
        header: MDNSHeader,
        mut queries: Vec<MDNSQuery>,
        mut answers: Vec<MDNSResourceRecord>,
        mut authorities: Vec<MDNSResourceRecord>,
        mut additional: Vec<MDNSResourceRecord>,
    ) -> Self {
        let mut data_cache = HashMap::new();

        queries
            .iter_mut()
            .for_each(|q| q.resolve(&data, &mut data_cache));
        answers
            .iter_mut()
            .for_each(|a| a.resolve(&data, &mut data_cache));
        authorities
            .iter_mut()
            .for_each(|a| a.resolve(&data, &mut data_cache));
        additional
            .iter_mut()
            .for_each(|a| a.resolve(&data, &mut data_cache));

        MDNSResponse {
            header,
            queries,
            answers,
            authorities,
            additional,
        }
    }
}

impl Packable for MDNSResponse {
    fn pack(&self) -> crate::BitVec {
        pack_chain![
            self.header,
            self.queries,
            self.answers,
            self.authorities,
            self.additional
        ]
    }

    fn unpack(data: &mut crate::BitVec) -> Result<Self> {
        let data_copy = data.clone();

        let header = MDNSHeader::unpack(data)?;
        let queries = read_vec_of_t(data, header.questions as usize)?;
        let answers = read_vec_of_t(data, header.answer_rrs as usize)?;
        let authorities = read_vec_of_t(data, header.authority_rrs as usize)?;
        let additional = read_vec_of_t(data, header.additional_rrs as usize)?;

        let response =
            MDNSResponse::new(data_copy, header, queries, answers, authorities, additional);

        Ok(response)
    }
}
