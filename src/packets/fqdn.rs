use anyhow::Result;
use bitvec::{order::Msb0, vec::BitVec, view::BitView};
use std::{collections::HashMap, fmt::Debug};

use crate::{load, pack::Packable};

#[derive(PartialEq, Eq, Clone)]
pub struct MDNSFQDN {
    pub labels: Vec<Label>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Label {
    String(String),
    Pointer(u16),
}

impl MDNSFQDN {
    pub fn new(s: &str) -> Self {
        MDNSFQDN {
            labels: s.split('.').map(|s| Label::String(s.to_string())).collect(),
        }
    }

    pub fn to_string(&self) -> String {
        let strings = self
            .labels
            .iter()
            .cloned()
            .map(|s| match s {
                Label::String(s) => s,
                Label::Pointer(p) => format!("<<#{p}>>"),
            })
            .collect::<Vec<_>>();

        strings.join(".")
    }

    pub fn get_labels(&self) -> Vec<Label> {
        self.labels.clone()
    }

    pub fn resolve(
        &mut self,
        data: &crate::Data,
        data_cache: &mut HashMap<usize, String>,
        pointer_idx: Option<usize>,
    ) -> &mut Self {
        if pointer_idx.is_some() && data_cache.contains_key(&pointer_idx.unwrap()) {
            *self = MDNSFQDN::new(&data_cache.get(&pointer_idx.unwrap()).unwrap().clone());
        }

        self.labels.iter_mut().for_each(|label| {
            if let Label::Pointer(p) = label {
                *label = Label::String(
                    MDNSFQDN::unpack(&mut data[(*p as usize) * 8..].to_bitvec())
                        .unwrap()
                        .resolve(&data, data_cache, Some(*p as usize))
                        .to_string(),
                )
            };
        });

        if pointer_idx.is_some() && !data_cache.contains_key(&pointer_idx.unwrap()) {
            data_cache.insert(pointer_idx.unwrap(), self.to_string());
        }

        self
    }
}

impl Debug for MDNSFQDN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MDNSFQDN({})", self.to_string())
    }
}

impl Packable for MDNSFQDN {
    fn pack(&self) -> crate::Data {
        let mut data = BitVec::new();
        for label in &self.labels {
            match label {
                Label::String(s) => {
                    data.extend((s.len() as u8).view_bits::<Msb0>());
                    data.extend(s.as_bytes());
                }
                Label::Pointer(p) => {
                    let p = *p | 0b1100_0000;
                    data.extend(p.view_bits::<Msb0>());
                }
            }
        }
        if let Label::String(_) = self.labels.last().unwrap() {
            data.extend_from_bitslice(0u8.view_bits::<Msb0>());
        }
        data
    }

    fn unpack(data: &mut crate::Data) -> Result<Self> {
        let mut labels = vec![];

        while data[..8].any() {
            let len = load!(data => u8) as usize;
            if len & 0b1100_0000 == 0b1100_0000 {
                let pointer = ((len as u16 & 0b0011_1111) << 8) | load!(data => u8) as u16;
                labels.push(Label::Pointer(pointer));
                break;
            }

            let label =
                String::from_utf8((0..len).map(|_| load!(data => u8)).collect::<Vec<_>>()).unwrap();
            labels.push(Label::String(label));
        }
        // For the terminating zero.
        if !labels.is_empty() {
            if let Label::String(_) = labels.last().unwrap() {
                data.drain(..8);
            }
        }
        let fqdn = MDNSFQDN { labels };

        Ok(fqdn)
    }
}
