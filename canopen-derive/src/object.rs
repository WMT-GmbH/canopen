use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::cell::Cell;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::*;

#[derive(Eq)]
pub struct Object {
    pub index: u16,
    pub subindex: u8,
    pub read_only: bool,
    pub write_only: bool,
    pub name: Option<String>,
    pub typ: Option<DataType>,
    pub ident: Ident,
}

impl Object {
    pub fn new(attr: &Attribute, ident: Ident, ty: Type) -> Result<Object> {
        let mut index: Option<u16> = None;
        let mut subindex: u8 = 0;
        let mut read_only = false;
        let mut write_only = false;
        let mut name = None;
        let mut typ = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("index") {
                let val: LitInt = meta.value()?.parse()?;
                index = Some(val.base10_parse()?);
            }
            if meta.path.is_ident("subindex") {
                let val: LitInt = meta.value()?.parse()?;
                subindex = val.base10_parse()?;
            }
            if meta.path.is_ident("read_only") {
                if write_only {
                    return Err(Error::new(
                        attr.span(),
                        "Object cannot be both read-only and write-only",
                    ));
                }
                read_only = true;
            }
            if meta.path.is_ident("write_only") {
                if read_only {
                    return Err(Error::new(
                        attr.span(),
                        "Object cannot be both read-only and write-only",
                    ));
                }
                write_only = true;
            }
            if meta.path.is_ident("name") {
                let val: LitStr = meta.value()?.parse()?;
                name = Some(val.value());
            }
            if meta.path.is_ident("type") {
                typ = Some(Self::parse_datatype(meta.value()?)?);
            }

            Ok(())
        })?;
        let index = index.ok_or(Error::new(attr.span(), "Object must declare `index`"))?;
        if typ.is_none() {
            typ = Self::guess_type(ty);
        }
        Ok(Object {
            index,
            subindex,
            read_only,
            write_only,
            name,
            typ,
            ident,
        })
    }

    pub fn flags(&self) -> TokenStream {
        let mut flags = quote!(::canopen::objectdictionary::object::ObjectFlags::empty());
        if self.read_only {
            flags = quote!(#flags.set_read_only());
        }
        if self.write_only {
            flags = quote!(#flags.set_write_only());
        }
        flags
    }

    fn parse_datatype(parse_stream: ParseStream) -> Result<DataType> {
        let val: LitInt = parse_stream.parse()?;
        let num: u8 = val.base10_parse()?;
        match num {
            0x1 => Ok(DataType::BOOLEAN),
            0x2 => Ok(DataType::INTEGER8),
            0x3 => Ok(DataType::INTEGER16),
            0x4 => Ok(DataType::INTEGER32),
            0x5 => Ok(DataType::UNSIGNED8),
            0x6 => Ok(DataType::UNSIGNED16),
            0x7 => Ok(DataType::UNSIGNED32),
            0x8 => Ok(DataType::REAL32),
            0x9 => Ok(DataType::VISIBLE_STRING),
            0xA => Ok(DataType::OCTET_STRING),
            0xB => Ok(DataType::UNICODE_STRING),
            0xC => Ok(DataType::TIME_OF_DAY),
            0xD => Ok(DataType::TIME_DIFFERENCE),
            0xF => Ok(DataType::DOMAIN),
            0x10 => Ok(DataType::INTEGER24),
            0x11 => Ok(DataType::REAL64),
            0x12 => Ok(DataType::INTEGER40),
            0x13 => Ok(DataType::INTEGER48),
            0x14 => Ok(DataType::INTEGER56),
            0x15 => Ok(DataType::INTEGER64),
            0x16 => Ok(DataType::UNSIGNED24),
            0x18 => Ok(DataType::UNSIGNED40),
            0x19 => Ok(DataType::UNSIGNED48),
            0x1A => Ok(DataType::UNSIGNED56),
            0x1B => Ok(DataType::UNSIGNED64),
            0x20 => Ok(DataType::PDO_COMMUNICATION_PARAMETER),
            0x21 => Ok(DataType::PDO_MAPPING),
            0x22 => Ok(DataType::SDO_PARAMETER),
            0x23 => Ok(DataType::IDENTITY),
            _ => Err(Error::new(val.span(), "Invalid data type")),
        }
    }

    fn guess_type(ty: Type) -> Option<DataType> {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                if path.is_ident("bool") {
                    Some(DataType::BOOLEAN)
                } else if path.is_ident("i8") {
                    Some(DataType::INTEGER8)
                } else if path.is_ident("i16") {
                    Some(DataType::INTEGER16)
                } else if path.is_ident("i32") {
                    Some(DataType::INTEGER32)
                } else if path.is_ident("u8") {
                    Some(DataType::UNSIGNED8)
                } else if path.is_ident("u16") {
                    Some(DataType::UNSIGNED16)
                } else if path.is_ident("u32") {
                    Some(DataType::UNSIGNED32)
                } else if path.is_ident("f32") {
                    Some(DataType::REAL32)
                } else if path.is_ident("str") {
                    Some(DataType::OCTET_STRING)
                } else {
                    None
                }
            }
            Type::Reference(reference) => Self::guess_type(*reference.elem),
            _ => None,
        }
    }
}

// Implement Ord and PartialOrd for Object so we can sort them
impl Ord for Object {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index
            .cmp(&other.index)
            .then_with(|| self.subindex.cmp(&other.subindex))
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.subindex == other.subindex
    }
}

/// Taken from CiA 301, Table 44: Object dictionary data types
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DataType {
    BOOLEAN = 0x1,
    INTEGER8 = 0x2,
    INTEGER16 = 0x3,
    INTEGER32 = 0x4,
    UNSIGNED8 = 0x5,
    UNSIGNED16 = 0x6,
    UNSIGNED32 = 0x7,
    REAL32 = 0x8,
    VISIBLE_STRING = 0x9,
    OCTET_STRING = 0xA,
    UNICODE_STRING = 0xB,
    TIME_OF_DAY = 0xC,
    TIME_DIFFERENCE = 0xD,
    // reserved = 0xE
    DOMAIN = 0xF,
    INTEGER24 = 0x10,
    REAL64 = 0x11,
    INTEGER40 = 0x12,
    INTEGER48 = 0x13,
    INTEGER56 = 0x14,
    INTEGER64 = 0x15,
    UNSIGNED24 = 0x16,
    // reserved = 0x17
    UNSIGNED40 = 0x18,
    UNSIGNED48 = 0x19,
    UNSIGNED56 = 0x1A,
    UNSIGNED64 = 0x1B,
    // reserved = 0x1C - 0x1F
    PDO_COMMUNICATION_PARAMETER = 0x20,
    PDO_MAPPING = 0x21,
    SDO_PARAMETER = 0x22,
    IDENTITY = 0x23,
}

/// Taken from CiA 301, Table 42: Object dictionary object definitions
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ObjectType {
    NULL = 0x0,
    DOMAIN = 0x2,
    DEFTYPE = 0x5,
    DEFSTRUCT = 0x6,
    VAR = 0x7,
    ARRAY = 0x8,
    RECORD = 0x9,
}
