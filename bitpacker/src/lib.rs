use std::fmt::Debug;

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

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(token::Bracket) {
            parse_padding(input)?
        } else {
            let ident = input.parse::<Ident>()?;
            let ident_str = ident.to_string();

            if input.peek(token::Brace) {
                let exprs = parse_block(input)?;
                Type::Inline(Input { name: ident, exprs })
            } else if ident_str.starts_with("u") {
                let num = ident_str[1..].parse::<usize>().unwrap();
                Type::UType(num)
            } else if ident_str.starts_with("i") {
                let num = ident_str[1..].parse::<usize>().unwrap();
                Type::IType(num)
            } else if ident_str.starts_with("f") {
                let num = ident_str[1..].parse::<usize>().unwrap();
                Type::FType(num)
            } else {
                Type::Arb(ident)
            }
        })
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
