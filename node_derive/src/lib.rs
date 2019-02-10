extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

enum FieldType {
    Input,
    Output,
    State,
}

#[proc_macro_derive(Node, attributes(recv, send))]
pub fn node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let data = &input.data;
    let mut recv_fields = vec![];
    let mut send_fields = vec![];
    match data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    match parse_type(&field) {
                        FieldType::Input => {
                            recv_fields.push(field.ident.clone().unwrap())
                        }
                        FieldType::Output => {
                            send_fields.push(field.ident.clone().unwrap())
                        }
                        _ => (),
                    }
                }
            }
            _ => panic!("Node macro needs named fields."),
        },
        _ => panic!("Node macro only supports structures."),
    }

    let recv_idents1: Vec<syn::Ident> = recv_fields
        .iter()
        .map(|x| syn::Ident::new(&format!("_{}", x), x.span()))
        .collect();
    let recv_idents2 = recv_idents1.clone();
    let macro_out = quote! {
        impl #impl_generics comms_rs::node::Node for #name #ty_generics #where_clause {
            fn call(&mut self) -> Result<(), NodeError> {
                #(
                    let #recv_idents1 = match self.#recv_fields {
                        Some(ref r) => r.recv().unwrap(),
                        None => return Err(NodeError::PermanentError),
                    };
                )*
                let res = self.run(#(#recv_idents2),*);
                #(
                    for (send, _) in &self.#send_fields {
                        send.send(res.clone());
                    }
                )*
                Ok(())
            }
        }
    };
    macro_out.into()
}

fn parse_type(field: &syn::Field) -> FieldType {
    for attr in field.attrs.iter() {
        let meta = attr.parse_meta().unwrap();
        match meta {
            syn::Meta::Word(ref ident) if ident == "recv" => {
                return FieldType::Input;
            }
            syn::Meta::Word(ref ident) if ident == "send" => {
                return FieldType::Output;
            }
            _ => panic!("Unsupported attribute on structure.")
        }
    }
    FieldType::State
}
