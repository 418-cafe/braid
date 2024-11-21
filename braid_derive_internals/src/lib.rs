use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, LitStr, Token};

use braid_fields::commit::CommitField as CommitFieldExt;

pub fn derive_from_commit_data(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,

        _ => panic!("Only structs with named fields are supported"),
    };

    let braid_path = match proc_macro_crate::crate_name("braid").expect("braid must be a dependency of the project") {
        proc_macro_crate::FoundCrate::Itself => quote!(crate),
        proc_macro_crate::FoundCrate::Name(name) => quote!(::#name),
    };

    let ident = &input.ident;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut sets = vec![];
    let mut required = HashSet::new();

    for field in fields {
        let mut skipped = false;

        let ident = match field.ident.as_ref() {
            Some(ident) => ident,
            None => return Err(syn::Error::new_spanned(&field, "Unnamed fields are not supported")),
        };

        let mut mapped_name = None;

        for attr in field.attrs.iter().filter(|attr| attr.path().is_ident("braid")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("map_to") {
                    meta.input.parse::<Token![=]>()?;
                    let val: LitStr = meta.input.parse()?;

                    if let Some(existing) = mapped_name.replace(val.value()) {
                        return Err(syn::Error::new_spanned(
                            val,
                            format!("Duplicate map_to attribute, already mapped to '{}'", existing),
                        ));
                    }
                } else if meta.path.is_ident("skip") {
                    skipped = true;
                } else {
                    return Err(syn::Error::new_spanned(meta.path, "Unknown attribute"));
                }

                Ok(())
            })?;
        }

        if skipped {
            if mapped_name.is_some() {
                return Err(syn::Error::new_spanned(
                    field,
                    "Cannot have both map_to and skip attributes",
                ));
            }
            
            sets.push(quote! {
                let #ident = Default::default();
            });
        }
        
        let field = CommitField::try_from_str(mapped_name.as_ref().unwrap_or(&ident.to_string().trim_start_matches("r#").to_owned()))
            .ok_or_else(|| syn::Error::new_spanned(&field, "Unknown field name"))?;

        if !required.insert(field) {
            unreachable!("Duplicate field");
        }

        let field = field.tokenize(&braid_path);
        sets.push(quote! {
            let #ident = data.try_get(#field)?;
        });
    }

    let n = required.len();
    let s = {
        let mut s = "SELECT ".len();
        s += required.iter().map(|field| field.0.as_quoted_str().len()).sum::<usize>();
        s += " FROM commit where \"oid\" = $1".len();
        s += ", ".len() * (n - 1);
        s
    };

    let fields = required.into_iter().map(|field| field.tokenize(&braid_path));

    let impl_block = quote! {
        impl #impl_generics #braid_path::FromCommitData<#n, #s> for #ident #ty_generics #where_clause {
            const FIELDS: [#braid_path::CommitField; #n] = [#(#fields),*];

            /*fn from_commit_data<D: #braid_path::CommitData>(data: &D) -> #braid_path::Result<Self> {
                #(#sets)*

                Ok(Self {
                    #(#sets,)*
                })
            }*/
        }
    };

    println!("{}", impl_block);

    Ok(impl_block)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct CommitField(CommitFieldExt);

impl CommitField {
    fn try_from_str(name: &str) -> Option<Self> {
        CommitFieldExt::try_from_str(name).map(Self)
    }

    fn tokenize(&self, path: &TokenStream) -> proc_macro2::TokenStream {
        match self.0 {
            CommitFieldExt::Oid => quote! { #path::CommitField::Oid },
            CommitFieldExt::Parent => quote! { #path::CommitField::Parent },
            CommitFieldExt::MergeParent => quote! { #path::CommitField::MergeParent },
        }
    }
}

const fn as_quoted_bytes<const N: usize>(src: &[u8]) -> [u8; N] {
    const QUOTE: u8 = b'"';
    let mut bytes = [QUOTE; N];

    let mut i = 1;
    while i < N -1 {
        bytes[i] = src[i - 1];
        i += 1;
    }

    bytes
}