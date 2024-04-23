use proc_macro::TokenStream;
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

    // convert fields to Variable's or collect per-field errors into on large error
    let mut variables = ast.fields.iter().map(field_to_variables).flatten().fold(
        Ok(Vec::new()),
        |acc: Result<Vec<Variable>>, var| match (acc, var) {
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

    variables.sort_unstable_by_key(|a| (a.index, a.subindex));
    let od_size = variables.len();
    // TODO assert uniqueness
    // TODO assert every field is a variable

    let indices = variables.iter().map(|v| v.index);
    let subindices = variables.iter().map(|v| v.subindex);
    let flags = variables.iter().map(Variable::flags);
    let idents: Vec<_> = variables.iter().map(|v| &v.ident).collect();

    Ok(quote! {
        impl #impl_generics ::canopen::objectdictionary::OdData for #struct_name #ty_generics #where_clause {
            type OdType = ::canopen::objectdictionary::ObjectDictionary<#struct_name, #od_size>;

            fn into_od(self) -> Self::OdType {
                unsafe {
                    ::canopen::objectdictionary::ObjectDictionary::new(
                        [#(#indices),*],
                        [#(#subindices),*],
                        [#(#flags),*],
                        [#(::core::mem::offset_of!(#struct_name, #idents)),*],
                        [#(::canopen::meta::metadata(&self.#idents as &dyn ::canopen::objectdictionary::datalink::DataLink)),*],
                        self,
                    )
                }
            }
        }
    })
}

struct Variable {
    index: u16,
    subindex: u8,
    read_only: bool,
    write_only: bool,
    ident: Ident,
}

impl Variable {
    fn flags(&self) -> TokenStream2 {
        let mut flags = quote!(::canopen::objectdictionary::variable::VariableFlags::empty());
        if self.read_only {
            flags = quote!(#flags.set_read_only());
        }
        if self.write_only {
            flags = quote!(#flags.set_write_only());
        }
        flags
    }
}

fn field_to_variables(field: &Field) -> impl Iterator<Item = Result<Variable>> + '_ {
    let ident = field.ident.as_ref().expect("field should have name");
    field.attrs.iter().filter_map(|attr| {
        if attr.path().is_ident("canopen") {
            Some(attr_to_variable(attr, ident.clone()))
        } else {
            None
        }
    })
}

fn attr_to_variable(attr: &Attribute, ident: Ident) -> Result<Variable> {
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
            read_only = true;
        }
        if meta.path.is_ident("write_only") {
            write_only = true;
        }
        Ok(())
    })?;
    let index = index.ok_or(Error::new(attr.span(), "Entry must declare `index`"))?;
    Ok(Variable {
        index,
        subindex,
        read_only,
        write_only,
        ident,
    })
}
