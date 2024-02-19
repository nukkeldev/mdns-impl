use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{braced, bracketed, parse::Parse, parse_macro_input, token, Ident, Token};

#[proc_macro]
pub fn bitpacked(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(_input as Inputs);

    proc_macro::TokenStream::from(input.into_token_stream())
}

// Declarations

/// A list of bitpacked struct definitions.
#[derive(Debug, Clone)]
struct Inputs(Vec<Input>);

/// A bitpacked struct definition.
/// ```ignore
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
#[derive(Debug, Clone)]
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
/// ```ignore
/// ...
/// $ u8 a, b, c;
/// ...
/// ```
/// Multiple variables can be defined by separating their names with `,`s.
#[derive(Debug, Clone)]
struct Expr {
    ty: SpannedType,
    names: Vec<Ident>,
}

/// A spanned type.
#[derive(Debug, Clone)]
struct SpannedType {
    span: Span,
    ty: Type,
}

/// A valid type for a bitpacked field.
/// This can be a padding type, an arbitrary type, an inline type, or a variably-sized numerical type.
#[derive(Debug, Clone)]
enum Type {
    /// Padding is interpreted as a type for convenience sake.
    /// Padding must follow the one of the formats below (spacing is ignored, and # denotes a hexadecimal number):
    /// ```ignore
    /// $[x#];
    /// $[x#b#];
    /// $[b#];
    /// ```
    Padding(PaddingSize),
    /// Any non-standard type.
    /// ```ignore
    /// $ Foo foo; // Foo must be a bitpacked type.
    /// ```
    Arb(Ident),
    /// A bitpacked struct, defined inline in-place of the type.
    /// ```ignore
    /// $ Foo {
    ///    $ u8 a;
    /// } foo;
    /// ```
    Inline(Input),
    /// Variably-sized numerical unsigned type.
    /// ```ignore
    /// $ u4 n;
    /// ```
    UType(usize),
    /// Variably-sized numerical signed type.
    /// ```ignore
    /// $ i4 n;
    /// ```
    IType(usize),
    /// Variably-sized numerical float type.
    /// ```ignore
    /// $ f4 n;
    /// ```
    /// TODO: Not sure if I want this to be implemented, as it is quite niche.
    FType(usize),
}

/// A bit-aligned padding offset.
#[derive(Debug, Clone)]
struct PaddingSize {
    byte: usize,
    bit: usize,
}

// Parsing

/// Parses a list of expressions between braces.
/// ```ignore
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

impl Parse for Inputs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut inputs = vec![];

        while !input.is_empty() {
            inputs.push(input.parse()?);
        }

        Ok(Self(inputs))
    }
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

impl Parse for SpannedType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let ty = input.parse()?;

        Ok(Self { span, ty })
    }
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
            return match ident_str[1..].parse() {
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

impl ToTokens for Inputs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let inputs = &self.0;

        tokens.extend(quote! {
            #(#inputs)*
        });
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let exprs = &self.exprs;
        
        let getters = self.exprs.iter().flat_map(|expr| expr.getters()).collect::<Vec<_>>();
        let setters = self.exprs.iter().flat_map(|expr| expr.setters()).collect::<Vec<_>>();

        let packed_sizes = exprs.iter().map(|expr| {
            match &expr.ty.ty {
                Type::Arb(ident) => quote! {
                    <#ident>::packed_size()
                },
                _ => {
                    let size = expr.packed_size();
                    quote! {
                        #size
                    }
                }
            }
        }).collect::<Vec<_>>();

        tokens.extend(quote! {
            struct #name {
                #(#exprs)*
            }

            impl #name {
                /// The size of the bitpacked struct in bits.
                pub fn packed_size() -> usize {
                    #(#packed_sizes +)* 0
                }

                #(
                    #getters
                    #setters
                )*
            }
        });

        exprs.iter().filter(|expr| matches!(expr.ty.ty, Type::Inline(_))).for_each(|inline| {
            if let Type::Inline(input) = &inline.ty.ty {
                input.to_tokens(tokens);
            }
        });
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = &self.ty;
        let decls = self.names.iter().map(|name| {
            quote! {
                #name: #ty,
            }
        }).collect::<Vec<_>>();

        tokens.extend(decls);
    }
}

