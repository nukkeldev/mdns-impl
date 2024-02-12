use anyhow::Result;

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

#[macro_export]
macro_rules! concat_slices {
    ($($i:expr),*) => {
        [$(&$i[..]),*].concat()
    };
}

#[macro_export]
macro_rules! concat_slices_to_bytes {
    ($($i:expr),*) => {
        [$(&($i.to_be_bytes())[..]),*].concat()
    };
}

// PACKING UTILS

pub fn read_u16s_be<const N: usize>(data: &[u8]) -> Result<[u16; N]> {
    let mut v = [0; N];
    for i in 0..N {
        let start = i * 2;

        if start + 1 >= data.len() {
            return Err(anyhow::anyhow!("Not enough data to read u16!"));
        }

        v[i] = u16::from_be_bytes([data[start], data[start + 1]]);
    }
    Ok(v)
}

pub fn read_vec_of_t<T: Packable>(mut data: &[u8], n: usize) -> Result<(&[u8], Vec<T>)> {
    let mut v = Vec::new();
    for _ in 0..n {
        let (d, item) = T::unpack(data)?;
        v.push(item);
        data = d;
    }
    Ok((data, v))
}

#[macro_export]
macro_rules! unpack_chain {
    ($data:ident => $($t:ty),*) => {{
        use paste::paste;

        let data = $data;
        $(
            #[allow(non_snake_case)]
            let (data, paste! {[<_$t>]}) = <$t>::unpack(data)?;
        )*

        (data, ($(paste! {[<_$t>]}),*))
    }};
}

#[macro_export]
macro_rules! pack_chain {
    ($($i:expr),*) => {{
        use crate::concat_slices;

        concat_slices![$($i.pack()),*]
    }};
}
