use std::fmt::Debug;

use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, quote, ToTokens, TokenStreamExt};
use syn::{braced, parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, token, Attribute, Ident, Token, Visibility};

#[proc_macro]
pub fn bitpacked(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // eprintln!("{_attr}");
    // eprintln!("{_input}");

    let input = parse_macro_input!(_input as BitpackedStruct);
    eprintln!("{:?}", input);

    proc_macro::TokenStream::from(input.to_token_stream())
}

struct BitpackedStruct {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    fields: Vec<BitpackedField>
}

impl Debug for BitpackedStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitpackedStruct {{ ident: {:?}, fields: {:?} }}", self.ident, self.fields.iter().collect::<Vec<_>>())
    }
}

struct BitpackedField {
    vis: Visibility,
    ident: Ident,
    ty: BitpackedType
}

impl Debug for BitpackedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitpackedField {{ vis, ident: {:?}, ty: {:?} }}", self.ident, self.ty)
    }
}

enum BitpackedType {
    Std(Ident),
    U(UType),
    LocalPointer(Box<BitpackedType>)
}

impl Debug for BitpackedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitpackedType::Std(ident) => write!(f, "BitpackedType::Std({})", ident),
            BitpackedType::U(ut) => write!(f, "BitpackedType::U({:?})", ut),
            BitpackedType::LocalPointer(t) => write!(f, "BitpackedType::LocalPointer({:?})", t)
        }
    }
}

struct UType {
    size: usize,
    backing: Ident
}

impl Debug for UType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UType {{ size: {}, backing: {} }}", self.size, self.backing)
    }
}

impl Parse for BitpackedStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.call(syn::Visibility::parse)?;

        input.parse::<syn::token::Struct>()?;
        let ident = input.parse::<syn::Ident>()?;

        let inner;
        braced!(inner in input);
        
        let fields = inner.parse_terminated(BitpackedField::parse, syn::token::Comma)?.into_iter().collect();

        Ok(Self {
            attrs,
            vis,
            ident,
            fields
        })
    }
}

impl ToTokens for BitpackedStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let BitpackedStruct { attrs, vis, ident, fields } = self;

        quote! {
            #(#attrs)*
            #vis struct #ident {
                #(#fields),*
            }
        }.to_tokens(tokens);
    }
}

impl Parse for BitpackedField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.call(syn::Visibility::parse)?;
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<BitpackedType>()?;

        Ok(Self {
            vis,
            ident,
            ty
        })
    }
}

impl ToTokens for BitpackedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let BitpackedField { vis, ident, ty } = self;
        
        quote! {
            #vis #ident: #ty
        }.to_tokens(tokens)
    }
}

impl Parse for BitpackedType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![<]>().is_ok() {
            let pointer = BitpackedType::LocalPointer(input.parse()?);
            input.parse::<Token![>]>()?;
            Ok(pointer)
        } else {
            let ident = input.parse::<Ident>()?;
            if let Some(captures) = regex::Regex::new(r"u(\d+)").unwrap().captures(ident.to_string().as_str()) {
                let size = captures.get(1).unwrap().as_str().parse().unwrap();
                let backing = match size {
                    1..=8 => Ident::new("u8", ident.span()),
                    9..=16 => Ident::new("u16", ident.span()),
                    17..=32 => Ident::new("u32", ident.span()),
                    33..=64 => Ident::new("u64", ident.span()),
                    65..=128 => Ident::new("u128", ident.span()),
                    _ => return Err(syn::Error::new(ident.span(), "Invalid size!"))
                };

                Ok(BitpackedType::U(UType {
                    size,
                    backing
                }))
            } else {
                Ok(BitpackedType::Std(ident))
            }
        }
    }
}

impl ToTokens for BitpackedType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BitpackedType::Std(ty) => ty.to_tokens(tokens),
            BitpackedType::U(UType { backing, .. }) => {
                quote_spanned! { Span::call_site() =>
                    #backing
                }.to_tokens(tokens);
            },
            BitpackedType::LocalPointer(_) => {
                quote_spanned! { Span::call_site() =>
                    usize
                }.to_tokens(tokens);
            }
        }
    }
}