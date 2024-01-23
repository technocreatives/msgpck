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

/// Identifiers used by the derive macro that could conflict with user defined ones.
const RESERVED_NAMES: &[&str] = &["__msgpck_rs_w", "__MsgpackerIter"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeriveKind {
    MsgPack,
    MsgUnpack,
}

/// Generate code that packs an array marker to a writer for the given length.
fn array_len_write(len: usize) -> TokenStream {
    let marker_t = quote! { ::msgpck_rs::Marker };

    let len: u32 = len.try_into().expect("array length doesn't fit in u32");

    match len {
        ..=0xf => {
            let len = len as u8;
            quote! {
                __msgpck_rs_n += 1;
                __msgpck_rs_w.write_all(&[#marker_t::FixArray(#len).to_u8()])?;
            }
        }
        ..=0xffff => {
            let len = len as u16;
            quote! {
                __msgpck_rs_n += 3;
                __msgpck_rs_w.write_all(&[#marker_t::Array16.to_u8()])?;
                __msgpck_rs_w.write_all(::msgpck_rs::Piece::from(#len).as_bytes())?;
            }
        }
        _ => {
            quote! {
                __msgpck_rs_n += 5;
                __msgpck_rs_w.write_all(&[#marker_t::Array32.to_u8()])?;
                __msgpck_rs_w.write_all(::msgpck_rs::Piece::from(#len).as_bytes())?;
            }
        }
    }
}
