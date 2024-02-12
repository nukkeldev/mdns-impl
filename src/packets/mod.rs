use anyhow::Result;
use log::debug;

use crate::pack::Packable;

pub mod flags;
pub mod fqdn;
pub mod header;
pub mod packet;
pub mod query;
pub mod resource_record;
pub mod response;

// ENUMS

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MDNSTYPE {
    // RESOURCE RECORDS
    A = 1,
    AAAA = 28,
    AFSDB = 18,
    APL = 42,
    CAA = 257,
    CDNSKEY = 60,
    CDS = 59,
    CERT = 37,
    CNAME = 5,
    CSYNC = 62,
    DHCID = 49,
    DLV = 32769,
    DNAME = 39,
    DNSKEY = 48,
    DS = 43,
    EUI48 = 108,
    EUI64 = 109,
    HINFO = 13,
    HIP = 55,
    HTTPS = 65,
    IPSECKEY = 45,
    KEY = 25,
    KX = 36,
    LOC = 29,
    MX = 15,
    NAPTR = 35,
    NS = 2,
    NSEC = 47,
    NSEC3 = 50,
    NSEC3PARAM = 51,
    OPENPGPKEY = 61,
    PTR = 12,
    RRSIG = 46,
    RP = 17,
    SIG = 24,
    SMIMEA = 53,
    SOA = 6,
    SRV = 33,
    SSHFP = 44,
    SVCB = 64,
    TA = 32768,
    TKEY = 249,
    TLSA = 52,
    TSIG = 250,
    TXT = 16,
    URI = 256,
    ZONEMD = 63,
    // OTHER TYPES
    ANY = 255,
    AXFR = 252,
    IXFR = 251,
    OPT = 41,
}

impl From<u16> for MDNSTYPE {
    fn from(value: u16) -> Self {
        use MDNSTYPE::*;

        match value {
            // RESOURCE RECORDS
            1 => A,
            28 => AAAA,
            18 => AFSDB,
            42 => APL,
            257 => CAA,
            60 => CDNSKEY,
            59 => CDS,
            37 => CERT,
            5 => CNAME,
            62 => CSYNC,
            49 => DHCID,
            3276 => DLV,
            39 => DNAME,
            48 => DNSKEY,
            43 => DS,
            108 => EUI48,
            109 => EUI64,
            13 => HINFO,
            55 => HIP,
            65 => HTTPS,
            45 => IPSECKEY,
            25 => KEY,
            36 => KX,
            29 => LOC,
            15 => MX,
            35 => NAPTR,
            2 => NS,
            47 => NSEC,
            50 => NSEC3,
            51 => NSEC3PARAM,
            61 => OPENPGPKEY,
            12 => PTR,
            46 => RRSIG,
            17 => RP,
            24 => SIG,
            53 => SMIMEA,
            6 => SOA,
            33 => SRV,
            44 => SSHFP,
            64 => SVCB,
            32768 => TA,
            249 => TKEY,
            52 => TLSA,
            250 => TSIG,
            16 => TXT,
            256 => URI,
            63 => ZONEMD,
            // OTHER TYPES
            255 => ANY,
            252 => AXFR,
            251 => IXFR,
            41 => OPT,
            _ => panic!("Invalid MDNSTYPE received: {}", value),
        }
    }
}

impl Packable for MDNSTYPE {
    fn pack(&self) -> Vec<u8> {
        (*self as u16).to_be_bytes().to_vec()
    }

    fn unpack(data: &[u8]) -> Result<(&[u8], Self)> {
        let ty = u16::from_be_bytes([data[0], data[1]]).into();

        debug!("Unpacked MDNSTYPE: {ty:?}");

        Ok((&data[2..], ty))
    }
}
