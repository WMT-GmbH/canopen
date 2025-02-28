use std::slice;

use darling::{Error, FromAttributes, FromMeta, Result};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::*;

pub fn extract_field_info(field: &Field) -> Result<FieldInfo> {
    let ident = field.ident.as_ref().expect("field should have name");
    let mut errors = Error::accumulator();
    let mut records = Vec::new();
    let mut objects = Vec::new();

    for attr in &field.attrs {
        // try to parse as a record, then as an array, then as an object
        if let Ok(record) = RecordParser::from_attributes(slice::from_ref(attr)) {
            records.push(Record {
                ident: ident.clone(),
                index: record.index,
                name: record.record,
                is_array: false,
            });
        } else if let Ok(array) = ArrayParser::from_attributes(slice::from_ref(attr)) {
            array.generate_objects(ident, &mut objects);
            records.push(Record {
                ident: ident.clone(),
                index: array.index,
                name: array.array,
                is_array: true,
            });
        } else {
            if let Some(object) = errors.handle(Object::new(attr, ident.clone(), &field.ty)) {
                objects.push(object);
            }
        }
    }

    errors.finish()?;

    if objects.is_empty() {
        return Err(
            Error::custom("field must have at least one #[canopen()] attribute").with_span(field),
        );
    }

    Ok(FieldInfo { objects, records })
}

pub struct FieldInfo {
    pub objects: Vec<Object>,
    pub records: Vec<Record>,
}

#[derive(Eq, Debug)]
pub struct Object {
    pub ident: Ident,
    pub index: u16,
    pub subindex: u8,
    pub read_only: bool,
    pub write_only: bool,
    pub name: Option<String>,
    pub typ: Option<DataType>,
}

#[derive(Eq, Debug)]
pub struct Record {
    pub ident: Ident,
    pub index: u16,
    pub name: String,
    pub is_array: bool,
}

#[derive(darling::FromAttributes)]
#[darling(attributes(canopen))]
struct ObjectParser {
    index: u16,
    #[darling(default)]
    subindex: Option<u8>,
    #[darling(default)]
    read_only: bool,
    #[darling(default)]
    write_only: bool,
    #[darling(default)]
    name: Option<String>,
    #[darling(default, and_then = "Object::parse_datatype")]
    typ: Option<DataType>,
}

