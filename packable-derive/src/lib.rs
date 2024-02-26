use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Packable)]
pub fn packable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(data) => {}
        Data::Enum(data) => unimplemented!("Enums are not yet supported!"),
        Data::Union(data) => unimplemented!("Unions are not yet supported!"),
    }

    todo!()
}
