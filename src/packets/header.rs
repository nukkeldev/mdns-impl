use crate::{concat_packable_bits, pack::Packable, util::read_u16s_be};
use anyhow::Result;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDNSHeader {
    pub transaction_id: u16,
    pub flags: u16,
    pub questions: u16,
    pub answer_rrs: u16,
    pub authority_rrs: u16,
    pub additional_rrs: u16,
}

impl Default for MDNSHeader {
    fn default() -> Self {
        MDNSHeader {
            transaction_id: 0,
            flags: 0,
            questions: 1,
            answer_rrs: 0,
            authority_rrs: 0,
            additional_rrs: 0,
        }
    }
}

impl MDNSHeader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Packable for MDNSHeader {
    fn pack(&self) -> crate::Data {
        concat_packable_bits![
            self.transaction_id,
            self.flags,
            self.questions,
            self.answer_rrs,
            self.authority_rrs,
            self.additional_rrs
        ]
    }

    fn unpack(data: &mut crate::Data) -> Result<Self> {
        let [transaction_id, flags, questions, answer_rrs, authority_rrs, additional_rrs] =
            read_u16s_be::<6>(data).expect("Failed to read u16s from data.");

        let header = MDNSHeader {
            transaction_id,
            flags,
            questions,
            answer_rrs,
            authority_rrs,
            additional_rrs,
        };

        Ok(header)
    }
}
