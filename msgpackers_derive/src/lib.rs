#![allow(clippy::match_overlapping_arm)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned, DataEnum, DataStruct, DeriveInput};

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

fn derive_unpack_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let enum_name = &input.ident;

    let mut unpack_variants = quote! {};

    let mut discriminator = 0u64;
    for variant in &data.variants {
        if let Some((_, explicit_discriminant)) = &variant.discriminant {
            // TODO
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
            Discriminator(#discriminator) | Name(#variant_name_str)
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

        discriminator += 1;
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

fn derive_pack_enum(_input: &DeriveInput, _data: &DataEnum) -> TokenStream {
    todo!("derive pack for enums")

    //let enum_name = &input.ident;
    //quote! {
    //    type Iter<'a> = impl Iterator<Item = Piece<'a>>
    //    where
    //        Self: 'a;

    //    impl ::msgpackers::MsgUnpack for #enum_name {
    //        fn pack(&self) -> Self::Iter<'_> {
    //            let header = match self {
    //                #pack_headers
    //            };

    //            ::msgpackers::pack_enum_header(header)?;
    //        }
    //    }
    //}
    //.into()
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
