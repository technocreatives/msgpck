extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, DeriveInput, DataStruct, spanned::Spanned};

#[proc_macro_derive(MsgPack)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => {
            derive_struct(&input, &s)
        }
        syn::Data::Enum(_) => unimplemented!("TODO: implement derive for enums"),
        syn::Data::Union(_) => panic!("no union pls"),
    }
}
 
fn derive_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let mut encode_body = array_len_iter(data.fields.len());

    for (i, field) in data.fields.iter().enumerate() {
        let ident = field.ident.clone().unwrap_or_else(|| Ident::new(&format!("{i}"), field.span()));
        encode_body.append_all(quote! {
            .chain(self.#ident.encode())
        });
    }

    quote! {
        impl msgpackers::MsgPack for #struct_name {
            type Iter<'a> = impl Iterator<Item = msgpackers::Piece<'a>>
            where
                Self: 'a;

            fn encode<'a>(&'a self) -> Self::Iter<'a> {
                use ::std::iter::once;
                use ::msgpackers::Marker;
                #encode_body
            }
        }
    }.into()
}

fn array_len_iter(len: usize) -> proc_macro2::TokenStream {
    match len {
        ..=0xf => {
            let len = len as u8;
            quote! { once(Marker::FixArray(#len).into()) }
        },
        ..=0xffff => {
            let len = len as u16;
            quote! { 
                [
                    Marker::Array16.into(),
                    #len.into(),
                ].into_iter()
            }
        }
        _ => {
            let len = len as u32;
            quote! { 
                [
                    Marker::Array32.into(),
                    #len.into(),
                ].into_iter()
            }
        }
    }
}
