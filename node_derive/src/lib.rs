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

#[proc_macro_derive(Node, attributes(aggregate, pass_by_ref))]
/// Creates a node derived from an input structure with a constructor and
/// implements the Node trait.
///
/// Takes a given structure and from it derives a node-based structure
/// with the sending and receiving out of the node hidden from the user so
/// the user can just focus on the implementation. This function does some
/// transformations on the input data structure's types depending on the name
/// of the fields in the structure.
///
/// The implementation of the Node trait depends on whether #[aggregate] is
/// specified on the structure or not. If it is, the Node will only send out
/// data when it gets a Some(T) from the run() function, otherwise it will
/// continue to the next iteration.
///
/// This macro currently assumes that the structure in question implements a
/// function called `run()` which is exercised in the `call()` function from
/// the Node trait. The return type of the run() depends on whether the
/// #[aggregate] flag is specified on the input structure. If #[aggregate] is
/// present on the structure, the return type must be an Option. Otherwise, it
/// can be anything.
///
/// Fields that are receivers must be of type NodeReceiver<T>. Fields that are
/// senders must be of type NodeSender<T>.
///
/// Example:
/// ```no_run
/// #[derive(Node)]
/// pub struct Node1<T> where T: Into<u32> {
///     input: NodeReceiver<T>,
///     internal_state: u32,
///     output: NodeSender<T>,
/// }
/// ```
///  
pub fn node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let attributes = &input.attrs;
    let mut aggregate = false;
    let mut pass_by_ref = false;
    for attr in attributes {
        match attr.parse_meta() {
            Ok(syn::Meta::Word(ref id)) if id.to_string() == "aggregate" => {
                aggregate = true
            }
            Ok(syn::Meta::Word(ref id)) if id.to_string() == "pass_by_ref" => {
                pass_by_ref = true
            }
            Ok(_) => continue,
            Err(e) => panic!("Invalid attribute {}", e),
        }
    }

    let data = &input.data;
    let mut recv_fields = vec![];
    let mut send_fields = vec![];
    let mut state_fields = vec![];
    match data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
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
        },
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
    let send_idents2 = &send_idents;
    let state_fields1 = &state_fields;
    let state_idents1 = &state_idents;

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

    let run_func = if pass_by_ref {
        quote! {
            let res = self.run(#(&#recv_idents3),*)?;
        }
    } else {
        quote! {
            let res = self.run(#(#recv_idents3),*)?;
        }
    };

    let derive_node = if aggregate {
        quote! {
            impl #impl_generics Node for #name #ty_generics #where_clause {
                fn start(&mut self) {
                    #(
                        for (send, val) in &self.#send_idents2 {
                            match val {
                                Some(v) => send.send(v.clone()),
                                None => continue,
                            }
                        }
                    )*
                    loop {
                        if self.call().is_err() {
                            break;
                        }
                    }
                }

                fn call(&mut self) -> Result<(), NodeError> {
                    #(
                        let #recv_idents1 = match self.#recv_idents2 {
                            Some(ref r) => r.recv().unwrap(),
                            None => return Err(NodeError::PermanentError),
                        };
                    )*
                    #run_func
                    if let Some(res) = res {
                        #(
                            for (send, _) in &self.#send_idents1 {
                                send.send(res.clone());
                            }
                        )*
                    }
                    Ok(())
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics Node for #name #ty_generics #where_clause {
                fn start(&mut self) {
                    #(
                        for (send, val) in &self.#send_idents2 {
                            match val {
                                Some(v) => send.send(v.clone()),
                                None => continue,
                            }
                        }
                    )*
                    loop {
                        if self.call().is_err() {
                            break;
                        }
                    }
                }

                fn call(&mut self) -> Result<(), NodeError> {
                    #(
                        let #recv_idents1 = match self.#recv_idents2 {
                            Some(ref r) => r.recv().unwrap(),
                            None => return Err(NodeError::PermanentError),
                        };
                    )*
                    #run_func
                    #(
                        for (send, _) in &self.#send_idents1 {
                            send.send(res.clone());
                        }
                    )*
                    Ok(())
                }
            }
        }
    };

    let macro_out = quote! {
        #new_impl
        #derive_node
    };
    macro_out.into()
}

fn parse_type(field: &syn::Field) -> FieldType {
    let ty = &field.ty;
    let type_str = quote! {#ty}.to_string();
    if type_str.starts_with("NodeReceiver") {
        FieldType::Input
    } else if type_str.starts_with("NodeSender") {
        FieldType::Output
    } else {
        FieldType::State
    }
}
