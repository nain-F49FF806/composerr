use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemTrait, TraitItem};

#[proc_macro_attribute]
pub fn generate_enum_from_trait(_: TokenStream, trait_def: TokenStream) -> TokenStream {
    // Parse the trait definition
    let trait_def: ItemTrait = parse_macro_input!(trait_def as ItemTrait);

    // Extract function signatures from the trait
    let functions = extract_functions(&trait_def);

    // Generate the enum
    let enum_name = &trait_def.ident;
    let variants = functions.iter().map(|i| {
        let name = i.to_string();
        let capital_name = i
            .to_string()
            .chars()
            .next()
            .unwrap_or_default()
            .to_uppercase()
            .collect::<String>()
            + &name[1..];
        Ident::new(&capital_name, i.span())
    });

    // Return the generated code
    TokenStream::from(quote! {
        pub enum #enum_name {
            #(#variants),*
        }
    })
}

fn extract_functions(trait_def: &ItemTrait) -> Vec<Ident> {
    trait_def
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Fn(item) => Some(item.sig.ident.clone()),
            _ => None,
        })
        .collect()
}
