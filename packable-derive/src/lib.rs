use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Packable, attributes(size, post_process))]
pub fn packable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let packable_impl = match input.data {
        Data::Struct(data) => match data.fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
                let (names, upcks): (Vec<&syn::Ident>, Vec<proc_macro2::TokenStream>) = named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    let ty = &f.ty;

                    let size = f.attrs.iter().find_map(|f| {
                        if f.path().is_ident("size") {
                            f.parse_args::<syn::Expr>().ok()
                        } else {
                            None
                        }
                    });

                    let upck = if let Some(size) = size {
                        let inner_ty = ty.to_token_stream().into_iter().nth(2);
                        quote! {
                            let #name = (0..#size).map(|_| <#inner_ty>::unpack(data).unwrap()).collect::<Vec<_>>();
                        }
                    } else {
                        quote! {
                            let #name = <#ty>::unpack(data)?;
                        }
                    };

                    (name, upck)
                }).unzip();

                let post_process = input.attrs.iter().find_map(|f| {
                    if f.path().is_ident("post_process") {
                        f.parse_args::<syn::Expr>()
                            .ok()
                            .map(|e| e.into_token_stream())
                    } else {
                        None
                    }
                });

                let new_fn = if let Some(post) = post_process {
                    quote! {
                        Ok(#post(post_data, #(#names),*))
                    }
                } else {
                    quote! {
                        Ok(#name {
                            #(#names),*
                        })
                    }
                };

                quote! {
                    fn pack(&self) -> crate::Data {
                        let mut out = crate::Data::new();
                        #(out.extend(self.#names.pack());)*
                        out
                    }

                    fn unpack(data: &mut crate::Data) -> anyhow::Result<Self> {
                        let post_data = data.clone();

                        #(
                            #upcks
                        )*

                        #new_fn
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
