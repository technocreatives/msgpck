#![allow(clippy::match_overlapping_arm)]

extern crate proc_macro;

mod attribute;
mod pack;
mod unpack;

use pack::enums::derive_pack_enum;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use unpack::{enums::derive_unpack_enum, structs::derive_unpack_struct};

use crate::pack::structs::derive_pack_struct;

#[proc_macro_derive(MsgPack, attributes(msgpck_rs))]
pub fn derive_pack(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_pack_struct(&input, s),
        syn::Data::Enum(s) => derive_pack_enum(&input, s),
        syn::Data::Union(_) => Ok(quote! {
            compile_error!("derive(MsgPack) is not supported for unions");
        }),
    }
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[proc_macro_derive(MsgUnpack, attributes(msgpck_rs))]
pub fn derive_unpack(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_unpack_struct(&input, s),
        syn::Data::Enum(s) => derive_unpack_enum(&input, s),
        syn::Data::Union(_) => Ok(quote! {
            compile_error!("derive(MsgUnpack) is not supported for unions");
        }),
    }
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeriveKind {
    MsgPack,
    MsgUnpack,
}

fn array_len_iter(len: usize) -> TokenStream {
    match len {
        ..=0xf => {
            let len = len as u8;
            quote! { once(Marker::FixArray(#len).into()) }
        }
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
