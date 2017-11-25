extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate quote;

use std::env;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::*;

#[proc_macro_derive(Configure, attributes(configure))]
pub fn derive_configure(input: TokenStream) -> TokenStream {
    let ast = parse_derive_input(&input.to_string()).unwrap();
    let gen = impl_configure(ast);
    gen.parse().unwrap()
}

fn impl_configure(ast: DeriveInput) -> Tokens {
    let ty = &ast.ident;
    let generics = &ast.generics;
    let project = ast.attrs.iter()
                     .filter_map(|attr| project_name(&attr.value))
                     .next().or_else(|| env::var("CARGO_PKG_NAME").ok()).unwrap();

    quote!{
        impl #generics ::configure::Configure for #ty #generics {
            fn generate() -> Result<Self, ::configure::DeserializeError> {
                let deserializer = ::configure::source::CONFIGURATION.get(#project);
                ::serde::Deserialize::deserialize(deserializer)
            }
        }
    }
}

fn project_name(attr: &MetaItem) -> Option<String> {
    if attr.name() != "configure" { return None }

    if let MetaItem::List(_, ref members) = *attr {
        if members.len() == 1 {
            if let NestedMetaItem::MetaItem(ref meta_item) = members[0] {
                if meta_item.name() == "name" {
                    if let MetaItem::NameValue(_, ref name) = *meta_item {
                        if let Lit::Str(ref string, _) = *name {
                            return Some(string.clone())
                        }
                    }
                }
            }
        }
    } 
    panic!("Unsupported `configure` attribute. Only supported attribute is #[configure(name = \"$NAME\")].")
}
