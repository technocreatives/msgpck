use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, DataEnum, DeriveInput};

use crate::{
    attribute::{parse_attributes, AttrLocation, Attribute},
    DeriveKind, RESERVED_NAMES,
};

use super::{pack_fields, PackFields};

/// Generate impl MsgPack for an enum
pub fn derive_pack_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let attributes = parse_attributes(&input.attrs, AttrLocation::Enum, DeriveKind::MsgPack)?;
    let untagged = attributes.contains(&Attribute::Untagged);

    if RESERVED_NAMES.contains(&enum_name.to_string().as_str()) {
        return Err(syn::Error::new(
            input.ident.span(),
            "MsgPack: reserved identifier",
        ));
    }

    // TODO: where-clause for enums
    if let Some(where_clause) = &input.generics.where_clause {
        return Err(syn::Error::new(
            where_clause.span(),
            "derive(MsgPack) doesn't support where-clauses for enums yet",
        ));
    }

    // TODO: generics for enums
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.params.span(),
            "derive(MsgPack) doesn't support generics for enums yet",
        ));
    }

    let mut iter_enum_generics = quote! {};
    let mut iter_enum_variants = quote! {};
    let mut iter_enum_bounds = quote! {};
    let mut iter_enum_match = quote! {};
    let mut pack_variants = quote! {};
    let mut writer_pack_variants = quote! {};

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        // TODO: for if/when we wan't to support serializing by discriminant, instead of name
        //if let Some((_, explicit_discriminant)) = &variant.discriminant {
        //    // TODO: handle explicitly setting the discriminant.
        //    // This loop should track the discriminant in the same fashion as rustc
        //    match explicit_discriminant {
        //        syn::Expr::Lit(_lit) => {
        //            return quote_spanned! {
        //                explicit_discriminant.span() =>
        //                compile_error!("not supported (yet) by derive(MsgUnpack)");
        //            };
        //        }
        //        _ => {
        //            return quote_spanned! {
        //                explicit_discriminant.span() =>
        //                compile_error!("not supported by derive(MsgUnpack)");
        //            };
        //        }
        //    }
        //}

        // generate stuff for the iterator enum
        iter_enum_generics.append_all(quote! {#variant_name,});
        iter_enum_variants.append_all(quote! {#variant_name(#variant_name),});
        iter_enum_bounds
            .append_all(quote! {#variant_name: Iterator<Item = ::msgpck_rs::Piece<'a>>,});
        iter_enum_match.append_all(quote! {
            Self::#variant_name(inner_iter) => inner_iter.next(),
        });

        // generate the actual iterator

        let PackFields {
            pack_fields,
            write_pack_fields,
            match_fields,
            unit,
        } = pack_fields(&variant.fields)?;

        let pack = if untagged && unit {
            // untagged variants with no fields are serialized as null
            quote! { ::core::iter::once(::msgpck_rs::Marker::Null.into()) }
        } else if untagged {
            quote! { ::core::iter::empty() #pack_fields }
        } else {
            quote! {
                ::msgpck_rs::helpers::pack_enum_header(::msgpck_rs::EnumHeader {
                    variant: #variant_name_str.into(),
                    unit: #unit,
                })
                #pack_fields
            }
        };
        pack_variants.append_all(quote! {
            Self::#variant_name #match_fields => {
                __MsgpackerIter::#variant_name(#pack)
            }
        });

        let write_pack = if untagged && unit {
            // untagged variants with no fields are serialized as null
            quote! {
                __msgpck_rs_w.write_all(&[::msgpck_rs::Marker::Null.to_u8()])?;
                __msgpck_rs_n += 1;
            }
        } else if untagged {
            write_pack_fields
        } else {
            quote! {
                for piece in ::msgpck_rs::helpers::pack_enum_header(::msgpck_rs::EnumHeader {
                    variant: #variant_name_str.into(),
                    unit: #unit,
                }) {
                    __msgpck_rs_w.write_all(piece.as_bytes())?;
                    __msgpck_rs_n += piece.as_bytes().len();
                }
                #write_pack_fields
            }
        };
        writer_pack_variants.append_all(quote! {
            Self::#variant_name #match_fields => {
                #write_pack
            }
        });
    }

    Ok(quote! {
        impl ::msgpck_rs::MsgPack for #enum_name {
            fn pack(&self) -> impl Iterator<Item = ::msgpck_rs::Piece<'_>> {
                // Because we need different msgpack iterator types for each variant, we need a new
                // enum type that impls Iterator to contain them. To avoid naming the inner iterator
                // types, we use generics. It's not the prettiest, but it works.
                enum __MsgpackerIter<#iter_enum_generics> {
                    #iter_enum_variants
                }

                impl<'a, #iter_enum_bounds> Iterator for __MsgpackerIter<#iter_enum_generics> {
                    type Item = ::msgpck_rs::Piece<'a>;

                    // the implementation of next simply forwards to the inner iterators
                    fn next(&mut self) -> Option<Self::Item> {
                        match self {
                            #iter_enum_match
                        }
                    }
                }

                // This match statment returns a __MsgpackerIter
                match self {
                    #pack_variants
                }
            }

            fn pack_with_writer(&self, __msgpck_rs_w: &mut dyn ::msgpck_rs::Write)
                -> Result<usize, ::msgpck_rs::BufferOverflow>
            {
                let mut __msgpck_rs_n = 0usize;
                match self {
                    #writer_pack_variants
                }

                Ok(__msgpck_rs_n)
            }
        }
    })
}
