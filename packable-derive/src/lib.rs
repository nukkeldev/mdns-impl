use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Packable, attributes(size))]
pub fn packable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let packable_impl = match input.data {
        Data::Struct(data) => match data.fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
                let (names, upcks): (Vec<&syn::Ident>, Vec<proc_macro2::TokenStream>) = named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    let ty = &f.ty;

                    let size = f.attrs.iter().filter_map(|f| 
                        if f.path().is_ident("size") {
                            match &f.meta {
                                syn::Meta::NameValue(nv) => Some(&nv.value),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    ).next();

                    let upck = if let Some(size) = size {
                        quote! {
                            let #name = (0..#size).map(|_| <#ty>::unpack(data).unwrap()).collect::<Vec<_>>();
                        }
                    } else {
                        quote! {
                            let #name = <#ty>::unpack(data)?;
                        }
                    };

                    (name, upck)
                }).unzip();

                quote! {
                    fn pack(&self) -> crate::Data {
                        let mut out = crate::Data::new();
                        #(out.extend(self.#names.pack());)*
                        out
                    }

                    fn unpack(data: &mut crate::Data) -> anyhow::Result<Self> {
                        #(
                            #upcks
                        )*

                        Ok(#name {
                            #(#names),*
                        })
                    }
                }
            }
            syn::Fields::Unnamed(_) => unimplemented!("Tuple structs are not yet supported."),
            syn::Fields::Unit => quote! {},
        },
        Data::Enum(_) => unimplemented!("Enums are not yet supported!"),
        Data::Union(_) => unimplemented!("Unions are not yet supported!"),
    };

    let output = quote! {
        impl crate::packets::pack::Packable for #name {
            #packable_impl
        }
    };

    output.into()
}