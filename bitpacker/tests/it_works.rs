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
    // let test = Test {
    //     bln: true,
    //     num: 12,
    //     num_ref: 1,
    // };
}
