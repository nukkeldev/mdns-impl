use std::fmt::Debug;

use syn::{braced, bracketed, parse::Parse, parse_macro_input, token, Ident, Token, Visibility};

#[proc_macro]
pub fn bitpacked(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(_input as Input);
    eprintln!("{:#?}", input);

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
    location: Location,
    ty: Type,
    names: Vec<Ident>,
}

#[derive(Debug)]
enum Type {
    Arb(Ident),
    Inline(Input),
    UType(usize),
    IType(usize),
    FType(usize),
}

#[derive(Debug)]
enum Location {
    Current,
    Adjusted(LocationAdjustment),
}

#[derive(Debug)]
struct LocationAdjustment {
    mode: LocationAdjustmentMode,
    byte: usize,
    bit: usize,
}

#[derive(Debug)]
enum LocationAdjustmentMode {
    Relative,
    RelativeIncrement,
    RelativeDecrement,
}

// Parsing

impl Input {
    pub fn parse_block(input: syn::parse::ParseStream) -> syn::Result<Vec<Expr>> {
        let content;
        braced!(content in input);
        Ok(content
            .parse_terminated(Expr::parse, Token![;])?
            .into_iter()
            .collect())
    }
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let exprs = Input::parse_block(input)?;

        Ok(Self { name, exprs })
    }
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![$]>()?;

        let location = if input.peek(token::Bracket) {
            let location;
            bracketed!(location in input);

            let mode = if location.peek(Token![+]) {
                location.parse::<Token![+]>()?;
                LocationAdjustmentMode::RelativeIncrement
            } else if location.peek(Token![-]) {
                location.parse::<Token![-]>()?;
                LocationAdjustmentMode::RelativeDecrement
            } else {
                LocationAdjustmentMode::Relative
            };

            let mut offset = location.parse::<Ident>()?.to_string();
            let mut byte = 0;
            let mut bit = 0;
            if offset.starts_with("x") {
                let end = offset.find("_").unwrap_or(offset.len());
                byte = usize::from_str_radix(&offset[1..end], 16).unwrap();
                if let Some("_") = offset.get(end..=end) {
                    offset = offset[end + 1..].to_string();
                } else {
                    offset = offset[end..].to_string();
                }
            }
            if offset.starts_with("b") {
                bit = usize::from_str_radix(&offset[1..], 16).unwrap();
            }

            Location::Adjusted(LocationAdjustment { mode, byte, bit })
        } else {
            Location::Current
        };

        let ty = input.parse()?;
        let mut names = vec![];

        loop {
            names.push(input.parse()?);

            if input.peek(Token![;]) {
                break;
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            location,
            ty,
            names,
        })
    }
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let ident_str = ident.to_string();
        if input.peek(token::Brace) {
            let exprs = Input::parse_block(input)?;

            Ok(Type::Inline(Input { name: ident, exprs }))
        } else if ident_str.starts_with("u") {
            let num = ident_str[1..].parse::<usize>().unwrap();
            Ok(Type::UType(num))
        } else if ident_str.starts_with("i") {
            let num = ident_str[1..].parse::<usize>().unwrap();
            Ok(Type::IType(num))
        } else if ident_str.starts_with("f") {
            let num = ident_str[1..].parse::<usize>().unwrap();
            Ok(Type::FType(num))
        } else {
            Ok(Type::Arb(ident))
        }
    }
}
