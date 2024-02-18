use bitpacker::bitpacked;

bitpacked! {
    pub Test {
        [$] bool bln;
        [$] u15 num;
    }
}

fn main() {
    let test = Test {
        bln: true,
        num: 12,
        num_ref: 1
    };
}

// https://bnfplayground.pauliankline.com/?bnf=%3Cfile%3E+%3A%3A%3D+%28%3Cnewline%3E+%3Cstruct%3E+%3Cnewline%3E%29*%0A%0A%3Cstruct%3E+%3A%3A%3D+%3Cvis%3E+%3Cident%3E+%3Cseperator%3E+%3Cblock%3E%0A%3Cvis%3E+%3A%3A%3D+%28%22pub%22+%3Cseperator%3E%29+%7C+E%0A%3Cblock%3E+%3A%3A%3D+%22%7B%22+%3Cnewline%3E+%28%3Cexpr%3E+%3Cnewline%3E%29*+%3Cnewline%3E+%22%7D%22%0A%0A%3Cident%3E+%3A%3A%3D+%3Cleading%3E+%3Cany%3E*%0A%3Cleading%3E+%3A%3A%3D+%5Ba-z%5D+%7C+%5BA-Z%5D+%7C+%22_%22%0A%3Ctrailing%3E+%3A%3A%3D+%3Cleading%3E+%7C+%5B0-9%5D%0A%3Cidents%3E+%3A%3A%3D+%3Cident%3E+%28%3Cnewline%3E+%22%2C%22+%3Cnewline%3E+%3Cident%3E%29*%0A%0A%3Cexpr%3E+%3A%3A%3D+%28%28%3Cdecl%3E+%7C+%3Cloc%3E%29+%22%3B%22%29+%7C+%3Ccomment%3E%0A%3Cdecl%3E+%3A%3A%3D+%3Cloc%3E+%3Cseperator%3E%3F+%3Ctype%3E+%3Cseperator%3E+%3Cidents%3E%0A%3Ccomment%3E+%3A%3A%3D+%22%2F%2F%22+%3Cany%3E*+%22%5Cn%22%0A%0A%3Cloc%3E+%3A%3A%3D+%22%24%22+%3Cloc_adj%3E%3F%0A%3Cloc_adj%3E+%3A%3A%3D+%22%5B%22+%28%22%2B%22+%7C+%22-%22%29%3F+%28%3Cbyte%3E+%28%22_%22+%3Cbit%3E%29%3F+%7C+%3Cbit%3E%29%3F+%22%5D%22%0A%3Cbyte%3E+%3A%3A%3D+%220x%22+%3Chex%3E%2B%0A%3Cbit%3E+%3A%3A%3D+%220b%22+%3Chex%3E%2B%0A%3Chex%3E+%3A%3A%3D+%28%5B0-9%5D+%7C+%5Ba-f%5D%29%0A%0A%3Ctype%3E+%3A%3A%3D+%22bool%22+%7C+%3Cutype%3E+%7C+%3Cident%3E%0A%3Cutype%3E+%3A%3A%3D+%22u%22+%5B0-9%5D%2B%0A%0A%3Cseperator%3E+%3A%3A%3D+%22+%22%0A%3Cnewline%3E+%3A%3A%3D+%28%22+%22+%7C+%22%5Cn%22%29*%0A%3Cany%3E+%3A%3A%3D+%5Ba-z%5D+%7C+%5BA-Z%5D+%7C+%5B0-9%5D+%7C+%22+%22+%7C+%22_%22+%7C+%22-%22+%7C+%22%3B%22&name=