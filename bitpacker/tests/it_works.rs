use bitpacker::bitpacked;

bitpacked! {
    MDNSHeader {
        $ u16 transaction_id;
        $ Flags {
            $ u1 qr;
            $ u4 opcode;
            $ u1 aa, tc, rd, ra;
            $[b3];
            $ u4 rcode;
        } flags;
        $ u16 qn, an, authn, addn;
    }
}

fn main() {
    let header = MDNSHeader {
        transaction_id: 0x1234,
        flags: Flags {
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
}
