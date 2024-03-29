use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{spanned::Spanned, DataEnum, DeriveInput, Expr, ExprUnary, Fields, Lit, UnOp};

use crate::{
    attribute::{parse_attributes, AttrLocation, Attribute},
    DeriveKind,
};

/// Generate impl MsgUnpack for an enum
pub fn derive_unpack_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let _attributes = parse_attributes(&input.attrs, AttrLocation::Enum, DeriveKind::MsgUnpack)?;

    let generics = &input.generics;
    let generics: TokenStream = generics
        .type_params()
        .map(|t| quote! {#t,})
        .chain(generics.const_params().map(|c| quote! {#c,}))
        .collect();
    let lifetimes: TokenStream = input
        .generics
        .lifetimes()
        .map(|_l| quote! { '_msgpck, })
        .collect();
    let mut impl_generics = quote! { '_msgpck, #generics };

    for l in input.generics.lifetimes() {
        impl_generics.append_all(quote! { #l, 'msgpck: #l, });
    }

    let ty_generics = quote! { <#lifetimes #generics> };

    let mut unpack_variants = quote! {};
    let mut other_variant = None;
    let mut discriminant = 0isize;

    for variant in &data.variants {
        if let Some((_, explicit_discriminant)) = &variant.discriminant {
            let not_supported_err = Err(syn::Error::new(
                explicit_discriminant.span(),
                "not supported by derive(MsgUnpack)",
            ));

            let (is_positive, lit) = match explicit_discriminant {
                Expr::Lit(lit) => (true, lit),
                Expr::Unary(ExprUnary {
                    op: UnOp::Neg(_),
                    expr,
                    ..
                }) => match &**expr {
                    Expr::Lit(lit) => (false, lit),
                    _ => return not_supported_err,
                },
                _ => return not_supported_err,
            };

            let Lit::Int(lit_int) = &lit.lit else {
                return not_supported_err;
            };

            let n = match lit_int.base10_parse() {
                Err(e) => {
                    let e = format!("failed to parse integer as isize: {e}");
                    return Err(syn::Error::new(lit.span(), e));
                }
                Ok(n) => n,
            };

            if is_positive {
                discriminant = n;
            } else {
                discriminant = -n;
            }
        }

        let variant_attributes = parse_attributes(
            &variant.attrs,
            AttrLocation::EnumVariant,
            DeriveKind::MsgUnpack,
        )?;

        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        if variant_attributes.contains(&Attribute::Other) {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(syn::Error::new(
                    variant.fields.span(),
                    "#[msgpck(other)] must be applied to a unit variant",
                ));
            }

            if other_variant.is_some() {
                return Err(syn::Error::new(
                    variant.span(),
                    "there can only be one variant marked #[msgpck(other)]",
                ));
            }

            other_variant = Some(variant_name);
            continue;
        }

        let match_pattern = quote! {
            Discriminant(#discriminant) | Name(#variant_name_str)
        };

        let validate_type = match variant.fields.len() {
            0 => quote! {
                if !header.unit {
                    return Err(UnpackErr::UnexpectedUnitVariant);
                }
            },
            1 => quote! {
                if header.unit {
                    return Err(UnpackErr::ExpectedUnitVariant);
                }
            },
            n => quote! {
                if header.unit {
                    return Err(UnpackErr::ExpectedUnitVariant);
                }

                let array_len = unpack_array_header(bytes)?;

                if array_len < #n {
                    return Err(UnpackErr::MissingFields {
                        expected: #n,
                        got: array_len,
                    });
                };

                if array_len > #n {
                    todo!("decide how to handle encodede enum variants with too many fields");
                };
            },
        };

        let mut construct_fields = quote! {};

        match &variant.fields {
            syn::Fields::Unit => {}
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    let field_attributes = parse_attributes(
                        &field.attrs,
                        AttrLocation::EnumVariantField,
                        DeriveKind::MsgUnpack,
                    )?;

                    if field_attributes.contains(&Attribute::Default) {
                        return Err(syn::Error::new(
                            field.span(),
                            "msgpck(default) is not yet implemented for enum variant fields",
                        ));
                    }

                    let field_name = field.ident.as_ref().unwrap();

                    construct_fields.append_all(if field_attributes.contains(&Attribute::Skip) {
                        quote! { #field_name: ::core::default::Default::default(), }
                    } else {
                        quote! { #field_name: MsgUnpack::unpack(bytes)?, }
                    })
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    let field_attributes = parse_attributes(
                        &field.attrs,
                        AttrLocation::EnumVariantField,
                        DeriveKind::MsgUnpack,
                    )?;

                    if field_attributes.contains(&Attribute::Default) {
                        return Err(syn::Error::new(
                            field.span(),
                            "msgpck(default) is not yet implemented for enum variant fields",
                        ));
                    }

                    if field_attributes.contains(&Attribute::Skip) {
                        continue;
                    }

                    construct_fields.append_all(if field_attributes.contains(&Attribute::Skip) {
                        quote! { ::core::default::Default::default(), }
                    } else {
                        quote! { MsgUnpack::unpack(bytes)?, }
                    })
                }
            }
        };

        let constructor = match &variant.fields {
            syn::Fields::Named(_) => quote! { Self::#variant_name { #construct_fields } },
            syn::Fields::Unnamed(_) => quote! { Self::#variant_name(#construct_fields) },
            syn::Fields::Unit => quote! { Self::#variant_name },
        };

        unpack_variants.append_all(quote! {
            #match_pattern => {
                #validate_type
                #constructor
            }
        });

        discriminant += 1;
    }

    let unknown_variant_match = match other_variant {
        Some(other) => quote! { _unknown_variant => Self::#other, },
        None => quote! { _unknown_variant => return Err(UnpackErr::UnknownVariant) },
    };

    Ok(quote! {
        #[automatically_derived]
        impl<#impl_generics> ::msgpck::MsgUnpack<'_msgpck> for #enum_name #ty_generics {
            fn unpack(bytes: &mut &'_msgpck [u8]) -> Result<Self, ::msgpck::UnpackErr>
            where
                Self: Sized + '_msgpck,
            {
                use ::msgpck::{UnpackErr, Variant::*, MsgUnpack};
                use ::msgpck::helpers::{unpack_enum_header, unpack_array_header};

                let header = unpack_enum_header(bytes)?;

                Ok(match &header.variant {
                    #unpack_variants
                    #unknown_variant_match
                })
            }
        }
    })
}
