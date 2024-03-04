use std::{collections::HashMap, fmt::Debug};

use anyhow::Result;
use packable_derive::Packable;

use super::{header::MDNSHeader, query::MDNSQuery, resource_record::MDNSResourceRecord, MDNSTYPE};

#[derive(Debug, PartialEq, Eq, Clone, Packable)]
#[post_process(MDNSResponse::new)]
pub struct MDNSResponse {
    pub header: MDNSHeader,
    #[size(header.questions)]
    pub queries: Vec<MDNSQuery>,
    #[size(header.answer_rrs)]
    pub answers: Vec<MDNSResourceRecord>,
    #[size(header.authority_rrs)]
    pub authorities: Vec<MDNSResourceRecord>,
    #[size(header.additional_rrs)]
    pub additional: Vec<MDNSResourceRecord>,
}

impl MDNSResponse {
    pub fn new(
        data: crate::Data,
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

    pub fn get_resource_record_of_type(&self, ty: MDNSTYPE) -> Result<MDNSResourceRecord> {
        let record = self
            .answers
            .iter()
            .chain(self.authorities.iter())
            .chain(self.additional.iter())
            .find(|r| r.rr_type == ty)
            .ok_or_else(|| anyhow::anyhow!("No record of type {:?} found.", ty))?;

        Ok(record.clone())
    }

    pub fn get_resource_records(&self) -> Vec<&MDNSResourceRecord> {
        self.answers
            .iter()
            .chain(self.authorities.iter())
            .chain(self.additional.iter())
            .collect()
    }
}
