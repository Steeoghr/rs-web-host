extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, NestedMeta, Signature};

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

    let name = &input.sig.ident;
    let vis = &input.vis;
    let block = &input.block;
    let params = extract_params_from_signature(&input.sig);
    let handler_name = format_ident!("{}_{}_handler", method.to_lowercase(), name);

    let expanded = quote! {
        #vis fn #name(#params) -> String {
            #block
        }

        inventory::submit! {
            crate::Route {
                method: #method.to_string(),
                path: #path.to_string(),
                handler: #name,
                has_body: #has_body,
            }
        }
    };

    expanded.into()
}

fn extract_params_from_signature(sig: &Signature) -> proc_macro2::TokenStream {
    let params = sig.inputs.iter().map(|input| {
        if let syn::FnArg::Typed(pat_type) = input {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;
            quote! {
                #pat: #ty
            }
        } else {
            quote!()
        }
    });

    quote! { #( #params ),* }
}
