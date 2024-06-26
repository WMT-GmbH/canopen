use proc_macro::TokenStream;
use std::collections::BTreeSet;

use darling::{Error, Result};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::*;

use crate::object::{extract_field_info, Object, Record};

mod eds;
mod object;

#[proc_macro_derive(OdData, attributes(canopen))]
pub fn derive_interactive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemStruct);
    od_data_impl(&ast)
        .unwrap_or_else(|e| e.write_errors())
        .into()
}

fn od_data_impl(ast: &ItemStruct) -> Result<TokenStream2> {
    let struct_name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    if !matches!(ast.fields, Fields::Named(_)) {
        return Err(Error::custom("struct must have named fields").with_span(ast));
    }

    let (objects, records) = get_objects_and_records(ast)?;
    let od_size = objects.len();

    if let Some(top_level_attr) = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("canopen"))
    {
        top_level_attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("eds_path") {
                let path: LitStr = meta.value()?.parse()?;
                eds::write_eds(std::path::Path::new(&path.value()), &objects, &records).unwrap()
            }
            Ok(())
        })?;
    }

    let indices = objects.iter().map(|v| v.index);
    let subindices = objects.iter().map(|v| v.subindex);
    let flags = objects.iter().map(Object::flags);
    let idents: Vec<_> = objects.iter().map(|v| &v.ident).collect();

    Ok(quote! {
        impl #impl_generics ::canopen::objectdictionary::OdData for #struct_name #ty_generics #where_clause {
            type OdType = ::canopen::objectdictionary::ObjectDictionary<#struct_name #ty_generics, #od_size>;

            fn into_od(self) -> Self::OdType {
                unsafe {
                    ::canopen::objectdictionary::ObjectDictionary::new(
                        [#(#indices),*],
                        [#(#subindices),*],
                        [#(#flags),*],
                        [#(::core::mem::offset_of!(#struct_name #ty_generics, #idents)),*],
                        [#(::canopen::meta::metadata(&self.#idents as &dyn ::canopen::objectdictionary::datalink::DataLink)),*],
                        self,
                    )
                }
            }
        }
    })
}

/// convert fields to `Objects`s and `Record`s and sort/de-duplicate them
fn get_objects_and_records(ast: &ItemStruct) -> Result<(Vec<Object>, Vec<Record>)> {
    let mut errors = Error::accumulator();

    let mut objects = Vec::new();
    let mut records = Vec::new();
    for field_info in ast
        .fields
        .iter()
        .filter_map(|field| errors.handle(extract_field_info(field)))
    {
        objects.extend(field_info.objects);
        records.extend(field_info.records);
    }
    errors.finish()?;

    objects.sort_unstable_by_key(|o| (o.index, o.subindex));
    check_for_duplicates(&objects, |o| (o.index, o.subindex), |o| &o.ident)?;
    records.sort_unstable_by_key(|r| r.index);
    check_for_duplicates(&records, |r| r.index, |o| &o.ident)?;
    Ok((objects, records))
}

// expects items to be sorted
fn check_for_duplicates<T: Ord, K: PartialEq>(
    objects: &[T],
    f: impl Fn(&T) -> K,
    f2: impl Fn(&T) -> &Ident,
) -> Result<()> {
    let Some((first, rest)) = objects.split_first() else {
        return Ok(());
    };

    let mut duplicates = BTreeSet::new();
    let mut last_object = first;
    for object in rest {
        if f(object) == f(last_object) {
            duplicates.insert(last_object);
            duplicates.insert(object);
        } else {
            last_object = object;
        }
    }

    let mut errors = Error::accumulator();
    for item in duplicates {
        errors.push(Error::custom("Duplicate index and subindex combination").with_span(f2(item)));
    }
    errors.finish()?;
    Ok(())
}
