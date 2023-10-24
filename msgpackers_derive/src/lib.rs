#![allow(clippy::match_overlapping_arm)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned, DataEnum, DataStruct, DeriveInput};

#[proc_macro_derive(MsgPack)]
pub fn derive_pack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_pack_struct(&input, s),
        syn::Data::Enum(s) => derive_pack_enum(&input, s),
        syn::Data::Union(_) => quote! {
            compile_error!("derive(MsgPack) is not supported for unions");
        }
        .into(),
    }
}

#[proc_macro_derive(MsgUnpack)]
pub fn derive_unpack(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(s) => derive_unpack_struct(&input, s),
        syn::Data::Enum(s) => derive_unpack_enum(&input, s),
        syn::Data::Union(_) => quote! {
            compile_error!("derive(MsgUnpack) is not supported for unions");
        }
        .into(),
    }
}

/// Generate impl MsgPack for a struct
fn derive_pack_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let mut encode_body = array_len_iter(data.fields.len());

    for (i, field) in data.fields.iter().enumerate() {
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("{i}"), field.span()));
        encode_body.append_all(quote! {
            .chain(self.#ident.pack())
        });
    }

    quote! {
        impl msgpackers::MsgPack for #struct_name {
            type Iter<'a> = impl Iterator<Item = msgpackers::Piece<'a>>
            where
                Self: 'a;

            fn pack<'a>(&'a self) -> Self::Iter<'a> {
                use ::std::iter::once;
                use ::msgpackers::Marker;
                #encode_body
            }
        }
    }
    .into()
}

/// Generate impl MsgUnpack for a struct
fn derive_unpack_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let struct_name = &input.ident;
    let struct_len = data.fields.len();
    let mut unpack_body = quote! {};

    for (i, field) in data.fields.iter().enumerate() {
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("{i}"), field.span()));
        unpack_body.append_all(quote! {
            #ident: MsgUnpack::unpack(bytes)?,
        });
    }

    quote! {
        impl<'buf> ::msgpackers::MsgUnpack<'buf> for #struct_name {
            fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, ::msgpackers::UnpackErr>
            where
                Self: Sized,
            {
                use ::msgpackers::{MsgUnpack, UnpackErr, unpack_array_header};
                let n = unpack_array_header(bytes)?;

                if n < #struct_len {
                    return Err(UnpackErr::UnexpectedEof);
                }

                let value = Self {
                    #unpack_body
                };

                if n > #struct_len {
                    return Err(UnpackErr::UnexpectedEof);
                }

                // TODO: be lenient with parsing
                //for _ in #struct_len..=n {
                //    let _ = MsgUnpack::unpack(bytes)?;
                //}

                Ok(value)
            }
        }

    }
    .into()
}

/// Generate impl MsgUnpack for an enum
fn derive_unpack_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

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
                        compile_error!("not supported (yet) by derive(MsgUnpack)");
                    }
                    .into();
                }
                _ => {
                    return quote_spanned! {
                        explicit_discriminant.span() =>
                        compile_error!("not supported by derive(MsgUnpack)");
                    }
                    .into();
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
                    let field_name = field.ident.as_ref().unwrap();
                    construct_fields.append_all(quote! {
                        #field_name: MsgUnpack::unpack(bytes)?,
                    });
                }
            }
            syn::Fields::Unnamed(fields) => {
                for _ in &fields.unnamed {
                    construct_fields.append_all(quote! {
                        MsgUnpack::unpack(bytes)?,
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
        impl<'buf> ::msgpackers::MsgUnpack<'buf> for #enum_name {
            fn unpack(bytes: &mut &'buf [u8]) -> Result<Self, ::msgpackers::UnpackErr>
            where
                Self: Sized + 'buf,
            {
                use ::msgpackers::{UnpackErr, Variant::*, MsgUnpack, unpack_enum_header, unpack_array_header};

                let header = unpack_enum_header(bytes)?;

                Ok(match &header.variant {
                    #unpack_variants
                    _unknown_variant => return Err(UnpackErr::UnknownVariant)
                })
            }
        }
    }
    .into()
}

/// Generate impl MsgPack for an enum
fn derive_pack_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

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
        //            }
        //            .into();
        //        }
        //        _ => {
        //            return quote_spanned! {
        //                explicit_discriminant.span() =>
        //                compile_error!("not supported by derive(MsgUnpack)");
        //            }
        //            .into();
        //        }
        //    }
        //}

        // generate stuff for the iterator enum
        iter_enum_generics.append_all(quote! {#variant_name,});
        iter_enum_variants.append_all(quote! {#variant_name(#variant_name),});
        iter_enum_bounds
            .append_all(quote! {#variant_name: Iterator<Item = ::msgpackers::Piece<'a>>,});
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
                        .chain(::msgpackers::pack_array_header(#fields_len))
                    });
                }

                unit = fields_len == 0;

                for field in &fields.named {
                    let field_name = &field.ident;
                    // pattern match all the fields
                    match_fields.append_all(quote! {#field_name, });

                    // pack all the named fields
                    pack_fields.append_all(quote! {
                        .chain(#field_name.pack())
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
                        .chain(::msgpackers::pack_array_header(#fields_len))
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

        pack_variants.append_all(quote! {
            Self::#variant_name #match_fields => {
                __MsgpackerIter::#variant_name(
                    ::msgpackers::pack_enum_header(::msgpackers::EnumHeader {
                        variant: #variant_name_str.into(),
                        unit: #unit,
                    })
                    #pack_fields
                )
            }
        });
    }

    quote! {
        impl ::msgpackers::MsgPack for #enum_name {
            type Iter<'a> = impl Iterator<Item = Piece<'a>>
            where
                Self: 'a;

            fn pack(&self) -> Self::Iter<'_> {

                // Because we need different msgpack iterator types for each variant, we need an
                // enum type that impls Iterator to contain them. To avoid naming the inner iterator
                // types, we use generics. It's not the prettiest, but it works.
                enum __MsgpackerIter<#iter_enum_generics> {
                    #iter_enum_variants
                }

                impl<'a, #iter_enum_bounds> Iterator for __MsgpackerIter<#iter_enum_generics> {
                    type Item = Piece<'a>;

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
    }
    .into()
}

fn array_len_iter(len: usize) -> proc_macro2::TokenStream {
    match len {
        ..=0xf => {
            let len = len as u8;
            quote! { once(Marker::FixArray(#len).into()) }
        }
        ..=0xffff => {
            let len = len as u16;
            quote! {
                [
                    Marker::Array16.into(),
                    #len.into(),
                ].into_iter()
            }
        }
        _ => {
            let len = len as u32;
            quote! {
                [
                    Marker::Array32.into(),
                    #len.into(),
                ].into_iter()
            }
        }
    }
}
