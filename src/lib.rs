use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Ident, ImplItem, Item, ItemFn, ItemImpl, ItemTrait,
    TraitItem,
};

#[proc_macro_attribute]
pub fn generate_enum(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input into a syntax tree
    let mut ast = parse_macro_input!(input as syn::Item);

    // Check if the input is a function, trait def or an impl block
    let (enum_name, functions) = match &mut ast {
        Item::Trait(trait_def) => {
            // For a trait, use the trait name as the enum name
            let enum_name = Ident::new(
                &(trait_def.ident.to_string() + "TraitEnum"),
                trait_def.ident.span(),
            );
            let functions = extract_trait_functions(trait_def);
            strip_trait_functions_attrs(trait_def);
            (enum_name, functions)
        }
        Item::Impl(impl_block) => {
            // For an implementation, use the type name as the enum name
            let ident = match &*impl_block.self_ty {
                syn::Type::Path(tp) => tp.path.segments.last().unwrap().ident.clone(),
                _ => panic!("not supported tokens"),
            };
            let enum_name = Ident::new(&(ident.to_string() + "ImplEnum"), ident.span());
            // If it's an impl trait, then abort.
            if impl_block.trait_.is_some() {
                panic!("Use this macro on the trait definition, not the implementation.")
            };

            let functions = extract_impl_functions(impl_block);
            strip_impl_functions_attrs(impl_block);
            (enum_name, functions)
        }
        Item::Fn(function) => {
            // For bare function, use it's own name as the enum name
            let capital_name = capitalize(function.sig.ident.to_string());
            let enum_name = Ident::new(&capital_name, function.sig.span());
            let functions = extract_bare_function(function);
            strip_bare_function_attrs(function);
            (enum_name, functions)
        }
        _ => panic!("This macro can only be used on traits or implementations."),
    };

    // Generate the enum variants
    let variants = functions.iter().map(|i| {
        let capital_name = capitalize(i.to_string());
        Ident::new(&capital_name, i.span())
    });

    // Return the generated code
    TokenStream::from(quote! {
        pub enum #enum_name {
            #(#variants),*
        }
        #ast
    })
}

fn extract_trait_functions(trait_def: &ItemTrait) -> Vec<Ident> {
    trait_def
        .items
        .iter()
        // We want only function items
        .filter_map(|item| match item {
            TraitItem::Fn(item) => Some(item),
            _ => None,
        })
        // and only those functions with #[select] attribute
        .filter(|item| {
            item.attrs
                .iter()
                .any(|attr| attr.path().segments.last().unwrap().ident == "select")
        })
        .map(|item| item.sig.ident.clone())
        .collect()
}

// Mutates ItemTrait in place. Removing the #[select] helper attribute
fn strip_trait_functions_attrs(trait_def: &mut ItemTrait) {
    let cleaned_items = trait_def
        .items
        .iter()
        .map(|item| match item {
            TraitItem::Fn(item_fn) => {
                let mut item_fn = item_fn.clone();
                item_fn
                    .attrs
                    .retain(|attr| attr.path().segments.last().unwrap().ident != "select");
                TraitItem::Fn(item_fn)
            }
            _ => item.clone(),
        })
        .collect();
    trait_def.items = cleaned_items;
}

// Mutates ItemImpl in place. Removing the #[select] helper attribute
fn strip_impl_functions_attrs(impl_block: &mut ItemImpl) {
    let cleaned_items = impl_block
        .items
        .iter()
        .map(|item| match item {
            ImplItem::Fn(item_fn) => {
                let mut item_fn = item_fn.clone();
                item_fn
                    .attrs
                    .retain(|attr| attr.path().segments.last().unwrap().ident != "select");
                ImplItem::Fn(item_fn)
            }
            _ => item.clone(),
        })
        .collect();
    impl_block.items = cleaned_items;
}

// Mutates function in place. Removing the #[select] helper attribute
fn strip_bare_function_attrs(function: &mut ItemFn) {
    function
        .attrs
        .retain(|attr| attr.path().segments.last().unwrap().ident != "select");
}

fn extract_impl_functions(impl_block: &ItemImpl) -> Vec<Ident> {
    impl_block
        .items
        .iter()
        // We want only function items
        .filter_map(|item| match item {
            ImplItem::Fn(item) => Some(item),
            _ => None,
        })
        // and only those functions with #[select] attribute
        .filter(|item| {
            item.attrs
                .iter()
                .any(|attr| attr.path().segments.last().unwrap().ident == "select")
        })
        .map(|item| item.sig.ident.clone())
        .collect()
}

fn extract_bare_function(function: &ItemFn) -> Vec<Ident> {
    if function
        .attrs
        .iter()
        .any(|attr| attr.path().segments.last().unwrap().ident == "select")
    {
        vec![function.sig.ident.clone()]
    } else {
        vec![]
    }
}

fn capitalize(name: impl Into<String>) -> String {
    let name: String = name.into();
    name.chars()
        .next()
        .unwrap_or_default()
        .to_uppercase()
        .collect::<String>()
        + &name[1..]
}
