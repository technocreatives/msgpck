#![allow(clippy::match_overlapping_arm)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned, DataEnum, DataStruct, DeriveInput};

#[proc_macro_derive(MsgUnpack)]
pub fn derive_unpack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_unpack_struct(&input, s),
        syn::Data::Enum(_) => unimplemented!("TODO: implement derive for enums"),
        syn::Data::Union(_) => panic!("no union pls"),
    }
}

#[proc_macro_derive(MsgPack)]
pub fn derive_pack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_pack_struct(&input, s),
        syn::Data::Enum(_) => unimplemented!("TODO: implement derive for enums"),
        syn::Data::Union(_) => panic!("no union pls"),
    }
}

fn derive_pack_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let mut encode_body = array_len_iter(data.fields.len());

    for (i, field) in data.fields.iter().enumerate() {
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("{i}"), field.span()));
        encode_body.append_all(quote! {
            .chain(self.#ident.pack())
        });
    }

    quote! {
        impl msgpackers::MsgPack for #struct_name {
            type Iter<'a> = impl Iterator<Item = msgpackers::Piece<'a>>
            where
                Self: 'a;

            fn pack<'a>(&'a self) -> Self::Iter<'a> {
                use ::std::iter::once;
                use ::msgpackers::Marker;
                #encode_body
            }
        }
    }
    .into()
}

fn derive_unpack_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let struct_len = data.fields.len();
    let mut unpack_body = quote! {};

    for (i, field) in data.fields.iter().enumerate() {
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("{i}"), field.span()));
        unpack_body.append_all(quote! {
            #ident: MsgUnpack::unpack(bytes)?,
        });
    }

    quote! {
        impl<'buf> ::msgpackers::MsgUnpack<'buf> for #struct_name {
            fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, ::msgpackers::UnpackErr>
            where
                Self: Sized,
            {
                use ::msgpackers::{MsgUnpack, UnpackErr, unpack_array_header};
                let n = unpack_array_header(bytes)?;

                if n < #struct_len {
                    return Err(UnpackErr::UnexpectedEof);
                }

                let value = Self {
                    #unpack_body
                };

                if n > #struct_len {
                    return Err(UnpackErr::UnexpectedEof);
                }

                // TODO: be lenient with parsing
                //for _ in #struct_len..=n {
                //    let _ = MsgUnpack::unpack(bytes)?;
                //}

                Ok(value)
            }
        }

    }
    .into()
}

fn derive_pack_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;
    let variant_count = &data.variants.len();

    let mut unpack_body = quote! {};

    //todo!("derive pack for enums")

    quote! {
        impl ::msgpackers::MsgUnpack for #enum_name {
            fn unpack<'buf>(bytes: &mut &'buf [u8]) -> Result<Self, ::msgpackers::UnpackErr>
            where
                Self: Sized + 'buf,
            {

            }
        }
    }
    .into()
}

fn derive_unpack_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    todo!("derive unpack for enums")
}

fn array_len_iter(len: usize) -> proc_macro2::TokenStream {
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
