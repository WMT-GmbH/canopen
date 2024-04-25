use proc_macro::TokenStream;
use std::collections::BTreeSet;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::*;

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

#[derive(Eq)]
struct Object {
    index: u16,
    subindex: u8,
    read_only: bool,
    write_only: bool,
    ident: Ident,
}

impl Object {
    fn flags(&self) -> TokenStream2 {
        let mut flags = quote!(::canopen::objectdictionary::object::ObjectFlags::empty());
        if self.read_only {
            flags = quote!(#flags.set_read_only());
        }
        if self.write_only {
            flags = quote!(#flags.set_write_only());
        }
        flags
    }
}

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

fn field_to_objects(field: &Field) -> Vec<Result<Object>> {
    let ident = field.ident.as_ref().expect("field should have name");
    let objects: Vec<_> = field
        .attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("canopen") {
                Some(attr_to_object(attr, ident.clone()))
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

fn attr_to_object(attr: &Attribute, ident: Ident) -> Result<Object> {
    let mut index: Option<u16> = None;
    let mut subindex: u8 = 0;
    let mut read_only = false;
    let mut write_only = false;
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
        Ok(())
    })?;
    let index = index.ok_or(Error::new(attr.span(), "Object must declare `index`"))?;
    Ok(Object {
        index,
        subindex,
        read_only,
        write_only,
        ident,
    })
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
