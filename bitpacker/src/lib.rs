use std::fmt::Debug;

use syn::{braced, bracketed, parse::Parse, parse_macro_input, token, Ident, Token};

#[proc_macro]
pub fn bitpacked(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(_input as Input);

    proc_macro::TokenStream::default()
}

// Declarations

/// A bitpacked struct definition.
/// ```
/// ...
/// Foo {
///     $ u8 bar;
///     $[x1b2]; // 10-bit padding.
///     $ u4 baz;
///     $ Qux {
///         $ u1 a, b;
///     } Qux;
/// }
/// ...
/// ```
#[derive(Debug)]
struct Input {
    /// The name of this struct, matches the name of the struct being generated.
    name: Ident,
    /// The expressions that define the fields of this struct.
    exprs: Vec<Expr>,
}

/// An expression in a bitpacked struct.
/// Defines a single or multiple variables of the same type.
/// 
/// All expressions start with a `$` and end with a `;`.
/// ```
/// ...
/// $ u8 a, b, c;
/// ...
/// ```
/// Multiple variables can be defined by separating their names with `,`s.
#[derive(Debug)]
struct Expr {
    ty: Type,
    names: Vec<Ident>,
}

/// A valid type for a bitpacked field.
/// This can be a padding type, an arbitrary type, an inline type, or a variably-sized numerical type.
#[derive(Debug)]
enum Type {
    /// Padding is interpreted as a type for convenience sake.
    /// Padding must follow the one of the formats below (spacing is ignored, and # denotes a hexadecimal number):
    /// ```
    /// $[x#];
    /// $[x#b#];
    /// $[b#];
    /// ```
    Padding(PaddingSize),
    /// Any non-standard type.
    /// ```
    /// $ Foo foo; // Foo must be a bitpacked type.
    /// ```
    Arb(Ident),
    /// A bitpacked struct, defined inline in-place of the type.
    /// ```
    /// $ Foo {
    ///    $ u8 a;
    /// } foo;
    /// ```
    Inline(Input),
    /// Variably-sized numerical unsigned type.
    /// ```
    /// $ u4 n;
    /// ```
    UType(usize),
    /// Variably-sized numerical signed type.
    /// ```
    /// $ i4 n;
    /// ```
    IType(usize),
    /// Variably-sized numerical float type.
    /// ```
    /// $ f4 n;
    /// ```
    /// TODO: Not sure if I want this to be implemented, as it is quite niche.
    FType(usize),
}

/// A bit-aligned padding offset.
#[derive(Debug)]
struct PaddingSize {
    byte: usize,
    bit: usize,
}

// Parsing

/// Parses a list of expressions between braces.
/// ```
/// ... {
///     $ u4 a, b, c;
///     $ [b2];
/// } ...
/// ```
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


/// Trys to parse a hexadecimal number from a string.
fn try_parse_hex(input: &str) -> Result<usize, ()> {
    usize::from_str_radix(input, 16).map_err(|_| ())
}

// Parses a hexadecimal number from a string.
fn parse_hex(input: &str) -> usize {
    usize::from_str_radix(input, 16).map_err(|_| ())
        .expect(&format!("'{}' is not a valid hexadecimal number!", input))
}

fn parse_padding(input: syn::parse::ParseStream) -> syn::Result<Type> {
    let location;
    bracketed!(location in input);

    let mut offset = location.parse::<Ident>()?.to_string();
    
    let byte = if offset.starts_with("x") {
        let end = offset.find("_").unwrap_or(offset.len() - 1) + 1;
        let n = parse_hex(&offset[1..end]);
        offset = offset[end..].to_string();

        n
    } else { 0 };

    let bit = if offset.starts_with("b") {
        parse_hex(&offset[1..])
    } else { 0 };

    Ok(Type::Padding(PaddingSize { byte, bit }))
}

impl Parse for Type {
    #[rustfmt::skip]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
            return match try_parse_hex(&ident_str[1..]) {
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
