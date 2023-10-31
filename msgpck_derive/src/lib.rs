#![allow(clippy::match_overlapping_arm)]

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::{
    parse_macro_input, spanned::Spanned, DataEnum, DataStruct, DeriveInput, Fields, GenericParam,
    Ident, Index, Member,
};

#[proc_macro_derive(MsgPck)]
pub fn derive_pack(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_pack_struct(&input, s),
        syn::Data::Enum(s) => derive_pack_enum(&input, s),
        syn::Data::Union(_) => quote! {
            compile_error!("derive(MsgPck) is not supported for unions");
        },
    }
    .into()
}

#[proc_macro_derive(AsyncMsgPck)]
pub fn derive_pack_async(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_pack_async_struct(&input, s),
        syn::Data::Enum(s) => derive_pack_async_enum(&input, s),
        syn::Data::Union(_) => quote! {
            compile_error!("derive(MsgPck) is not supported for unions");
        },
    }
    .into()
}

#[proc_macro_derive(UnMsgPck)]
pub fn derive_unpack(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_unpack_struct(&input, s),
        syn::Data::Enum(s) => derive_unpack_enum(&input, s),
        syn::Data::Union(_) => quote! {
            compile_error!("derive(UnMsgPck) is not supported for unions");
        },
    }
    .into()
}

/// Generate impl MsgPck for a struct
fn derive_pack_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let struct_len = data.fields.len();

    if let Some(where_clause) = &input.generics.where_clause {
        return quote_spanned! {
            where_clause.span() =>
            compile_error!("derive(MsgPck) doesn't support where-clauses for structs");
        };
    }

    let mut encode_body = quote! {};
    let mut generic_bounds = quote! {};
    let impl_generics = &input.generics.params;
    let mut struct_generics = quote!();
    let mut size_hint = quote! { ::msgpck::SizeHint { min: Some(0), max: Some(0) } };

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
                    #t: ::msgpck::MsgPck,
                });
            }
            GenericParam::Const(..) => continue,
        }
    }

    // serialize newtype structs without using array, this is to maintain compatibility with serde
    match &data.fields {
        Fields::Unnamed(..) if struct_len == 1 => {
            encode_body.append_all(quote! {});
        }
        _ => {
            encode_body.append_all(array_len_iter(data.fields.len()));
            let header_len = struct_len.to_be_bytes().len() + 1;
            size_hint.append_all(
                quote! { + ::msgpck::SizeHint { min: Some(1), max: Some(#header_len) } },
            );
        }
    }

    for (i, field) in data.fields.iter().enumerate() {
        let member = field.ident.clone().map(Member::Named).unwrap_or_else(|| {
            Member::Unnamed(Index {
                index: i as u32,
                span: field.span(),
            })
        });
        encode_body.append_all(quote_spanned! { field.span() =>
            ::msgpck::MsgPck::pack(&self.#member, writer)?;
        });

        size_hint.append_all(quote_spanned! { field.span() =>
            + ::msgpck::MsgPck::size_hint(&self.#member)
        });
    }

    quote! {
        impl<#impl_generics> ::msgpck::MsgPck for #struct_name<#struct_generics>
        where #generic_bounds {
            #[cfg_attr(feature = "reduce-size", inline(never))]
            fn pack(&self, writer: &mut dyn ::msgpck::MsgWriter) -> ::core::result::Result<(), ::msgpck::PackError> {
                #encode_body
                Ok(())
            }

            #[cfg(feature = "size-hints")]
            fn size_hint(&self) -> ::msgpck::SizeHint {
                #size_hint
            }
        }
    }
}

/// Generate impl AsyncMsgPck for a struct
fn derive_pack_async_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let struct_len = data.fields.len();

    if let Some(where_clause) = &input.generics.where_clause {
        return quote_spanned! {
            where_clause.span() =>
            compile_error!("derive(AsyncMsgPck) doesn't support where-clauses for structs");
        };
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
                    #t: ::msgpck::MsgPck,
                });
            }
            GenericParam::Const(..) => continue,
        }
    }

    // serialize newtype structs without using array, this is to maintain compatibility with serde
    match &data.fields {
        Fields::Unnamed(..) if struct_len == 1 => {
            encode_body.append_all(quote! {});
        }
        _ => {
            encode_body.append_all(array_len_iter_async(data.fields.len()));
        }
    }

    for (i, field) in data.fields.iter().enumerate() {
        let member = field.ident.clone().map(Member::Named).unwrap_or_else(|| {
            Member::Unnamed(Index {
                index: i as u32,
                span: field.span(),
            })
        });
        encode_body.append_all(quote_spanned! { field.span() =>
            ::msgpck::AsyncMsgPck::pack_async(&self.#member, &mut writer).await?;
        });
    }

    quote! {
        impl<#impl_generics> ::msgpck::AsyncMsgPck for #struct_name<#struct_generics>
        where #generic_bounds {
            #[cfg_attr(feature = "reduce-size", inline(never))]
            async fn pack_async(&self, mut writer: impl ::embedded_io_async::Write) -> ::core::result::Result<(), ::msgpck::PackError> {
                #encode_body
                Ok(())
            }
        }
    }
}

