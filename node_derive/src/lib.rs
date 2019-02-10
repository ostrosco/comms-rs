#![recursion_limit="128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

enum FieldType {
    Input,
    Output,
    State,
}

#[proc_macro]
pub fn node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let data = &input.data;
    let body; 
    let mut recv_fields = vec![];
    let mut send_fields = vec![];
    match data {
        syn::Data::Struct(data_struct) => {
            body = data_struct;
            match &data_struct.fields {
                syn::Fields::Named(fields) => {
                    for field in &fields.named {
                        match parse_type(&field) {
                            FieldType::Input => {
                                recv_fields.push(field)
                            }
                            FieldType::Output => {
                                send_fields.push(field)
                            }
                            _ => (),
                        }
                    }
                }
                _ => panic!("Node macro needs named fields."),
            }
        },
        _ => panic!("Node macro only supports structures."),
    }
    let body_fields = &body.fields;

    let recv_idents1: Vec<syn::Ident> = recv_fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .map(|x| syn::Ident::new(&format!("{}", x), x.span()))
        .collect();

    let receivers: Vec<syn::Ident> = recv_fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .map(|x| syn::Ident::new(&format!("recv_{}", x), x.span()))
        .collect();

    let recv_idents2 = recv_idents1.clone();
    let recv_idents3 = recv_idents1.clone();

    let send_idents: Vec<syn::Ident> = send_fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .map(|x| syn::Ident::new(&format!("{}", x), x.span()))
        .collect();

    let macro_out = quote! {
        pub #impl_generics struct #name #ty_generics #where_clause {
            #(#body_fields)*
        }

        impl #impl_generics comms_rs::node::Node for #name #ty_generics #where_clause {
            fn call(&mut self) -> Result<(), NodeError> {
                #(
                    let #recv_idents2 = match self.#recv_idents1 {
                        Some(ref r) => r.recv().unwrap(),
                        None => return Err(NodeError::PermanentError),
                    };
                )*
                let res = self.run(#(#recv_idents3),*);
                #(
                    for (send, _) in &self.#send_idents {
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
    let ident = &field.ident;
    match ident {
        Some(ref id) if id.to_string().starts_with("recv") => {
            FieldType::Input
        }
        Some(ref id) if id.to_string().starts_with("send") => {
            FieldType::Output
        } 
        _ => FieldType::State,
    }
}
