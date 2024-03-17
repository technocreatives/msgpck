use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, Fields};

use crate::{
    array_len_write,
    attribute::{parse_attributes, AttrLocation, Attribute},
    DeriveKind, RESERVED_NAMES,
};

pub mod enums;
pub mod structs;

pub struct PackFields {
    /// ```ignore
    ///     .chain(field1.pack())
    ///     .chain(field2.pack())
    ///     .chain(field3.pack())
    /// ```
    pub pack_fields: TokenStream,

    /// ```ignore
    /// __msgpck_n += field1,pack_with_writer(__msgpck_w)?;
    /// __msgpck_n += field2,pack_with_writer(__msgpck_w)?;
    /// __msgpck_n += field3,pack_with_writer(__msgpck_w)?;
    /// ```
    pub write_pack_fields: TokenStream,

    /// ```ignore
    /// // either
    ///     { field1, field2, field3 }
    /// // or
    ///     ( field1, field2, field3 )
    /// // or nothing, in the case of an item without fields, brackets, and parentheses
    /// ```
    pub match_fields: TokenStream,

    /// `true` the item contains no fields.
    pub unit: bool,
}

/// Pack a set of fields, i.e. a struct or the fields of an enum variant.
pub fn pack_fields(fields: &Fields, location: AttrLocation) -> syn::Result<PackFields> {
    let unit;
    let mut pack_fields = quote! {};
    let mut write_pack_fields = quote! {};
    let mut match_fields = quote! {};

    match fields {
        Fields::Named(fields) => {
            let mut fields_len = 0usize;
            for field in fields.named.iter() {
                let attributes = parse_attributes(&field.attrs, location, DeriveKind::MsgPack)?;
                if !attributes.contains(&Attribute::Skip) {
                    fields_len += 1;
                }
            }

            pack_fields.append_all(quote! {
                .chain(::msgpck::helpers::pack_array_header(#fields_len))
            });
            write_pack_fields.append_all(array_len_write(fields_len));

            unit = fields_len == 0;

            for field in &fields.named {
                let field_attributes =
                    parse_attributes(&field.attrs, location, DeriveKind::MsgPack)?;
                if field_attributes.contains(&Attribute::Skip) {
                    continue;
                }

                let field_name = field.ident.as_ref().expect("fields are named");

                if RESERVED_NAMES.contains(&field_name.to_string().as_str()) {
                    return Err(syn::Error::new(
                        field.ident.span(),
                        "MsgPack: reserved identifier",
                    ));
                }

                // pattern match all the fields
                match_fields.append_all(quote! {#field_name, });

                // pack all the named fields
                pack_fields.append_all(quote! {
                    .chain(::msgpck::MsgPack::pack(#field_name))
                });

                write_pack_fields.append_all(quote! {
                    __msgpck_n += ::msgpck::MsgPack::pack_with_writer(#field_name, __msgpck_w)?;
                });
            }

            // wrap fields pattern in brackets
            match_fields = quote! { { #match_fields .. } };
        }
        syn::Fields::Unnamed(fields) => {
            // if there is more than one field, pack them as an array
            let mut fields_len = 0usize;
            for field in fields.unnamed.iter() {
                let attributes = parse_attributes(&field.attrs, location, DeriveKind::MsgPack)?;
                if !attributes.contains(&Attribute::Skip) {
                    fields_len += 1;
                }
            }

            if fields_len != 1 {
                pack_fields.append_all(quote! {
                    .chain(::msgpck::helpers::pack_array_header(#fields_len))
                });
                write_pack_fields.append_all(array_len_write(fields_len));
            }

            unit = fields_len == 0;

            for (i, field) in fields.unnamed.iter().enumerate() {
                let field_attributes =
                    parse_attributes(&field.attrs, location, DeriveKind::MsgPack)?;
                if field_attributes.contains(&Attribute::Skip) {
                    continue;
                }

                let field_name = Ident::new(&format!("_{i}"), field.span());
                // pattern match all the fields
                match_fields.append_all(quote! {#field_name, });

                // pack all the fields
                pack_fields.append_all(quote! {
                    .chain(#field_name.pack())
                });

                write_pack_fields.append_all(quote! {
                    __msgpck_n += ::msgpck::MsgPack::pack_with_writer(#field_name, __msgpck_w)?;
                });
            }

            // wrap fields pattern in parentheses
            match_fields = quote! { (#match_fields ..) };
        }
        syn::Fields::Unit => {
            pack_fields.append_all(quote! {
                .chain(::msgpck::helpers::pack_array_header(0))
            });
            write_pack_fields.append_all(array_len_write(0));
            unit = true;
        }
    }

    Ok(PackFields {
        pack_fields,
        write_pack_fields,
        match_fields,
        unit,
    })
}
