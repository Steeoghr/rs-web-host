
pub mod host;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, NestedMeta};



#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream {
    generate_route("GET", args, input, false)
}

#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream {
    generate_route("POST", args, input, true)
}

fn generate_route(method: &str, args: TokenStream, input: TokenStream, has_body: bool) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);
    let path = match args.get(0) {
        Some(NestedMeta::Lit(Lit::Str(lit))) => lit.value(),
        _ => panic!("Expected path as first argument"),
    };
    let params = extract_params_from_path(&path);

    let name = &input.sig.ident;
    let vis = &input.vis;
    let block = &input.block;

    let expanded = quote! {
        #vis fn #name(params: &[(&str, &str)], body: Option<String>) -> String {
            #block
        }

        // Store the route information
        inventory::submit! {
            crate::Route {
                method: #method.to_string(),
                path: format!("{}{}", <Self as Controller>::base_path(), #path),
                handler: #name,
                params: vec![#(#params.to_string()),*],
                has_body: #has_body,
            }
        }
    };

    expanded.into()
}

fn extract_params_from_path(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                Some(segment.trim_matches(&['{', '}'][..]).to_string())
            } else {
                None
            }
        })
        .collect()
}