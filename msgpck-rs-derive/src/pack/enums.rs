use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, DataEnum, DeriveInput};

use crate::{
    attribute::{parse_attributes, AttrLocation, Attribute},
    DeriveKind,
};

/// Generate impl MsgPack for an enum
pub fn derive_pack_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let attributes = parse_attributes(&input.attrs, AttrLocation::Enum, DeriveKind::MsgPack)?;
    let untagged = attributes.contains(&Attribute::Untagged);

    // TODO: where-clause for enums
    if let Some(where_clause) = &input.generics.where_clause {
        return Err(syn::Error::new(
            where_clause.span(),
            "derive(MsgPack) doesn't support where-clauses for enums",
        ));
    }

    // TODO: generics for enums
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.params.span(),
            "derive(MsgPack) doesn't support generics for enums",
        ));
    }

    let mut iter_enum_generics = quote! {};
    let mut iter_enum_variants = quote! {};
    let mut iter_enum_bounds = quote! {};
    let mut iter_enum_match = quote! {};
    let mut pack_variants = quote! {};

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

        let unit: bool;

        // anything that should be packed after the header goes in here
        let mut pack_fields = quote! {};

        // the pattern match for the enum variant fields
        let mut match_fields = quote! {};

        match &variant.fields {
            syn::Fields::Named(fields) => {
                // if there is more than one field, pack them as an array
                let fields_len = fields.named.len();
                if fields_len > 1 {
                    pack_fields.append_all(quote! {
                        .chain(::msgpck_rs::helpers::pack_array_header(#fields_len))
                    });
                }

                unit = fields_len == 0;

                for field in &fields.named {
                    let field_name = &field.ident;
                    // pattern match all the fields
                    match_fields.append_all(quote! {#field_name, });

                    // pack all the named fields
                    pack_fields.append_all(quote! {
                        .chain(::msgpck_rs::MsgPack::pack(#field_name))
                    });
                }

                // wrap fields pattern in brackets
                match_fields = quote! { { #match_fields } };
            }
            syn::Fields::Unnamed(fields) => {
                // if there is more than one field, pack them as an array
                let fields_len = fields.unnamed.len();
                if fields_len > 1 {
                    pack_fields.append_all(quote! {
                        .chain(::msgpck_rs::helpers::pack_array_header(#fields_len))
                    });
                }

                unit = fields_len == 0;

                for (i, field) in fields.unnamed.iter().enumerate() {
                    let field_name = Ident::new(&format!("_{i}"), field.span());
                    // pattern match all the fields
                    match_fields.append_all(quote! {#field_name, });

                    // pack all the fields
                    pack_fields.append_all(quote! {
                        .chain(#field_name.pack())
                    })
                }

                // wrap fields pattern in parentheses
                match_fields = quote! { (#match_fields) };
            }
            syn::Fields::Unit => unit = true,
        }

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
    }

    Ok(quote! {
        impl ::msgpck_rs::MsgPack for #enum_name {
            fn pack(&self) -> impl Iterator<Item = ::msgpck_rs::Piece<'_>> {
                // Because we need different msgpack iterator types for each variant, we need an
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
        }
    })
}
