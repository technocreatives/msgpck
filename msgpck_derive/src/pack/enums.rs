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

    let generics = &input.generics;

    let mut iter_enum_generics = quote! {};
    let mut iter_enum_variants = quote! {};
    let mut iter_enum_bounds = quote! {};
    let mut iter_enum_match = quote! {};
    let mut pack_variants = quote! {};
    let mut writer_pack_variants = quote! {};
    let mut writer_pack_variant_headers = quote! {};

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
            .append_all(quote! {#variant_name: Iterator<Item = ::msgpck::Piece<'a>>,});
        iter_enum_match.append_all(quote! {
            Self::#variant_name(inner_iter) => inner_iter.next(),
        });

        // generate the actual iterator

        let PackFields {
            pack_fields,
            write_pack_fields,
            match_fields,
            unit,
        } = pack_fields(&variant.fields, AttrLocation::EnumVariantField)?;

        let pack = if untagged && unit {
            // untagged variants with no fields are serialized as null
            quote! { ::core::iter::once(::msgpck::Marker::Null.into()) }
        } else if untagged {
            quote! { ::core::iter::empty() #pack_fields }
        } else if unit {
            quote! {
                ::msgpck::helpers::pack_enum_header(::msgpck::EnumHeader {
                    variant: #variant_name_str.into(),
                    unit: #unit,
                })
            }
        } else {
            quote! {
                ::msgpck::helpers::pack_enum_header(::msgpck::EnumHeader {
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
                __msgpck_n += 1;
                __msgpck_w.write_all(&[::msgpck::Marker::Null.to_u8()])?;
            }
        } else if untagged {
            write_pack_fields
        } else {
            writer_pack_variant_headers.append_all(quote! {
                Self::#variant_name #match_fields =>::msgpck::EnumHeader {
                    variant: #variant_name_str.into(),
                    unit: #unit,
                },
            });

            if unit {
                quote! {}
            } else {
                write_pack_fields
            }
        };

        writer_pack_variants.append_all(quote! {
            Self::#variant_name #match_fields => {
                #write_pack
            }
        });
    }

    if !untagged {
        writer_pack_variant_headers = quote! {
            // create and serialize enum header
            let header = match self {
                #writer_pack_variant_headers
            };
            __msgpck_n += ::msgpck::helpers::pack_enum_header_to_writer(header, __msgpck_w)?;
        }
    } else {
        writer_pack_variant_headers = quote! {
            // enum is marked as untagged, so we don't serialize the enum header
        }
    }

    Ok(quote! {
        #[automatically_derived]
        impl #generics ::msgpck::MsgPack for #enum_name #generics {
            fn pack(&self) -> impl Iterator<Item = ::msgpck::Piece<'_>> {
                // Because we need different msgpack iterator types for each variant, we need a new
                // enum type that impls Iterator to contain them. To avoid naming the inner iterator
                // types, we use generics. It's not the prettiest, but it works.
                enum __MsgpackerIter<#iter_enum_generics> {
                    #iter_enum_variants
                }

                impl<'a, #iter_enum_bounds> Iterator for __MsgpackerIter<#iter_enum_generics> {
                    type Item = ::msgpck::Piece<'a>;

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

            fn pack_with_writer(&self, __msgpck_w: &mut dyn ::msgpck::Write)
                -> Result<usize, ::msgpck::PackErr>
            {
                let mut __msgpck_n = 0usize;
                #writer_pack_variant_headers

                // serialize variant fields
                match self {
                    #writer_pack_variants
                }

                Ok(__msgpck_n)
            }
        }
    })
}
