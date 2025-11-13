// In a separate crate (e.g., `identifiable_derive`)
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Identifiable, attributes(id))]
pub fn derive_identifiable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Identifiable only supports structs with named fields"),
        },
        _ => panic!("Identifiable only supports structs"),
    };

    // First try to find field with #[id] attribute
    let id_field = fields
        .iter()
        .find(|f| f.attrs.iter().any(|attr| attr.path().is_ident("id")));

    // If not found, try to find field named "id"
    let id_field = id_field.or_else(|| {
        fields
            .iter()
            .find(|f| f.ident.as_ref().map(|i| i == "id").unwrap_or(false))
    });

    let id_field = id_field.expect("No field marked with #[id] or named 'id' found");

    let id_field_name = id_field.ident.as_ref().unwrap();
    let id_field_type = &id_field.ty;

    let expanded = quote! {
        impl Identifiable for #name {
            type Id = #id_field_type;

            fn id(&self) -> Self::Id {
                self.#id_field_name
            }
        }
    };

    TokenStream::from(expanded)
}
