use std::{collections::HashMap, sync::Mutex};
use inventory::iter;
use rs_macro_di::provider::{get_service_provider, ServiceProvider};

#[derive(Clone)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: fn(&[(&str, &str)], Option<String>) -> String,
    pub params: Vec<String>,
    pub has_body: bool,
}

inventory::collect!(Route);

pub struct WebHost {
    provider: &'static Mutex<ServiceProvider>,
    routes: HashMap<String, Route>,
}

impl WebHost {
    pub fn new<T: IStartup + 'static>() -> Self {
        let provider = get_service_provider();

        {
            let mut sp = provider.lock().unwrap();
            T::configure_services(&mut sp);
        }

        let mut web_host = WebHost {
            provider,
            routes: HashMap::new(),
        };

        web_host.add_controllers();
        web_host
    }

    pub fn add_controllers(&mut self) {
        for route in iter::<Route> {
            let key = format!("{}:{}", route.method, route.path);
            self.routes.insert(key, route.clone());
        }
    }

    fn extract_params<'a>(&self, path: &'a str, route: &'a Route) -> Vec<(&'a str, &'a str)> {
        let mut params = Vec::new();
        let path_parts: Vec<&str> = path.split('/').collect();
        let route_parts: Vec<&str> = route.path.split('/').collect();
        for (route_part, path_part) in route_parts.iter().zip(path_parts.iter()) {
            if route_part.starts_with('{') && route_part.ends_with('}') {
                params.push((route_part.trim_matches(&['{', '}'][..]), *path_part));
            }
        }
        params
    }

    pub fn handle_request(&self, method: &str, path: &str, body: Option<String>) -> Option<String> {
        let key = format!("{}:{}", method, path);
        for (route_key, route) in &self.routes {
            if route_key.starts_with(&key) {
                let params = self.extract_params(path, route);
                let response = Some((route.handler)(&params, body.clone()));
                self.provider.lock().unwrap().clear_scoped_instances();
                return response
            }
        }
        None
    }

    pub fn start(&self) {
        let routes = self.routes.clone(); // Clone the routes for the new thread
        std::thread::spawn(move || {
            // Simulate a request handling loop
            for (key, route) in &routes {
                let response = (route.handler)(&[], None);
                println!("Handled {}: {}", key, response);
            }
        }).join().unwrap();
    }
}

pub trait IStartup {
    fn configure_services(provider: &mut ServiceProvider);
}





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