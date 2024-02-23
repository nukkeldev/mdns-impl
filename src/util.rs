use std::fmt::Debug;

use crate::load;
use anyhow::Result;
use bitvec::vec::BitVec;

use crate::pack::Packable;

// FORMATING UTILS

pub fn format_slices_as_bits(v: &[u8], bytes_per_line: usize) -> String {
    let mut s = String::new();
    let mut i = 0;
    for byte in v {
        s.push_str(&format!("{:08b} ", byte));
        i += 1;
        if i % bytes_per_line == 0 {
            s.push_str("\n");
        }
    }
    s
}

pub fn format_slices_as_dec(v: &[u8], bytes_per_line: usize) -> String {
    let mut s = String::new();
    let mut i = 0;
    for byte in v {
        s.push_str(&format!("{:03} ", byte));
        i += 1;
        if i % bytes_per_line == 0 {
            s.push_str("\n");
        }
    }
    s
}

#[macro_export]
macro_rules! concat_bits {
    ($($j:expr),*) => {{
        let mut out = BitVec::<u8>::new();
        $(out.extend($j);)*
        out
    }};
}

#[macro_export]
macro_rules! concat_slices_to_bytes {
    ($($i:expr),*) => {
        [$(&($i.to_be_bytes())[..]),*].concat()
    };
}

// PACKING UTILS

pub fn read_u16s_be<const N: usize>(data: &mut BitVec<u8>) -> Result<[u16; N]> {
    Ok([0; N].map(|_| load!(data => u16)))
}

pub fn read_vec_of_t<T: Packable + Debug>(data: &mut BitVec<u8>, n: usize) -> Result<Vec<T>> {
    Ok((0..n).map(|_| T::unpack(data).unwrap()).collect::<Vec<_>>())
}

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

#[macro_export]
macro_rules! pack_chain {
    ($($i:expr),*) => {{
        use crate::concat_bits;

        concat_bits![$($i.pack()),*]
    }};
}

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
