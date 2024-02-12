use anyhow::Result;
use log::debug;
use std::{fmt::Debug, thread::sleep, vec};

use crate::pack::Packable;

#[derive(PartialEq, Eq, Clone)]
pub struct MDNSFQDN {
    labels: Vec<String>,
}

impl MDNSFQDN {
    pub fn new(s: &str) -> Self {
        MDNSFQDN {
            labels: s.split('.').map(|s| s.to_string()).collect(),
        }
    }

    pub fn to_string(&self) -> String {
        self.labels.join(".")
    }

    pub fn get_labels(&self) -> Vec<String> {
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
            data.push(label.len() as u8);
            for b in label.as_bytes() {
                data.push(*b);
            }
        }
        data.push(0);
        data
    }

    fn unpack(data: &[u8], mut offset: usize) -> Result<(usize, Self)> {
        let mut labels = vec![];

        while data[offset] != 0 {
            debug!("Labels: {:#?}", labels);
            // Debug the surrounding 5 bytes.
            debug!(
                "Surrounding 5 bytes: {:#?}",
                &data[offset.saturating_sub(5)..offset + 5]
            );

            let len = data[offset] as usize;
            if (len & 0b1100_0000) == 0b1100_0000 {
                let pointer = u16::from_be_bytes([data[offset] & 0b0011_1111, data[offset + 1]]);
                debug!("Pointer: {:#?}", pointer);
                let (_, fqdn) = MDNSFQDN::unpack(data, pointer as usize)?;
                sleep(std::time::Duration::from_secs(1));

                labels.extend(fqdn.get_labels());
                offset += 2;

                continue;
            }

            offset += 1;
            let label = String::from_utf8(data[offset..offset + len].to_vec()).unwrap();
            labels.push(label);
            offset += len;
        }
        // For the terminating zero.
        offset += 1;

        let fqdn = MDNSFQDN { labels };

        debug!("Unpacked MDNSFQDN: {fqdn:#?}");

        Ok((offset, fqdn))
    }
}
