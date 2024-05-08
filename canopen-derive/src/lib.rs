// TODO migrate to darling

use proc_macro::TokenStream;
use std::collections::BTreeSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::*;

use crate::object::Object;

mod eds;
mod object;

#[proc_macro_derive(OdData, attributes(canopen))]
pub fn derive_interactive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemStruct);
    od_data_impl(&ast)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn od_data_impl(ast: &ItemStruct) -> Result<TokenStream2> {
    let struct_name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    match &ast.fields {
        Fields::Named(_) => {}
        _ => return Err(Error::new(ast.span(), "struct must have named fields")),
    }

    // convert fields to Objects's or collect per-field errors into on large error
    let mut objects = ast.fields.iter().map(field_to_objects).flatten().fold(
        Ok(Vec::new()),
        |acc: Result<Vec<Object>>, var| match (acc, var) {
            (Ok(mut vec), Ok(var)) => {
                vec.push(var);
                Ok(vec)
            }
            (Err(acc_err), Ok(_)) => Err(acc_err),
            (Err(mut acc_err), Err(e)) => {
                acc_err.combine(e);
                Err(acc_err)
            }
            (Ok(_), Err(e)) => Err(e),
        },
    )?;

    objects.sort_unstable_by_key(|a| (a.index, a.subindex));
    let od_size = objects.len();
    check_for_duplicates(&objects)?;

    if let Some(top_level_attr) = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("canopen"))
    {
        top_level_attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("eds_path") {
                let path: LitStr = meta.value()?.parse()?;
                eds::write_eds(std::path::Path::new(&path.value()), &objects).unwrap()
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

fn field_to_objects(field: &Field) -> Vec<Result<Object>> {
    let ident = field.ident.as_ref().expect("field should have name");
    let objects: Vec<_> = field
        .attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("canopen") {
                Some(Object::new(attr, ident.clone(), field.ty.clone()))
            } else {
                None
            }
        })
        .collect();
    if objects.is_empty() {
        vec![Err(Error::new(
            field.span(),
            "field must have #[canopen()] attribute",
        ))]
    } else {
        objects
    }
}

// expects objects to be sorted
fn check_for_duplicates(objects: &[Object]) -> Result<()> {
    let Some((first, rest)) = objects.split_first() else {
        return Ok(());
    };

    let mut duplicates = BTreeSet::new();
    let mut last_object = first;
    for object in rest {
        if object.index == last_object.index && object.subindex == last_object.subindex {
            duplicates.insert(last_object);
            duplicates.insert(object);
        } else {
            last_object = object;
        }
    }

    if let Some(e) = duplicates
        .into_iter()
        .map(|object| {
            Error::new(
                object.ident.span(),
                "Duplicate index and subindex combination",
            )
        })
        .reduce(|mut acc, e| {
            acc.combine(e);
            acc
        })
    {
        Err(e)
    } else {
        Ok(())
    }
}
