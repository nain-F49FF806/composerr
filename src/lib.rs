use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ImplItem, Item, ItemImpl, ItemTrait, TraitItem};

#[proc_macro_attribute]
pub fn generate_enum(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input into a syntax tree
    let ast = parse_macro_input!(input as syn::Item);

    // Check if the input is a trait or an impl block
    let (enum_name, functions) = match ast {
        Item::Trait(trait_def) => {
            // For a trait, use the trait name as the enum name
            let enum_name = trait_def.ident.clone();
            let functions = extract_trait_functions(&trait_def);
            (enum_name, functions)
        }
        Item::Impl(impl_block) => {
            // For an implementation, use the struct name as the enum name
            let ident = match &*impl_block.self_ty {
                syn::Type::Path(tp) => tp.path.segments.first().unwrap().ident.clone(),
                _ => panic!("not supported tokens"),
            };
            let enum_name = Ident::new(&(ident.to_string() + "ImplEnum"), ident.span());
            // let enum_name = Ident::new("Unknown", impl_block.span());
            let functions = extract_impl_functions(&impl_block);
            (enum_name, functions)
        }
        _ => panic!("This macro can only be used on traits or implementations."),
    };

    // Generate the enum variants
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

fn extract_trait_functions(trait_def: &ItemTrait) -> Vec<Ident> {
    trait_def
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Fn(item) => Some(item.sig.ident.clone()),
            _ => None,
        })
        .collect()
}

fn extract_impl_functions(impl_block: &ItemImpl) -> Vec<Ident> {
    impl_block
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Fn(item) => Some(item.sig.ident.clone()),
            _ => None,
        })
        .collect()
}