impl Object {
    pub fn new(attr: &Attribute, ident: Ident, typ: &Type) -> Result<Self> {
        let object =
            ObjectParser::from_attributes(slice::from_ref(attr)).map_err(|e| e.with_span(attr))?;
        if object.read_only && object.write_only {
            return Err(
                Error::custom("Object cannot be both read-only and write-only").with_span(attr),
            );
        }
        let mut object = Object {
            ident,
            index: object.index,
            subindex: object.subindex.unwrap_or(0),
            read_only: object.read_only,
            write_only: object.write_only,
            name: object.name,
            typ: object.typ,
        };

        if object.typ.is_none() {
            object.typ = Object::guess_type(typ);
        }

        Ok(object)
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

    fn parse_datatype(val: Expr) -> Result<Option<DataType>> {
        DataType::from_expr(&val).map(Some)
    }

    fn guess_type(ty: &Type) -> Option<DataType> {
        match ty {
            Type::Path(TypePath { path, .. }) => DataType::from_rust_type(path),
            Type::Reference(reference) => Self::guess_type(&reference.elem),
            _ => None,
        }
    }
}

#[derive(darling::FromAttributes)]
#[darling(attributes(canopen))]
pub struct RecordParser {
    pub record: String,
    pub index: u16,
}

#[derive(darling::FromAttributes)]
#[darling(attributes(canopen))]
pub struct ArrayParser {
    pub array: String,
    pub index: u16,
    pub size: u8,
    pub typ: DataType,
    #[darling(default)]
    pub read_only: bool,
    #[darling(default)]
    pub write_only: bool,
}

impl ArrayParser {
    fn generate_objects(&self, ident: &Ident, objects: &mut Vec<Object>) {
        let array_len = Object {
            ident: ident.clone(),
            index: self.index,
            subindex: 0,
            read_only: true,
            write_only: false,
            name: None,
            typ: Some(DataType::UNSIGNED8),
        };
        objects.push(array_len);
        for i in 1..=self.size {
            let array_element = Object {
                ident: ident.clone(),
                index: self.index,
                subindex: i,
                read_only: self.read_only,
                write_only: self.write_only,
                name: None,
                typ: Some(self.typ),
            };
            objects.push(array_element);
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

// Implement Ord and PartialOrd for Record so we can sort them
impl Ord for Record {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

/// Taken from CiA 301, Table 44: Object dictionary data types
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

impl FromMeta for DataType {
    fn from_expr(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Lit(ExprLit { lit, .. }) => {
                let num = u8::from_value(&lit)?;
                match DataType::from_u8(num) {
                    Some(typ) => Ok(typ),
                    None => Err(Error::custom("invalid data type")),
                }
            }
            Expr::Path(ExprPath { path, .. }) => match DataType::from_rust_type(&path) {
                Some(typ) => Ok(typ),
                None => match DataType::from_str(&path) {
                    Some(typ) => Ok(typ),
                    None => Err(Error::custom("invalid data type")),
                },
            },
            _ => Err(Error::unexpected_expr_type(&expr)),
        }
    }
}

impl DataType {
    fn from_u8(val: u8) -> Option<DataType> {
        match val {
            0x1 => Some(DataType::BOOLEAN),
            0x2 => Some(DataType::INTEGER8),
            0x3 => Some(DataType::INTEGER16),
            0x4 => Some(DataType::INTEGER32),
            0x5 => Some(DataType::UNSIGNED8),
            0x6 => Some(DataType::UNSIGNED16),
            0x7 => Some(DataType::UNSIGNED32),
            0x8 => Some(DataType::REAL32),
            0x9 => Some(DataType::VISIBLE_STRING),
            0xA => Some(DataType::OCTET_STRING),
            0xB => Some(DataType::UNICODE_STRING),
            0xC => Some(DataType::TIME_OF_DAY),
            0xD => Some(DataType::TIME_DIFFERENCE),
            0xF => Some(DataType::DOMAIN),
            0x10 => Some(DataType::INTEGER24),
            0x11 => Some(DataType::REAL64),
            0x12 => Some(DataType::INTEGER40),
            0x13 => Some(DataType::INTEGER48),
            0x14 => Some(DataType::INTEGER56),
            0x15 => Some(DataType::INTEGER64),
            0x16 => Some(DataType::UNSIGNED24),
            0x18 => Some(DataType::UNSIGNED40),
            0x19 => Some(DataType::UNSIGNED48),
            0x1A => Some(DataType::UNSIGNED56),
            0x1B => Some(DataType::UNSIGNED64),
            0x20 => Some(DataType::PDO_COMMUNICATION_PARAMETER),
            0x21 => Some(DataType::PDO_MAPPING),
            0x22 => Some(DataType::SDO_PARAMETER),
            0x23 => Some(DataType::IDENTITY),
            _ => None,
        }
    }

    fn from_str(path: &Path) -> Option<DataType> {
        if path.is_ident("BOOLEAN") {
            Some(DataType::BOOLEAN)
        } else if path.is_ident("INTEGER8") {
            Some(DataType::INTEGER8)
        } else if path.is_ident("INTEGER16") {
            Some(DataType::INTEGER16)
        } else if path.is_ident("INTEGER32") {
            Some(DataType::INTEGER32)
        } else if path.is_ident("UNSIGNED8") {
            Some(DataType::UNSIGNED8)
        } else if path.is_ident("UNSIGNED16") {
            Some(DataType::UNSIGNED16)
        } else if path.is_ident("UNSIGNED32") {
            Some(DataType::UNSIGNED32)
        } else if path.is_ident("REAL32") {
            Some(DataType::REAL32)
        } else if path.is_ident("VISIBLE_STRING") {
            Some(DataType::VISIBLE_STRING)
        } else if path.is_ident("OCTET_STRING") {
            Some(DataType::OCTET_STRING)
        } else if path.is_ident("UNICODE_STRING") {
            Some(DataType::UNICODE_STRING)
        } else if path.is_ident("TIME_OF_DAY") {
            Some(DataType::TIME_OF_DAY)
        } else if path.is_ident("TIME_DIFFERENCE") {
            Some(DataType::TIME_DIFFERENCE)
        } else if path.is_ident("DOMAIN") {
            Some(DataType::DOMAIN)
        } else if path.is_ident("INTEGER24") {
            Some(DataType::INTEGER24)
        } else if path.is_ident("REAL64") {
            Some(DataType::REAL64)
        } else if path.is_ident("INTEGER40") {
            Some(DataType::INTEGER40)
        } else if path.is_ident("INTEGER48") {
            Some(DataType::INTEGER48)
        } else if path.is_ident("INTEGER56") {
            Some(DataType::INTEGER56)
        } else if path.is_ident("INTEGER64") {
            Some(DataType::INTEGER64)
        } else if path.is_ident("UNSIGNED24") {
            Some(DataType::UNSIGNED24)
        } else if path.is_ident("UNSIGNED40") {
            Some(DataType::UNSIGNED40)
        } else if path.is_ident("UNSIGNED48") {
            Some(DataType::UNSIGNED48)
        } else if path.is_ident("UNSIGNED56") {
            Some(DataType::UNSIGNED56)
        } else if path.is_ident("UNSIGNED64") {
            Some(DataType::UNSIGNED64)
        } else if path.is_ident("PDO_COMMUNICATION_PARAMETER") {
            Some(DataType::PDO_COMMUNICATION_PARAMETER)
        } else if path.is_ident("PDO_MAPPING") {
            Some(DataType::PDO_MAPPING)
        } else if path.is_ident("SDO_PARAMETER") {
            Some(DataType::SDO_PARAMETER)
        } else if path.is_ident("IDENTITY") {
            Some(DataType::IDENTITY)
        } else {
            None
        }
    }

    fn from_rust_type(path: &Path) -> Option<DataType> {
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
}

/// Taken from CiA 301, Table 42: Object dictionary object definitions
#[allow(non_camel_case_types, unused, clippy::upper_case_acronyms)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datatype() {
        let object =
            ObjectParser::from_attributes(&[parse_quote!(#[canopen(index = 0x1000, typ = 0x1)])])
                .expect("Failed to parse attribute");
        assert_eq!(object.typ, Some(DataType::BOOLEAN));

        let object =
            ObjectParser::from_attributes(&[parse_quote!(#[canopen(index = 0x1000, typ = bool)])])
                .expect("Failed to parse attribute");
        assert_eq!(object.typ, Some(DataType::BOOLEAN));
    }
}
