use std::fmt::Debug;

use anyhow::{Ok, Result};

pub trait Packable: Sized {
    fn pack(&self) -> Vec<u8>;
    fn unpack(data: &[u8]) -> Result<(&[u8], Self)>;
}

impl<T, const N: usize> Packable for [T; N]
where
    T: Packable + Clone + Copy,
{
    fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for item in self {
            data.append(&mut item.pack());
        }
        data
    }

    fn unpack(mut data: &[u8]) -> Result<(&[u8], Self)> {
        let mut items: [Option<T>; N] = [None; N];

        for i in 0..N {
            let (d, item) = T::unpack(data)?;

            items[i] = Some(item);
            data = d;
        }

        Ok((data, items.map(|n| n.unwrap())))
    }
}

impl<T> Packable for Vec<T>
where
    T: Packable,
{
    fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for item in self {
            data.append(&mut item.pack());
        }
        data
    }

    fn unpack(mut _data: &[u8]) -> Result<(&[u8], Self)> {
        panic!(
            "Unpacking Vec<T> is not allowed! Please unpack with a type using util::read_vec_of_t!"
        )
    }
}

impl Packable for u8 {
    fn pack(&self) -> Vec<u8> {
        vec![*self]
    }

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        Ok((&data[1..], data[0]))
    }
}

impl Packable for u16 {
    fn pack(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let n = u16::from_be_bytes([data[0], data[1]]);
        Ok((&data[2..], n))
    }
}

impl Packable for u32 {
    fn pack(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let n = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        Ok((&data[4..], n))
    }
}

impl Packable for u64 {
    fn pack(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let n = u64::from_be_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        Ok((&data[8..], n))
    }
}

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
    fn pack(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let n = u16::from_be_bytes([data[0], data[1]]);
        Ok((&data[2..], BoolU15(n)))
    }
}

// UNIT TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_u15() {
        let b = BoolU15::new(true, 234);
        assert_eq!(b.get_bool(), true);
        assert_eq!(b.get_u15(), 234);

        let b = BoolU15::new(false, 542);
        assert_eq!(b.get_bool(), false);
        assert_eq!(b.get_u15(), 542);
    }

    #[test]
    fn test_bool_u15_set() {
        let mut b = BoolU15::new(true, 234);
        b.set_bool(false);
        b.set_u15(542);
        assert_eq!(b.get_bool(), false);
        assert_eq!(b.get_u15(), 542);
    }

    #[test]
    fn test_bool_u15_overflow() {
        let b = BoolU15::new(true, 42691);
        assert_eq!(b.get_bool(), true);
        assert_eq!(b.get_u15(), 42691 % 32768);
    }

    #[test]
    fn test_bool_u15_pack() {
        let b = BoolU15::new(true, 8412);
        assert_eq!(b.pack(), vec![0b1010_0000, 0b1101_1100]);
    }

    #[test]
    fn test_bool_u15_unpack() {
        let (_, b) = BoolU15::unpack(&[0b1010_0000, 0b1101_1100]).unwrap();
        assert_eq!(b.get_bool(), true);
        assert_eq!(b.get_u15(), 8412);
    }
}
