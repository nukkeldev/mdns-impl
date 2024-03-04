use packable_derive::Packable;

#[derive(Debug, PartialEq, Eq, Clone, Packable)]
pub struct MDNSHeader {
    pub transaction_id: u16,
    pub flags: u16,
    pub questions: u16,
    pub answer_rrs: u16,
    pub authority_rrs: u16,
    #[size(questions)]
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