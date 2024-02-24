pub mod pack;
pub mod packets;
pub mod util;

pub type Data = bitvec::vec::BitVec<u8, bitvec::order::Msb0>;
#[allow(non_camel_case_types)]
pub type bool_u15 = u16;