/// Generate impl UnMsgPck for a struct
fn derive_unpack_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let struct_len = data.fields.len();

    if let Some(where_clause) = &input.generics.where_clause {
        return quote_spanned! {
            where_clause.span() =>
            compile_error!("derive(UnMsgPck) doesn't support where-clauses for structs");
        };
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
                    '_MsgPck: #l,
                });
            }
            GenericParam::Type(t) => {
                let t = &t.ident;
                struct_generics.append_all(quote! { #t, });
                generic_bounds.append_all(quote! {
                    #t: ::msgpck::UnMsgPck<'_MsgPck>,
                });
            }
            GenericParam::Const(..) => continue,
        }
    }

    for field in data.fields.iter() {
        if let Some(ident) = &field.ident {
            unpack_fields.append_all(quote_spanned! { field.span() =>
                #ident: UnMsgPck::unpack(bytes)?,
            });
        } else {
            unpack_fields.append_all(quote_spanned! { field.span() =>
                UnMsgPck::unpack(bytes)?,
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
                // if cfg!(feature = "debug") {
                //     return Err(UnpackError::MissingFields {
                //         got: n,
                //         expected: #struct_len
                //     });
                // } else {
                    return Err(UnpackError::MissingFields);
                // }
            }
            if n > #struct_len {
                // if cfg!(feature = "debug") {
                //     return Err(UnpackError::TooManyFields {
                //         got: n,
                //         expected: #struct_len
                //     });
                // } else {
                    return Err(UnpackError::TooManyFields);
                // }
            }

            let value = Self #unpack_fields;

            Ok(value)
        }
    };

    quote! {
        impl<'_MsgPck, #impl_generics> ::msgpck::UnMsgPck<'_MsgPck> for #struct_name<#struct_generics>
        where #generic_bounds {
            #[cfg_attr(feature = "reduce-size", inline(never))]
            fn unpack(bytes: &mut &'_MsgPck [u8]) -> ::core::result::Result<Self, ::msgpck::UnpackError>
            where
                Self: Sized,
            {
                use ::msgpck::{UnMsgPck, UnpackError, unpack_array_header, Marker};

                #unpack_body
            }
        }

    }
}

