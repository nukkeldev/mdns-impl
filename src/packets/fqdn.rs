use anyhow::Result;
use log::debug;
use std::fmt::Debug;

use crate::pack::Packable;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MDNSFQDN {
    labels: [&'static str; 3],
}

impl MDNSFQDN {
    pub fn new(s: &'static str) -> Self {
        let labels: Vec<&str> = s.split('.').collect();

        MDNSFQDN {
            labels: [labels[0], labels[1], labels[2]],
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for label in &self.labels {
            s.push_str(label);
            s.push('.');
        }
        s.pop();
        s
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

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let mut labels = [""; 3];

        let mut i = 0;
        for j in 0..3 {
            let len = data[i] as usize;
            i += 1;
            let label = String::from_utf8(data[i..i + len].to_vec()).unwrap();
            labels[j] = Box::leak(label.into_boxed_str());
            i += len;
        }
        // For the terminating zero.
        i += 1;

        let fqdn = MDNSFQDN { labels };

        debug!("Unpacked MDNSFQDN: {fqdn:#?}");

        Ok((&data[i..], fqdn))
    }
}
