use std::fmt::Debug;

use anyhow::{Ok, Result};
use bitvec::prelude::*;

pub trait Packable: Sized {
    fn pack(&self) -> BitVec<u8>;
    fn unpack(data: &mut BitVec<u8>) -> Result<Self>;
}

impl<T, const N: usize> Packable for [T; N]
where
    T: Packable + Clone + Copy + Default,
{
    fn pack(&self) -> BitVec<u8> {
        BitVec::from_iter(self.into_iter().flat_map(|e| e.pack()))
    }

    fn unpack(data: &mut BitVec<u8>) -> Result<Self> {
        Ok([T::default(); N].map(|_| T::unpack(data).unwrap()))
    }
}

impl<T> Packable for Vec<T>
where
    T: Packable,
{
    fn pack(&self) -> BitVec<u8> {
        BitVec::from_iter(self.into_iter().flat_map(|e| e.pack()))
    }

    fn unpack(_data: &mut BitVec<u8>) -> Result<Self> {
        panic!(
            "Unpacking Vec<T> is not allowed! Please unpack with a type using util::read_vec_of_t!"
        )
    }
}

use crate::{impl_packable_unsigned, load};
impl_packable_unsigned!(u8, u16, u32, u64, u128);

// COMPRESSED TYPES

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct BoolU15(u16);

impl BoolU15 {
    pub fn new(b: bool, u: u16) -> Self {
        BoolU15((b as u16) << 15 | u)
    }

    pub fn set_bool(&mut self, b: bool) {
        self.0 |= (b as u16) << 15;
    }

    pub fn set_u15(&mut self, u: u16) {
        self.0 = self.0 << 15 | u;
    }

    pub fn get_bool(&self) -> bool {
        (self.0 >> 15) == 1
    }

    pub fn get_u15(&self) -> u16 {
        self.0 & 0b0111111111111111
    }
}

impl Debug for BoolU15 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BoolU15({}, {})", self.get_bool(), self.get_u15())
    }
}

impl Packable for BoolU15 {
    fn pack(&self) -> BitVec<u8> {
        self.0.pack()
    }

    fn unpack(data: &mut BitVec<u8>) -> Result<Self> {
        Ok(BoolU15(load!(data => u16)))
    }
}

// Helper Macros
#[macro_export]
macro_rules! impl_packable_unsigned {
    ($($t:ty),*) => {
        $(
            impl Packable for $t {
                fn pack(&self) -> BitVec<u8> {
                    self.to_be_bytes().view_bits::<Lsb0>().to_bitvec()
                }

                fn unpack(data: &mut BitVec<u8>) -> Result<Self> {
                    Ok(load!(data => $t))
                }
            }
        )*
    };
    () => {

    };
}
