use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, DataStruct, DeriveInput, Fields, GenericParam, Index, Member};

use crate::{
    array_len_iter,
    attribute::{parse_attributes, AttrLocation},
    DeriveKind,
};

/// Generate impl MsgPack for a struct
pub fn derive_pack_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let struct_len = data.fields.len();
    let _attributes = parse_attributes(&input.attrs, AttrLocation::Struct, DeriveKind::MsgPack)?;

    // TODO: where-clause for structs
    if let Some(where_clause) = &input.generics.where_clause {
        return Err(syn::Error::new(
            where_clause.span(),
            "derive(MsgUnpack) doesn't support where-clauses for structs",
        ));
    }

    let mut encode_body = quote! {};
    let mut generic_bounds = quote! {};
    let impl_generics = &input.generics.params;
    let mut struct_generics = quote!();

    for param in impl_generics {
        match param {
            GenericParam::Lifetime(l) => {
                let l = &l.lifetime;
                struct_generics.append_all(quote! { #l, });
            }
            GenericParam::Type(t) => {
                let t = &t.ident;
                struct_generics.append_all(quote! { #t, });
                generic_bounds.append_all(quote! {
                    #t: ::msgpck_rs::MsgPack,
                });
            }
            GenericParam::Const(..) => continue,
        }
    }

    // serialize newtype structs without using array, this is to maintain compatibility with serde
    encode_body.append_all(match &data.fields {
        Fields::Unnamed(..) if struct_len == 1 => {
            quote! { ::core::iter::empty() }
        }
        _ => array_len_iter(data.fields.len()),
    });

    for (i, field) in data.fields.iter().enumerate() {
        let member = field.ident.clone().map(Member::Named).unwrap_or_else(|| {
            Member::Unnamed(Index {
                index: i as u32,
                span: field.span(),
            })
        });
        encode_body.append_all(quote! {
            .chain(self.#member.pack())
        });
    }

    Ok(quote! {
        impl<#impl_generics> msgpck_rs::MsgPack for #struct_name<#struct_generics>
        where #generic_bounds {
            fn pack<'_msgpack>(&'_msgpack self) -> impl Iterator<Item = ::msgpck_rs::Piece<'_msgpack>> {
                use ::core::iter::once;
                use ::msgpck_rs::Marker;
                #encode_body
            }
        }
    })
}
