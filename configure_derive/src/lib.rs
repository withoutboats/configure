extern crate heck;
extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate quote;

mod attrs;

use std::env;
use std::fmt::Write;

use heck::ShoutySnakeCase;
use proc_macro::TokenStream;
use quote::Tokens;
use syn::*;

use attrs::{CfgAttrs, FieldAttrs};

#[proc_macro_derive(Configure, attributes(configure))]
pub fn derive_configure(input: TokenStream) -> TokenStream {
    let ast = parse_derive_input(&input.to_string()).unwrap();
    let gen = impl_configure(ast);
    gen.parse().unwrap()
}

fn impl_configure(ast: DeriveInput) -> Tokens {
    let ty = &ast.ident;
    let generics = &ast.generics;
    let cfg_attrs = CfgAttrs::new(&ast.attrs[..]);
    let fields = assert_ast_is_struct(&ast);
    let project = cfg_attrs.name.or_else(|| env::var("CARGO_PKG_NAME").ok()).unwrap();
    let docs = if cfg_attrs.docs { Some(docs(fields, &project)) } else { None };

    quote!{
        impl #generics ::configure::Configure for #ty #generics {
            fn generate() -> ::std::result::Result<Self, ::configure::DeserializeError> {
                let deserializer = ::configure::source::CONFIGURATION.get(#project);
                ::serde::Deserialize::deserialize(deserializer)
            }
        }

        #docs
    }
}

fn assert_ast_is_struct(ast: &DeriveInput) -> &[Field] {
    match ast.body {
        Body::Struct(VariantData::Struct(ref fields))   => fields,
        Body::Struct(VariantData::Unit)                 => &[],
        Body::Struct(VariantData::Tuple(_))             => {
            panic!("Cannot derive `Configure` for tuple struct")
        }
        Body::Enum(_)                                   => {
            panic!("Cannot derive `Configure` for enum")
        }
    }
}

fn docs(fields: &[Field], project: &str) -> Tokens {
    let mut docs = format!("These environment variables can be used to configure {}.\n\n", project);
    for field in fields {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let attrs = FieldAttrs::new(field);

        let var_name = format!("{}_{}", project, name).to_shouty_snake_case();
        let var_type = quote! { #ty };

        if let Some(field_docs) = attrs.docs {
            let _ = writeln!(docs, "- **{}** ({}): {}", var_name, var_type, field_docs);
        } else {
            let _ = writeln!(docs, "- **{}** ({})", var_name, var_type);
        }
    }

    docs.push_str("\nThis library uses the configure crate to manage its configuration; you can\
                     also override how configuration is handled using the API in that crate.");

    quote! {
        #[doc = #docs]
        pub mod environment_variables { }
    }
}
