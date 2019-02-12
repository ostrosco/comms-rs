#![recursion_limit = "128"]
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
/// Creates a node derived from an input structure with a constructor and
/// implements the Node trait.
///
/// Takes a given structure and from it derives a node-based structure
/// with the sending and receiving out of the node hidden from the user so
/// the user can just focus on the implementation. This function does some
/// transformations on the input data structure's types depending on the name
/// of the fields in the structure. 
///
/// This macro currently assumes that the structure in question implements a
/// function called `run()` which is exercised in the `call()` function from
/// the Node trait. 
///
/// Fields that are receivers must be of type NodeReceiver<T>. Fields that are
/// senders must be of type NodeSender<T>.
///
/// Example:
/// ```no_run
/// node_derive!(
///     pub struct<T> Node1<T> where T: Into<u32> {
///         input: NodeReceiver<T>,
///         internal_state: u32,
///         output: NodeSender<T>,
///     }
/// );
/// ```
///  
pub fn node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let data = &input.data;
    let mut recv_fields = vec![];
    let mut send_fields = vec![];
    let mut state_fields = vec![];
    match data {
        syn::Data::Struct(data_struct) => {
            match &data_struct.fields {
                syn::Fields::Named(fields) => {
                    for field in &fields.named {
                        match parse_type(&field) {
                            FieldType::Input => recv_fields.push(field),
                            FieldType::Output => send_fields.push(field),
                            FieldType::State => state_fields.push(field),
                        }
                    }
                }
                _ => panic!("Node macro only supports named fields."),
            }
        }
        _ => panic!("Node macro only supports structs."),
    }

    let recv_idents: Vec<syn::Ident> = recv_fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .collect();

    let send_idents: Vec<syn::Ident> = send_fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .collect();

    let state_idents: Vec<syn::Ident> = state_fields
        .iter()
        .map(|x| x.ident.clone().unwrap())
        .collect();

    // In order to stop quote from moving any variables and from complaining
    // about duplicates bindings in the macros, we need to build references for
    // each field we need.
    let recv_idents1 = &recv_idents;
    let recv_idents2 = &recv_idents;
    let recv_idents3 = &recv_idents;
    let send_idents1 = &send_idents;
    let state_fields1 = &state_fields;
    let state_idents1 = &state_idents;

    let struct_def = quote! {
        pub struct #name #ty_generics #where_clause {
            #(#send_fields,)*
            #(#recv_fields,)*
            #(#state_fields1,)*
        }
    };

    let new_impl = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(#(#state_fields1,)*) -> #name #ty_generics {
                #name {
                    #(#recv_idents1: None,)*
                    #(#send_idents1: vec![],)*
                    #(#state_idents1,)*
                }
            }
        }
    };

    let derive_node = quote! {
        impl #impl_generics Node for #name #ty_generics #where_clause {
            fn call(&mut self) -> Result<(), NodeError> {
                #(
                    let #recv_idents1 = match self.#recv_idents2 {
                        Some(ref r) => r.recv().unwrap(),
                        None => return Err(NodeError::PermanentError),
                    };
                )*
                let res = self.run(#(#recv_idents3),*);
                #(
                    for (send, _) in &self.#send_idents1 {
                        send.send(res.clone());
                    }
                )*
                Ok(())
            }
        }
    };

    let macro_out = quote! {
        #struct_def
        #new_impl
        #derive_node
    };
    macro_out.into()
}

fn parse_type(field: &syn::Field) -> FieldType {
    let ty = &field.ty;
    let type_str = quote!{#ty}.to_string();
    if type_str.starts_with("NodeReceiver") {
        FieldType::Input
    } else if type_str.starts_with("NodeSender") {
        FieldType::Output
    } else {
        FieldType::State
    }
}
