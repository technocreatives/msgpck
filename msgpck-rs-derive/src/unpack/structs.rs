use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, DataStruct, DeriveInput, Fields, GenericParam};

use crate::{
    attribute::{parse_attributes, AttrLocation, Attribute},
    DeriveKind,
};

/// Generate impl MsgUnpack for a struct
pub fn derive_unpack_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let _attributes = parse_attributes(&input.attrs, AttrLocation::Struct, DeriveKind::MsgUnpack)?;

    let mut struct_len = 0usize;
    for field in data.fields.iter() {
        let field_attributes = parse_attributes(
            &field.attrs,
            AttrLocation::StructField,
            DeriveKind::MsgUnpack,
        )?;

        if !field_attributes.contains(&Attribute::Skip) {
            struct_len += 1;
        }
    }

    // TODO: where-clause for structs
    if let Some(where_clause) = &input.generics.where_clause {
        return Err(syn::Error::new(
            where_clause.span(),
            "derive(MsgUnpack) doesn't support where-clauses for structs",
        ));
    }

    let mut unpack_fields = quote! {};
    let mut generic_bounds = quote! {};
    let impl_generics = &input.generics.params;
    let mut struct_generics = quote! {};

    for param in impl_generics {
        match param {
            GenericParam::Lifetime(l) => {
                let l = &l.lifetime;
                struct_generics.append_all(quote! { #l, });
                generic_bounds.append_all(quote! {
                    '_msgpack: #l,
                });
            }
            GenericParam::Type(t) => {
                let t = &t.ident;
                struct_generics.append_all(quote! { #t, });
                generic_bounds.append_all(quote! {
                    #t: ::msgpck_rs::MsgUnpack<'_msgpack>,
                });
            }
            GenericParam::Const(..) => continue,
        }
    }

    for field in data.fields.iter() {
        let field_attributes = parse_attributes(
            &field.attrs,
            AttrLocation::StructField,
            DeriveKind::MsgUnpack,
        )?;

        if field_attributes.contains(&Attribute::Default) {
            return Err(syn::Error::new(
                field.span(),
                "msgpck_rs(default) is not yet implemented for struct fields",
            ));
        }

        if field_attributes.contains(&Attribute::Skip) {
            unpack_fields.append_all(match &field.ident {
                Some(ident) => quote! { #ident: ::core::default::Default::default(), },
                None => quote! { ::core::default::Default::default(), },
            });
        } else {
            unpack_fields.append_all(match &field.ident {
                Some(ident) => quote! { #ident: MsgUnpack::unpack(bytes)?, },
                None => quote! { MsgUnpack::unpack(bytes)?, },
            });
        }
    }

    // wrap the fields in the appropriate brackets, if any
    unpack_fields = match &data.fields {
        Fields::Named(_) => quote! { {#unpack_fields} },
        Fields::Unnamed(_) => quote! { (#unpack_fields) },
        Fields::Unit => quote! {},
    };

    // newtype structs are serialized without using an array, this is to maintain compatibility with serde
    let unpack_body = if matches!(&data.fields, Fields::Unnamed(..)) && struct_len == 1 {
        quote! {
            let value = Self #unpack_fields;
            Ok(value)
        }
    } else {
        quote! {
            let n = unpack_array_header(bytes)?;

            if n < #struct_len {
                return Err(UnpackErr::MissingFields {
                    got: n,
                    expected: #struct_len
                });
            }
            if n > #struct_len {
                return Err(UnpackErr::TooManyFields {
                    got: n,
                    expected: #struct_len
                });
            }

            let value = Self #unpack_fields;

            Ok(value)
        }
    };

    Ok(quote! {
        #[automatically_derived]
        impl<'_msgpack, #impl_generics> ::msgpck_rs::MsgUnpack<'_msgpack> for #struct_name<#struct_generics>
        where #generic_bounds {
            fn unpack(bytes: &mut &'_msgpack [u8]) -> Result<Self, ::msgpck_rs::UnpackErr>
            where
                Self: Sized,
            {
                use ::msgpck_rs::{MsgUnpack, UnpackErr, helpers::unpack_array_header};

                #unpack_body
            }
        }
    })
}