/// Generate impl UnMsgPck for an enum
fn derive_unpack_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    if let Some(where_clause) = &input.generics.where_clause {
        return quote_spanned! {
            where_clause.span() =>
            compile_error!("derive(UnMsgPck) doesn't support where-clauses for enums");
        };
    }

    if !input.generics.params.is_empty() {
        return quote_spanned! {
            input.generics.params.span() =>
            compile_error!("derive(UnMsgPck) doesn't support generics for enums");
        };
    }

    let mut unpack_variants = quote! {};

    let mut discriminant = 0u64;
    for variant in &data.variants {
        if let Some((_, explicit_discriminant)) = &variant.discriminant {
            // TODO: handle explicitly setting the discriminant.
            // This loop should track the discriminant in the same fashion as rustc
            match explicit_discriminant {
                syn::Expr::Lit(_lit) => {
                    return quote_spanned! {
                        explicit_discriminant.span() =>
                        compile_error!("not supported (yet) by derive(UnMsgPck)");
                    }
                }
                _ => {
                    return quote_spanned! {
                        explicit_discriminant.span() =>
                        compile_error!("not supported by derive(UnMsgPck)");
                    }
                }
            }
        }

        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();
        let match_pattern = quote! {
            Discriminant(#discriminant) | Name(#variant_name_str)
        };

        let validate_type = match variant.fields.len() {
            0 => quote! {
                if !header.unit {
                    return Err(UnpackError::UnexpectedUnitVariant);
                }
            },
            1 => quote! {
                if header.unit {
                    return Err(UnpackError::ExpectedUnitVariant);
                }
            },
            n => quote! {
                if header.unit {
                    return Err(UnpackError::ExpectedUnitVariant);
                }

                let array_len = unpack_array_header(bytes)?;

                if array_len < #n {
                    return Err(UnpackError::MissingFields);
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
                    let field_name = field.ident.as_ref().unwrap();
                    construct_fields.append_all(quote! {
                        #field_name: UnMsgPck::unpack(bytes)?,
                    });
                }
            }
            syn::Fields::Unnamed(fields) => {
                for _ in &fields.unnamed {
                    construct_fields.append_all(quote! {
                        UnMsgPck::unpack(bytes)?,
                    });
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

    quote! {
        impl<'buf> ::msgpck::UnMsgPck<'buf> for #enum_name {
            #[cfg_attr(feature = "reduce-size", inline(never))]
            fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, ::msgpck::UnpackError>
            where
                Self: Sized + 'buf,
            {
                use ::msgpck::{UnpackError, Variant::*, UnMsgPck, unpack_enum_header, unpack_array_header};

                let header = unpack_enum_header(bytes)?;

                Ok(match &header.variant {
                    #unpack_variants
                    _unknown_variant => return Err(UnpackError::UnknownVariant)
                })
            }
        }
    }
}

/// Generate impl MsgPck for an enum
fn derive_pack_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    if let Some(where_clause) = &input.generics.where_clause {
        return quote_spanned! {
            where_clause.span() =>
            compile_error!("derive(MsgPck) doesn't support where-clauses for enums");
        };
    }

    if !input.generics.params.is_empty() {
        return quote_spanned! {
            input.generics.params.span() =>
            compile_error!("derive(MsgPck) doesn't support generics for enums");
        };
    }

    let mut iter_enum_generics = quote! {};
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
        //                compile_error!("not supported (yet) by derive(UnMsgPck)");
        //            };
        //        }
        //        _ => {
        //            return quote_spanned! {
        //                explicit_discriminant.span() =>
        //                compile_error!("not supported by derive(UnMsgPck)");
        //            };
        //        }
        //    }
        //}

        // generate stuff for the iterator enum
        iter_enum_generics.append_all(quote_spanned! { variant.span() => #variant_name,});

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
                        ::msgpck::pack_array_header(writer, #fields_len)?;
                    });
                }

                unit = fields_len == 0;

                for field in &fields.named {
                    let field_name = &field.ident;
                    // pattern match all the fields
                    match_fields.append_all(quote_spanned! { field.span() => #field_name, });

                    // pack all the named fields
                    pack_fields.append_all(quote_spanned! { field.span() =>
                        ::msgpck::MsgPck::pack(&#field_name, writer)?;
                    })
                }

                // wrap fields pattern in brackets
                match_fields = quote! { { #match_fields } };
            }
            syn::Fields::Unnamed(fields) => {
                // if there is more than one field, pack them as an array
                let fields_len = fields.unnamed.len();
                if fields_len > 1 {
                    pack_fields.append_all(quote! {
                        ::msgpck::pack_array_header(writer, #fields_len)?;
                    });
                }

                unit = fields_len == 0;

                for (i, field) in fields.unnamed.iter().enumerate() {
                    let field_name = Ident::new(&format!("_{i}"), field.span());
                    // pattern match all the fields
                    match_fields.append_all(quote! { #field_name, });

                    // pack all the fields
                    pack_fields.append_all(quote! {
                        ::msgpck::MsgPck::pack(&#field_name, writer)?;
                    });
                }

                // wrap fields pattern in parentheses
                match_fields = quote! { (#match_fields) };
            }
            syn::Fields::Unit => unit = true,
        }

        pack_variants.append_all(quote! {
            Self::#variant_name #match_fields => {
                ::msgpck::MsgPck::pack(&::msgpck::EnumHeader {
                    variant: #variant_name_str.into(),
                    unit: #unit,
                }, writer)?;
                #pack_fields
            }
        });
    }

    quote! {
        impl ::msgpck::MsgPck for #enum_name {
            #[cfg_attr(feature = "reduce-size", inline(never))]
            fn pack(&self, writer: &mut dyn ::msgpck::MsgWriter) -> Result<(), ::msgpck::PackError> {
                match self {
                    #pack_variants
                }
                Ok(())
            }
        }
    }
}

/// Generate impl AsyncMsgPck for an enum
fn derive_pack_async_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    if let Some(where_clause) = &input.generics.where_clause {
        return quote_spanned! {
            where_clause.span() =>
            compile_error!("derive(MsgPck) doesn't support where-clauses for enums");
        };
    }

    if !input.generics.params.is_empty() {
        return quote_spanned! {
            input.generics.params.span() =>
            compile_error!("derive(MsgPck) doesn't support generics for enums");
        };
    }

    let mut pack_variants = quote! {};

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

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
                        ::msgpck::utils::pack_array_header_async(&mut writer, #fields_len).await?;
                    });
                }

                unit = fields_len == 0;

                for field in &fields.named {
                    let field_name = &field.ident;
                    // pattern match all the fields
                    match_fields.append_all(quote_spanned! { field.span() => #field_name, });

                    // pack all the named fields
                    pack_fields.append_all(quote_spanned! { field.span() =>
                        ::msgpck::AsyncMsgPck::pack_async(&#field_name, &mut writer).await?;
                    })
                }

                // wrap fields pattern in brackets
                match_fields = quote! { { #match_fields } };
            }
            syn::Fields::Unnamed(fields) => {
                // if there is more than one field, pack them as an array
                let fields_len = fields.unnamed.len();
                if fields_len > 1 {
                    pack_fields.append_all(quote! {
                        ::msgpck::utils::pack_array_header_async(&mut writer, #fields_len).await?;
                    });
                }

                unit = fields_len == 0;

                for (i, field) in fields.unnamed.iter().enumerate() {
                    let field_name = Ident::new(&format!("_{i}"), field.span());
                    // pattern match all the fields
                    match_fields.append_all(quote! { #field_name, });

                    // pack all the fields
                    pack_fields.append_all(quote! {
                        ::msgpck::AsyncMsgPck::pack_async(&#field_name, &mut writer).await?;
                    });
                }

                // wrap fields pattern in parentheses
                match_fields = quote! { (#match_fields) };
            }
            syn::Fields::Unit => unit = true,
        }

        pack_variants.append_all(quote! {
            Self::#variant_name #match_fields => {
                ::msgpck::AsyncMsgPck::pack_async(&::msgpck::EnumHeader {
                    variant: #variant_name_str.into(),
                    unit: #unit,
                }, &mut writer).await?;
                #pack_fields
            }
        });
    }

    quote! {
        impl ::msgpck::AsyncMsgPck for #enum_name {
            #[cfg_attr(feature = "reduce-size", inline(never))]
            async fn pack_async(&self, mut writer: impl ::embedded_io_async::Write) -> ::core::result::Result<(), ::msgpck::PackError> {
                match self {
                    #pack_variants
                }
                Ok(())
            }
        }
    }
}

fn array_len_iter(len: usize) -> TokenStream {
    match len {
        ..=0xf => {
            let len = len as u8;
            quote! { writer.write(&[::msgpck::Marker::FixArray(#len).to_u8()])?; }
        }
        ..=0xffff => {
            let len = len as u16;
            quote! {
                writer.write(&[::msgpck::Marker::Array16.to_u8()])?;
                writer.write(&((#len as u16).to_be_bytes()))?;
            }
        }
        _ => {
            let len = len as u32;
            quote! {
                writer.write(&[::msgpck::Marker::Array32.to_u8()])?;
                writer.write(&((#len as u32).to_be_bytes()))?;
            }
        }
    }
}

fn array_len_iter_async(len: usize) -> TokenStream {
    match len {
        ..=0xf => {
            let len = len as u8;
            quote! { writer.write_all(&[::msgpck::Marker::FixArray(#len).to_u8()]).await?; }
        }
        ..=0xffff => {
            let len = len as u16;
            quote! {
                writer.write_all(&[::msgpck::Marker::Array16.to_u8()]).await?;
                writer.write_all(&((#len as u16).to_be_bytes())).await?;
            }
        }
        _ => {
            let len = len as u32;
            quote! {
                writer.write_all(&[::msgpck::Marker::Array32.to_u8()]).await?;
                writer.write_all(&((#len as u32).to_be_bytes())).await?;
            }
        }
    }
}
