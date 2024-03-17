use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, DataStruct, DeriveInput, GenericParam};

use crate::{
    attribute::{parse_attributes, AttrLocation},
    DeriveKind,
};

use super::{pack_fields, PackFields};

/// Generate impl MsgPack for a struct
pub fn derive_pack_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
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
                    #t: ::msgpck::MsgPack,
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
    } = pack_fields(&data.fields, AttrLocation::StructField)?;

    let pack_body = quote! {
        let #struct_name #match_fields = self;
        ::core::iter::empty() #pack_fields
    };

    let writer_pack_body = quote! {
        let #struct_name #match_fields = self;
        let mut __msgpck_n = 0usize;
        #write_pack_fields
    };

    Ok(quote! {
        #[automatically_derived]
        impl<#impl_generics> msgpck::MsgPack for #struct_name<#struct_generics>
        where #generic_bounds {
            fn pack<'_msgpack>(&'_msgpack self) -> impl Iterator<Item = ::msgpck::Piece<'_msgpack>> {
                #pack_body
            }

            fn pack_with_writer(&self, __msgpck_w: &mut dyn ::msgpck::Write)
                -> ::core::result::Result<usize, ::msgpck::PackErr>
            {
                #writer_pack_body
                Ok(__msgpck_n)
            }
        }
    })
}
