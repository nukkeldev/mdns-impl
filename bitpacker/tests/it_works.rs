use bitpacker::bitpacked;

bitpacked! {
    MDNSHeader {
        $ u16 transaction_id;
        $ MDNSHeaderFlags flags;
        $ u16 qn, an, authn, addn;
    }

    MDNSHeaderFlags {
        $ u1 qr;
        $ u4 opcode;
        $ u1 aa, tc, rd, ra;
        $[b3];
        $ u4 rcode;
    }
}

fn main() {
    let mut header = MDNSHeader {
        transaction_id: 0x1234,
        flags: MDNSHeaderFlags {
            qr: true,
            opcode: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            rcode: 0,
        },
        qn: 1,
        an: 0,
        authn: 0,
        addn: 0,
    };

    header.get_flags_mut().set_rcode(u8::MAX);
}