impl ToTokens for SpannedType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let SpannedType { span, ty } = self;
        
        match ty {
            Type::Arb(ident) => {
                tokens.extend(quote! {
                    #ident
                });
            }
            Type::Inline(input) => {
                let name = &input.name;
                quote!(#name).to_tokens(tokens);
            }
            Type::UType(num) => {
                let pow = match num {
                    1 => "bool",
                    2..=8 => "u8",
                    9..=16 => "u16",
                    17..=32 => "u32",
                    33..=64 => "u64",
                    65..=128 => "u128",
                    _ => {
                        tokens.extend(quote_spanned! { *span =>
                            compile_error!("Only 128 bits max for unsigned integers are supported!");
                        });
                        return;
                    }
                };
                
                tokens.extend(format_ident!("{}", pow).into_token_stream());
            }
            Type::IType(num) => {
                let pow = match num {
                    1 => "bool",
                    2..=8 => "i8",
                    9..=16 => "i16",
                    17..=32 => "i32",
                    33..=64 => "i64",
                    65..=128 => "i128",
                    _ => {
                        tokens.extend(quote_spanned! { *span =>
                            compile_error!("Only 128 bits max for signed integers are supported!");
                        });
                        return;
                    }
                };
                
                tokens.extend(format_ident!("{}", pow).into_token_stream());
            }
            Type::FType(num) => {
                let pow = match num {
                    8..=32 => "f32",
                    33..=64 => "f64",
                    _ => {
                        tokens.extend(quote_spanned! { *span =>
                            compile_error!("Only 64 bits max for floats are supported!");
                        });
                        return;
                    }
                };

                tokens.extend(format_ident!("{}", pow).into_token_stream());
            }
            _ => {}
        }
    }
}

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

    fn getters(&self) -> Vec<TokenStream> {
        let ty = &self.ty;

        self.names.iter().map(|name| {
            let fn_name = format_ident!("get_{}", name);
            let fn_name_mut = format_ident!("get_{}_mut", name);
            let doc = format!("Returns a reference to the `{}` `{ty}` field.", name);
            let doc_mut = format!("Returns a mutable reference to the `{}` `{ty}` field.", name);

            quote! {
                #[doc = #doc]
                pub fn #fn_name(&self) -> &#ty {
                    &self.#name
                }

                #[doc = #doc_mut]
                pub fn #fn_name_mut(&mut self) -> &mut #ty {
                    &mut self.#name
                }
            }
        }).collect()
    }

    fn setters(&self) -> Vec<TokenStream> {
        let ty = &self.ty;
        let precondition = match &ty.ty {
            Type::UType(num) if *num > 1 => {
                quote! {
                    assert!(value <= (1 << #num) - 1, "Value is too large for a u{}!", #num);
                }
            }
            Type::IType(num) if *num > 1 => {
                quote! {
                    assert!(value <= (1 << (#num - 1)) - 1, "Value is too large for an i{}!", #num);
                    assert!(value >= -(1 << (#num - 1)), "Value is too small for an i{}!", #num);
                }
            }
            _ => quote! {},
        };

        self.names.iter().map(|name| {
            let fn_name = format_ident!("set_{}", name);
            let doc = format!("Sets the `{}` `{ty}` field.", name);

            quote! {
                #[doc = #doc]
                pub fn #fn_name(&mut self, value: #ty) {
                    #precondition
                    self.#name = value;
                }
            }
        }).collect()
    }
}

impl SpannedType {
    /// Returns the size of the bitpacked type in bits.
    fn packed_size(&self) -> usize {
        match &self.ty {
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

// Misc

impl Display for SpannedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Padding(size) => write!(f, "Padding({}b, {}b)", size.byte, size.bit),
            Type::Arb(ident) => write!(f, "{}", ident),
            Type::Inline(input) => write!(f, "{}", input.name),
            Type::UType(num) => write!(f, "u{}", num),
            Type::IType(num) => write!(f, "i{}", num),
            Type::FType(num) => write!(f, "f{}", num),
        }
    }

}

// Tests

#[test]
fn tests() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/*.rs");
}