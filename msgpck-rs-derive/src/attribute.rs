use core::fmt;
use std::{collections::HashSet, fmt::Display};

use strum::{EnumIter, IntoEnumIterator};
use syn::meta::ParseNestedMeta;

use crate::DeriveKind;

#[derive(Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Attribute {
    /// Pack enum without including information about the variant.
    Untagged,

    /// When unpacking an enum, an unknown discriminant/name will unpack as the variant tagged with
    /// `#[msgpck_rs(other)]`. Same as `#[serde(other)]` Only allowed on a unit variant.
    Other,

    /// If the value is not present when deserializing, use the `Default::default()`.
    ///
    /// Same as `#[serde(default)]`
    Default,

    /// Skip this field when packing/unpacking
    ///
    /// Same as `#[serde(skip)]`
    Skip,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AttrLocation {
    Struct,
    StructField,
    Enum,
    EnumVariant,
    EnumVariantField,
}

impl Attribute {
    /// Get the "name" of the attribute, i.e. the exact string that is used when declaring the
    /// attribute.
    fn name(&self) -> &'static str {
        match self {
            Attribute::Untagged => "untagged",
            Attribute::Other => "other",
            Attribute::Default => "default",
            Attribute::Skip => "skip",
        }
    }

    /// Returns true if the attribute can be used at the specfied location for the specified derive.
    pub fn is_supported_at(&self, location: AttrLocation, derive: DeriveKind) -> bool {
        use AttrLocation::*;
        use DeriveKind::*;

        match (self, derive) {
            (Attribute::Untagged, MsgPack) => matches!(location, Enum),
            (Attribute::Untagged, MsgUnpack) => false,

            (Attribute::Other, MsgPack) => false,
            (Attribute::Other, MsgUnpack) => matches!(location, EnumVariant),

            (Attribute::Default, MsgPack) => false,
            (Attribute::Default, MsgUnpack) => matches!(location, EnumVariantField | StructField),

            (Attribute::Skip, _) => matches!(location, EnumVariantField | StructField),
        }
    }
}

impl Display for AttrLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttrLocation::Struct => write!(f, "struct"),
            AttrLocation::StructField => write!(f, "struct field"),
            AttrLocation::Enum => write!(f, "enum"),
            AttrLocation::EnumVariant => write!(f, "enum variant"),
            AttrLocation::EnumVariantField => write!(f, "enum variant field"),
        }
    }
}

pub fn parse_attributes(
    attrs: &[syn::Attribute],
    location: AttrLocation,
    kind: DeriveKind,
) -> syn::Result<HashSet<Attribute>> {
    let mut attributes = HashSet::new();
    for attr in attrs {
        if !attr.path().is_ident("msgpck_rs") {
            continue;
        }

        let attributes = &mut attributes;

        attr.parse_nested_meta(|meta| {
            // check if this attribute is filtering for a specific derive kind
            let specific_kind;
            if meta.path.is_ident("pack") {
                specific_kind = Some(DeriveKind::MsgPack);
            } else if meta.path.is_ident("unpack") {
                specific_kind = Some(DeriveKind::MsgUnpack);
            } else {
                specific_kind = None;
            };

            let check_attribute =
                !matches!(specific_kind, Some(specific_kind) if specific_kind != kind);

            let mut parse_arg_meta = |meta: ParseNestedMeta| {
                let mut all_attributes = Attribute::iter();
                let attribute = loop {
                    let Some(attribute) = all_attributes.next() else {
                        return Err(meta.error("unexpected attribute"));
                    };

                    if meta.path.is_ident(attribute.name()) {
                        break attribute;
                    }
                };

                if check_attribute && !attribute.is_supported_at(location, kind) {
                    return Err(meta.error(format!(
                        "this attribute isn't supported by {kind:?} on item \"{location}\""
                    )));
                }

                attributes.insert(attribute);

                Ok(())
            };

            if specific_kind.is_some() {
                meta.parse_nested_meta(parse_arg_meta)?;
            } else {
                parse_arg_meta(meta)?;
            }

            Ok(())
        })?;
    }

    Ok(attributes)
}
