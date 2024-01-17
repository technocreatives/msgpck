use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, DataStruct, DeriveInput, Fields, GenericParam};

use crate::{
    array_len_iter, array_len_write,
    attribute::{parse_attributes, AttrLocation},
    DeriveKind,
};

use super::{pack_fields, PackFields};

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

    let PackFields {
        pack_fields,
        write_pack_fields,
        match_fields,
        ..
    } = pack_fields(&data.fields)?;

    let mut pack_body = quote! {
        let #struct_name #match_fields = self;
    };

    let mut writer_pack_body = quote! {
        let #struct_name #match_fields = self;
        let mut __msgpck_rs_n = 0usize;
    };

    // serialize newtype structs without using array, this is to maintain compatibility with serde
    match &data.fields {
        Fields::Unnamed(..) if struct_len == 1 => {
            pack_body.append_all(quote! { ::core::iter::empty() #pack_fields });
        }
        _ => {
            let array_header_iter = array_len_iter(data.fields.len());
            pack_body.append_all(quote! { #array_header_iter #pack_fields });

            writer_pack_body.append_all(array_len_write(data.fields.len()));
        }
    };
    writer_pack_body.append_all(write_pack_fields);

    Ok(quote! {
        impl<#impl_generics> msgpck_rs::MsgPack for #struct_name<#struct_generics>
        where #generic_bounds {
            fn pack<'_msgpack>(&'_msgpack self) -> impl Iterator<Item = ::msgpck_rs::Piece<'_msgpack>> {
                #pack_body
            }

            fn pack_with_writer(&self, __msgpck_rs_w: &mut dyn ::msgpck_rs::Write)
                -> ::core::result::Result<usize, ::msgpck_rs::BufferOverflow>
            {
                #writer_pack_body
                Ok(__msgpck_rs_n)
            }
        }
    })
}
