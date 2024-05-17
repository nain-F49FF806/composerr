use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Ident, ImplItem, Item, ItemFn, ItemImpl,
    ItemTrait, TraitItem,
};

#[proc_macro_attribute]
pub fn generate_enum(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input into a syntax tree
    let mut ast = parse_macro_input!(input as syn::Item);

    // Check if the input is a function, trait def or an impl block
    let (enum_name, functions) = match &mut ast {
        Item::Trait(trait_def) => process_trait_def(trait_def),
        Item::Impl(impl_block) => process_impl_block(impl_block),
        Item::Fn(function) => process_bare_function(function),
        _ => panic!("This macro can only be used on functions, traits or implementations."),
    };

    // Generate the enum variants
    let variants = functions.iter().map(|(i, _error_set)| {
        let error_names = _error_set.iter().map(|i| i.to_string()).collect::<String>();
        let capital_name = capitalize(i.to_string() + &error_names);

        // let capital_name = capitalize(i.to_string());
        Ident::new(&capital_name, i.span())
    });

    let mut enums = Vec::new();
    for (i, error_set) in &functions {
        let enum_name = capitalize(i.to_string()) + "Error";
        let enum_ident = Ident::new(&enum_name, i.span());

        let derive_attr = quote!(#[derive(Error, Debug)]);
        let from_attr = quote!(#[from]);
        let transparent_attr = quote!(#[error(transparent)]);

        enums.push(quote! {
            #derive_attr
            pub enum #enum_ident {
                #(
                    #transparent_attr
                    #error_set(#from_attr #error_set)
                ),*
            }

            #(
                impl TryFrom<#enum_ident> for #error_set {
                    type Error = String;
                    fn try_from(value: #enum_ident) -> Result<Self, Self::Error> {
                        match value {
                            #enum_ident::#error_set(e) => Ok(e),
                            _ => Err(
                                    concat!(
                                        "This instance of ", stringify!(#enum_ident),
                                        " is of variant different than the requested ", stringify!(#error_set)
                                    ).to_string()
                                ),
                        }
                    }
                }
            )*

        });
    }

    // Return the generated code
    TokenStream::from(quote! {
        #(#enums)*

        #ast
    })
}

type FuncErrors = (Ident, Vec<Ident>);
type ContextFuncs = (Ident, Vec<FuncErrors>);

fn process_trait_def(trait_def: &mut ItemTrait) -> ContextFuncs {
    // For a trait, use the trait name as the enum name
    let enum_name = Ident::new(
        &(trait_def.ident.to_string() + "TraitEnum"),
        trait_def.ident.span(),
    );
    let functions = extract_trait_functions(trait_def);
    strip_trait_functions_attrs(trait_def);
    (enum_name, functions)
}

fn process_impl_block(impl_block: &mut ItemImpl) -> ContextFuncs {
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

fn process_bare_function(function: &mut ItemFn) -> ContextFuncs {
    // For bare function, use it's own name as the enum name
    let capital_name = capitalize(function.sig.ident.to_string());
    let enum_name = Ident::new(&capital_name, function.sig.span());
    let functions = extract_bare_function(function);
    strip_bare_function_attrs(function);
    (enum_name, functions)
}

fn extract_trait_functions(trait_def: &ItemTrait) -> Vec<FuncErrors> {
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
                .any(|attr| attr.path().get_ident().unwrap() == "errorset")
        })
        .map(|item| {
            let func_name = item.sig.ident.clone();
            let errorset_attr = item
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("errorset"))
                .unwrap();
            let err_set: Vec<Ident> = extract_errorset_list(errorset_attr);
            (func_name, err_set)
        })
        .collect()
}

fn extract_impl_functions(impl_block: &ItemImpl) -> Vec<FuncErrors> {
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
                .any(|attr| attr.path().segments.last().unwrap().ident == "errorset")
        })
        .map(|item| {
            let func_name = item.sig.ident.clone();
            let errorset_attr = item
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("errorset"))
                .unwrap();
            let err_set: Vec<Ident> = extract_errorset_list(errorset_attr);
            (func_name, err_set)
        })
        .collect()
}

fn extract_bare_function(function: &ItemFn) -> Vec<FuncErrors> {
    if function
        .attrs
        .iter()
        .any(|attr| attr.path().segments.last().unwrap().ident == "errorset")
    {
        let func_name = function.sig.ident.clone();
        let errorset_attr = function
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("errorset"))
            .unwrap();
        let err_set: Vec<Ident> = extract_errorset_list(errorset_attr);

        vec![(func_name, err_set)]
    } else {
        vec![]
    }
}

fn extract_errorset_list(attr: &Attribute) -> Vec<Ident> {
    let mut idents = Vec::new();
    attr.parse_nested_meta(|meta| {
        let ident = meta
            .path
            .get_ident()
            .expect("Each item must be an ident, not long path");
        idents.push(ident.clone());
        Ok(())
    })
    .expect("Failed parsing args for errorset helper attribute");
    idents
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
                    .retain(|attr| attr.path().segments.last().unwrap().ident != "errorset");
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
                    .retain(|attr| attr.path().segments.last().unwrap().ident != "errorset");
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
        .retain(|attr| attr.path().segments.last().unwrap().ident != "errorset");
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
