use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use syn::*;

#[proc_macro_derive(OdData, attributes(canopen))]
pub fn derive_interactive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemStruct);
    od_data_impl(&ast).into()
}

fn od_data_impl(ast: &ItemStruct) -> TokenStream2 {
    let struct_name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut variables: Vec<_> = ast
        .fields
        .iter()
        .map(|field| field_to_variables(field))
        .flatten()
        .collect();
    variables.sort_unstable_by_key(|a| (a.index, a.subindex));
    let od_size = variables.len();
    // TODO assert uniqueness
    // TODO assert every field is a variable

    let indices = variables.iter().map(|v| v.index);
    let subindices = variables.iter().map(|v| v.subindex);
    let pdo_sizes = variables.iter().map(|_| quote!(None));
    let idents: Vec<_> = variables.iter().map(|v| &v.ident).collect();

    quote! {
        impl #impl_generics ::canopen::objectdictionary::OdData for #struct_name #ty_generics #where_clause {
            type OdType = ::canopen::objectdictionary::ObjectDictionary<#struct_name, #od_size>;

            fn into_od(self) -> Self::OdType {
                unsafe {
                    ::canopen::objectdictionary::ObjectDictionary::new(
                        [#(#indices),*],
                        [#(#subindices),*],
                        [#(#pdo_sizes),*],
                        [#(::core::mem::offset_of!(#struct_name, #idents)),*],
                        [#(::canopen::meta::metadata(&self.#idents as &dyn ::canopen::objectdictionary::datalink::DataLink)),*],
                        self,
                    )
                }
            }
        }
    }
}

struct Variable {
    index: u16,
    subindex: u8,
    ident: Ident,
}

fn field_to_variables(field: &Field) -> impl Iterator<Item = Variable> + '_ {
    field.attrs.iter().filter_map(|attr| {
        if attr.path().is_ident("canopen") {
            let mut index: Option<u16> = None;
            let mut subindex: u8 = 0;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("index") {
                    let val: LitInt = meta.value()?.parse()?;
                    index = Some(val.base10_parse()?);
                }
                if meta.path.is_ident("subindex") {
                    let val: LitInt = meta.value()?.parse()?;
                    subindex = val.base10_parse()?;
                }
                Ok(())
            })
            .unwrap();
            let index = index.expect("missing index"); // TODO spans;
            Some(Variable {
                index,
                subindex,
                ident: field.ident.clone().unwrap(),
            })
        } else {
            None
        }
    })
}
