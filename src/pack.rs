use crate::impl_packable_for_int;
use anyhow::{Ok, Result};
use bitvec::prelude::*;

pub trait Packable: Sized {
    fn pack(&self) -> crate::Data;
    fn unpack(data: &mut crate::Data) -> Result<Self>;
}

impl<T, const N: usize> Packable for [T; N]
where
    T: Packable + Clone + Copy + Default,
{
    fn pack(&self) -> crate::Data {
        BitVec::from_iter(self.into_iter().flat_map(|e| e.pack()))
    }

    fn unpack(data: &mut crate::Data) -> Result<Self> {
        Ok([T::default(); N].map(|_| T::unpack(data).unwrap()))
    }
}

impl<T> Packable for Vec<T>
where
    T: Packable,
{
    fn pack(&self) -> crate::Data {
        BitVec::from_iter(self.into_iter().flat_map(|e| e.pack()))
    }

    fn unpack(_data: &mut crate::Data) -> Result<Self> {
        panic!(
            "Unpacking Vec<T> is not allowed! Please unpack with a type using util::read_vec_of_t!"
        )
    }
}

impl Packable for bool {
    fn pack(&self) -> crate::Data {
        let mut out = BitVec::new();
        out.push(*self);
        out
    }

    fn unpack(data: &mut crate::Data) -> Result<Self> {
        Ok(data.pop().unwrap())
    }
}

impl_packable_for_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

#[macro_export]
macro_rules! impl_packable_for_int {
    ($($t:ty),*) => {
        $(
            impl Packable for $t {
                fn pack(&self) -> crate::Data {
                    self.to_be_bytes().view_bits().to_bitvec()
                }

                fn unpack(data: &mut crate::Data) -> Result<Self> {
                    Ok(data.drain(..std::mem::size_of::<$t>() * 8).as_bitslice().load_be::<$t>())
                }
            }
        )*
    };
}
