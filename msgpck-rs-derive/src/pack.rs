use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, Fields};

use crate::{array_len_write, RESERVED_NAMES};

pub mod enums;
pub mod structs;

pub struct PackFields {
    /// ```
    ///     .chain(field1.pack())
    ///     .chain(field2.pack())
    ///     .chain(field3.pack())
    /// ```
    pub pack_fields: TokenStream,

    /// ```
    /// __msgpck_rs_n += field1,pack_with_writer(__msgpck_rs_w)?;
    /// __msgpck_rs_n += field2,pack_with_writer(__msgpck_rs_w)?;
    /// __msgpck_rs_n += field3,pack_with_writer(__msgpck_rs_w)?;
    /// ```
    pub write_pack_fields: TokenStream,

    /// ```
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
pub fn pack_fields(fields: &Fields) -> syn::Result<PackFields> {
    let unit;
    let mut pack_fields = quote! {};
    let mut write_pack_fields = quote! {};
    let mut match_fields = quote! {};

    match fields {
        Fields::Named(fields) => {
            // if there is more than one field, pack them as an array
            let fields_len = fields.named.len();
            //if fields_len != 1 {
            pack_fields.append_all(quote! {
                .chain(::msgpck_rs::helpers::pack_array_header(#fields_len))
            });
            write_pack_fields.append_all(array_len_write(fields_len));
            //}

            unit = fields_len == 0;

            for field in &fields.named {
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
                    .chain(::msgpck_rs::MsgPack::pack(#field_name))
                });

                write_pack_fields.append_all(quote! {
                    __msgpck_rs_n += ::msgpck_rs::MsgPack::pack_with_writer(#field_name, __msgpck_rs_w)?;
                });
            }

            // wrap fields pattern in brackets
            match_fields = quote! { { #match_fields } };
        }
        syn::Fields::Unnamed(fields) => {
            // if there is more than one field, pack them as an array
            let fields_len = fields.unnamed.len();
            if fields_len != 1 {
                pack_fields.append_all(quote! {
                    .chain(::msgpck_rs::helpers::pack_array_header(#fields_len))
                });
                write_pack_fields.append_all(array_len_write(fields_len));
            }

            unit = fields_len == 0;

            for (i, field) in fields.unnamed.iter().enumerate() {
                let field_name = Ident::new(&format!("_{i}"), field.span());
                // pattern match all the fields
                match_fields.append_all(quote! {#field_name, });

                // pack all the fields
                pack_fields.append_all(quote! {
                    .chain(#field_name.pack())
                });

                write_pack_fields.append_all(quote! {
                    __msgpck_rs_n += ::msgpck_rs::MsgPack::pack_with_writer(#field_name, __msgpck_rs_w)?;
                });
            }

            // wrap fields pattern in parentheses
            match_fields = quote! { (#match_fields) };
        }
        syn::Fields::Unit => {
            pack_fields.append_all(quote! {
                .chain(::msgpck_rs::helpers::pack_array_header(0))
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
