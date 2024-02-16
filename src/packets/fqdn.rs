use anyhow::Result;
use log::debug;
use std::fmt::Debug;

use crate::pack::Packable;

#[derive(PartialEq, Eq, Clone)]
pub struct MDNSFQDN {
    labels: Vec<Label>,
}

#[derive(PartialEq, Eq, Clone)]
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
}

impl Debug for MDNSFQDN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MDNSFQDN({})", self.to_string())
    }
}

impl Packable for MDNSFQDN {
    fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for label in &self.labels {
            match label {
                Label::String(s) => {
                    data.push(s.len() as u8);
                    for b in s.as_bytes() {
                        data.push(*b);
                    }
                }
                Label::Pointer(p) => {
                    let p = *p | 0b1100_0000;
                    let bytes = p.to_be_bytes();
                    data.push(bytes[0]);
                    data.push(bytes[1]);
                }
            }
        }
        if let Label::String(_) = self.labels.last().unwrap() {
            data.push(0);
        }
        data
    }

    fn unpack(data: &[u8], mut offset: usize) -> Result<(usize, Self)> {
        let mut labels = vec![];

        while data[offset] != 0 {
            let len = data[offset] as usize;
            if len & 0b1100_0000 == 0b1100_0000 {
                let pointer = ((len as u16 & 0b0011_1111) << 8) | data[offset + 1] as u16;
                labels.push(Label::Pointer(pointer));
                offset += 2;
                continue;
            }

            offset += 1;
            let label = String::from_utf8(data[offset..offset + len].to_vec()).unwrap();
            labels.push(Label::String(label));
            offset += len;
        }
        // For the terminating zero.
        if !labels.is_empty() {
            if let Label::String(_) = labels.last().unwrap() {
                offset += 1;
            }
        }
        let fqdn = MDNSFQDN { labels };

        debug!("Unpacked MDNSFQDN: {fqdn:#?}");

        Ok((offset, fqdn))
    }
}
