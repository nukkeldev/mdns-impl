use std::{error::Error, fmt::Debug};

use syn::{braced, bracketed, parse::Parse, parse_macro_input, token, Ident, Token};

#[proc_macro]
pub fn bitpacked(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(_input as Input);
    // eprintln!("{:#?}", input);
    eprintln!("{:#?}", input.packed_size());

    proc_macro::TokenStream::default()
}

// Declarations

#[derive(Debug)]
struct Input {
    name: Ident,
    exprs: Vec<Expr>,
}

#[derive(Debug)]
struct Expr {
    ty: Type,
    names: Vec<Ident>,
}

#[derive(Debug)]
enum Type {
    Padding(PaddingSize),
    Arb(Ident),
    Inline(Input),
    UType(usize),
    IType(usize),
    FType(usize),
}

#[derive(Debug)]
struct PaddingSize {
    byte: usize,
    bit: usize,
}

// Parsing

fn parse_block(input: syn::parse::ParseStream) -> syn::Result<Vec<Expr>> {
    let content;
    braced!(content in input);
    Ok(content
        .parse_terminated(Expr::parse, Token![;])?
        .into_iter()
        .collect())
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let exprs = parse_block(input)?;

        Ok(Self { name, exprs })
    }
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![$]>()?;

        let ty = input.parse()?;
        let mut names = vec![];

        loop {
            if input.peek(Token![;]) {
                break;
            }

            names.push(input.parse()?);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(Self { ty, names })
    }
}

fn parse_padding(input: syn::parse::ParseStream) -> syn::Result<Type> {
    let location;
    bracketed!(location in input);

    let mut offset = location.parse::<Ident>()?.to_string();
    let mut byte = 0;
    let mut bit = 0;
    if offset.starts_with("x") {
        let end = offset.find("_").unwrap_or(offset.len());
        byte = usize::from_str_radix(&offset[1..end], 16)
            .expect(format!("'{}' is not a valid hexadecimal number!", &offset[1..end]).as_str());
        if let Some("_") = offset.get(end..=end) {
            offset = offset[end + 1..].to_string();
        } else {
            offset = offset[end..].to_string();
        }
    }
    if offset.starts_with("b") {
        bit = usize::from_str_radix(&offset[1..], 16)
            .expect(format!("'{}' is not a valid hexadecimal number!", &offset[1..]).as_str());
    }

    Ok(Type::Padding(PaddingSize { byte, bit }))
}

/// Parses a hexadecimal number from a string.
fn parse_hex(input: &str) -> Result<usize, ()> {
    usize::from_str_radix(input, 16).map_err(|_| ())
}

impl Parse for Type {
    #[rustfmt::skip]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Types can either be inline types of other bitpacked structs,
        // arbirary (bitpacked) types, variably sized unsigned/signed/float types,
        // or padding (bit-aligned).

        // Handle the parsing of padding, as it is a unique case compared to the other types.
        if input.peek(token::Bracket) {
            return Ok(parse_padding(input)?);
        }

        // All other types are prefixed with an identifier.
        let ident = input.parse::<Ident>()?;
        let ident_str = ident.to_string();

        // If the next token is a brace, then we are parsing an inline type.
        if input.peek(token::Brace) {
            let exprs = parse_block(input)?;
            return Ok(Type::Inline(Input { name: ident, exprs }));
        }

        // If the identifier starts with a u, i, or f, it is possible to be a variably sized number.
        if "uif".contains(&ident_str[0..1]) {
            // If the identifier is followed by a valid hexidecimal number, then the user likely intends for it to be interrepted as such.
            // Although this allows for some ambiguity, it is unlikely that the user will have a type that is in this format.
            let num = parse_hex(&ident_str[1..]);

            return match num {
                Ok(num) => Ok(match &ident_str.as_str()[0..1] {
                    "u" => Type::UType(num),
                    "i" => Type::IType(num),
                    "f" => Type::FType(num),
                    _ => unreachable!(),
                }),
                Err(_) => Ok(Type::Arb(ident)),
            };
        }
        
        Ok(Type::Arb(ident))
    }
}

// Codegen

// Impls

impl Input {
    /// Returns the size of the bitpacked struct in bits.
    fn packed_size(&self) -> usize {
        // Summate the sizes of fields.
        self.exprs.iter().map(|expr| expr.packed_size()).sum()
    }
}

impl Expr {
    /// Returns the size of the bitpacked field(s) in bits.
    fn packed_size(&self) -> usize {
        // Kind of a hacky way to make sure we don't return 0 for padding fields.
        self.ty.packed_size() * self.names.len().max(1)
    }
}

impl Type {
    /// Returns the size of the bitpacked type in bits.
    fn packed_size(&self) -> usize {
        match self {
            Type::Padding(size) => size.byte * 8 + size.bit,
            // I'm not quite sure if this is possible to implement,
            // due to needing to dynamically call sizes of other bitpacked structs.
            // It's quite easy to do at runtime though, just not compile time.
            Type::Arb(_) => 0,
            Type::Inline(input) => input.packed_size(),
            Type::UType(num) | Type::IType(num) | Type::FType(num) => *num,
        }
    }
}
