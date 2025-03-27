use proc_macro::TokenStream as TS;
use std::collections::HashMap;

use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::punctuated::{Pair, Punctuated};
use syn::token::{Comma, PathSep};
use syn::{Data, Field, Fields, Ident, Path, PathSegment, Type, TypePath};
use syn::{DeriveInput, parse_macro_input, parse_str};

#[proc_macro_derive(ChunkData)]
pub fn derive(input: TS) -> TS {
    let ast: DeriveInput = parse_macro_input!(input);
    eprintln!("{:#?}", ast);
    let name = ast.ident;
    let (idents, tys) = extract_fields(ast.data);
    let fields = parse_fields(idents, tys);

    quote! {
        impl<'a> TryFrom<&mut Iter<'a, u8>> for #name {
            type Error = PNGError;
            fn try_from(value: &mut Iter<u8>) -> Result<Self, Self::Error> {
                Ok(Self {
                    #(#fields,)*
                })
            }
        }
    }
    .into()
}

// parse the data
fn extract_fields(data: Data) -> (Vec<Ident>, Vec<Type>) {
    if let Data::Struct(ds) = data {
        if let Fields::Named(fiena) = ds.fields {
            return dump_fields(fiena.named);
        }
    }

    (vec![], vec![])
}

fn dump_fields(fields: Punctuated<Field, Comma>) -> (Vec<Ident>, Vec<Type>) {
    fields
        .into_iter()
        .map(|f| (f.ident.unwrap(), f.ty))
        .fold((vec![], vec![]), |mut vecs, next| {
            vecs.0.push(next.0);
            vecs.1.push(next.1);
            vecs
        })
}

// modify the data
fn pathseg_from_ty(ty: Type) -> Option<Pair<PathSegment, PathSep>> {
    if let Type::Path(tp) = ty {
        let TypePath { path, .. } = tp else {
            unreachable!("irrefutable")
        };
        let Path { mut segments, .. } = path else {
            unreachable!("irrefutable")
        };
        return segments.pop();
    }

    None
}

fn parse_field_value(ty: Type) -> TS2 {
    let ty = pathseg_from_ty(ty).unwrap().into_value();
    match &ty.ident.to_string()[..] {
        "u8" => quote! {*value.next().unwrap() },
        "u16" => quote! { stream_octets_to_u16(value) },
        "u32" => quote! { stream_octets_to_u32(value) },
        "u64" => quote! { stream_octets_to_u64(value) },
        ty => panic!("i was not prepared to handle the {} type", ty),
    }
}

fn parse_field(ident: Ident, value: TS2) -> TS2 {
    quote! { #ident: #value }
}

fn parse_fields(idents: Vec<Ident>, tys: Vec<Type>) -> Vec<TS2> {
    std::iter::zip(idents, tys)
        .map(|(i, t)| parse_field(i, parse_field_value(t)))
        .collect()
}
