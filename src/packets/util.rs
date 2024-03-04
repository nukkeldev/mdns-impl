use std::fmt::Debug;

use anyhow::Result;

use super::pack::Packable;

pub fn read_vec_of_t<T: Packable + Debug>(data: &mut crate::Data, n: usize) -> Result<Vec<T>> {
    Ok((0..n).map(|_| T::unpack(data).unwrap()).collect::<Vec<_>>())
}

/// Concatenate a series of `Packable` types into a single `BitVec`.
#[macro_export]
macro_rules! concat_packable_bits {
    ($($j:expr),*) => {{
        let mut out = crate::Data::new();
        $(out.extend($j.pack());)*
        out
    }};
}

/// Unpack a series of `Packable` types from a `&mut BitVec`.
#[macro_export]
macro_rules! unpack_chain {
    ($data:ident => $($t:ty),*) => {{
        use paste::paste;
        $(
            #[allow(non_snake_case)]
            let paste! {[<_$t>]} = <$t>::unpack($data)?;
        )*

        ($(paste! {[<_$t>]}),*)
    }};
}

/// Drain a numerical value from a `&mut BitVec`.
#[macro_export]
macro_rules! load {
    ($data:expr => $ty:ty) => {{
        use bitvec::field::BitField;
        $data
            .drain(..::std::mem::size_of::<$ty>() * 8)
            .as_bitslice()
            .load_be::<$ty>()
    }};
}
