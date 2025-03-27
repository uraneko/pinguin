use proc_macro::TokenStream as TS;
use std::collections::HashMap;

use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::punctuated::{Pair, Punctuated};
use syn::token::{Comma, PathSep};
use syn::{parse_macro_input, parse_str, DeriveInput};
use syn::{
    Data, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Path, PathSegment, Type, TypePath,
    Variant,
};

#[proc_macro_derive(ChunkData)]
pub fn derive(input: TS) -> TS {
    let ast: DeriveInput = parse_macro_input!(input);

    let name = ast.ident;
    match resolve_data_kind(&ast.data) {
        "struct" => struct_derive(name, ast.data),
        "enum" => enum_derive(name, ast.data),
        "union" => panic!("im sorry, I thought union was an arcane type"),
        _ => panic!("no such data variant"),
    }
}

fn resolve_data_kind(data: &Data) -> &str {
    match data {
        Data::Struct(_) => "struct",
        Data::Enum(_) => "enum",
        Data::Union(_) => "union",
    }
}

fn struct_derive(name: Ident, data: Data) -> TS {
    let fields = extract_fields(data);
    let fields = parse_fields(fields);

    quote! {
        impl<'a> TryFrom<ChunkProcess<'a>> for #name {
            type Error = PNGError;
            fn try_from(mut value: ChunkProcess<'a>) -> Result<Self, Self::Error> {
                let mut value = value.data().iter();
                Ok(Self {
                    #(#fields,)*
                })
            }
        }
    }
    .into()
}

fn enum_derive(name: Ident, data: Data) -> TS {
    let vars = extract_variants(data);
    let vars: Vec<TS2> = vars
        .into_iter()
        .map(|(i, fields)| parse_var(&name, i, fields))
        .collect();

    quote! {
        impl<'a> TryFrom<ChunkProcess<'a>> for #name {
            type Error = PNGError;
            fn try_from(mut value: ChunkProcess<'a>) -> Result<Self, Self::Error> {
                let mut ct = #name::var_from_ct(value.ct());
                let mut value = value.data().iter();
                Ok(match ct {
                    #(#vars,)*
                    _ => panic!("check your impl of EnumChunk::var_from_ct associated function")
                })
            }
        }
    }
    .into()
}

fn extract_variants(data: Data) -> Vec<(Ident, Fields)> {
    let Data::Enum(de) = data else {
        unreachable!("already on enum data match arm")
    };

    dump_variants(de.variants)
}

fn dump_variants(variants: Punctuated<Variant, Comma>) -> Vec<(Ident, Fields)> {
    variants.into_iter().map(|v| (v.ident, v.fields)).collect()
}

fn parse_var(ident: &Ident, name: Ident, fields: Fields) -> TS2 {
    let strname = name.to_string();
    match fields {
        Fields::Named(fiena) => {
            let f = spread_named_fields(fiena);
            let parsed = parse_fields(f);

            quote! { #strname => #ident::#name { #(#parsed,)* } }
        }
        Fields::Unnamed(fieun) => {
            let f = spread_unnamed_fields(fieun);
            let parsed = parse_anon_fields(f);

            quote! { #strname => #ident::#name (#(#parsed,)* ) }
        }
        Fields::Unit => quote! {},
    }
}

fn spread_named_fields(fiena: FieldsNamed) -> Vec<(Ident, Type)> {
    dump_fields(fiena.named)
}

fn spread_unnamed_fields(fieun: FieldsUnnamed) -> Vec<Type> {
    dump_anon_fields(fieun.unnamed)
}

fn dump_anon_fields(fields: Punctuated<Field, Comma>) -> Vec<Type> {
    fields.into_iter().map(|f| f.ty).collect()
}

fn parse_anon_fields(fields: Vec<Type>) -> Vec<TS2> {
    fields.into_iter().map(|f| parse_field_value(f)).collect()
}

// parse the data
fn extract_fields(data: Data) -> Vec<(Ident, Type)> {
    let Data::Struct(ds) = data else {
        unreachable!("already on struct pipe")
    };
    let Fields::Named(fiena) = ds.fields else {
        unreachable!("struct can only have named fields")
    };

    dump_fields(fiena.named)
}

fn dump_fields(fields: Punctuated<Field, Comma>) -> Vec<(Ident, Type)> {
    fields
        .into_iter()
        .map(|f| (f.ident.unwrap(), f.ty))
        .collect()
    // .fold((vec![], vec![]), |mut vecs, next| {
    //     vecs.0.push(next.0);
    //     vecs.1.push(next.1);
    //     vecs
    // })
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
        "u16" => quote! { stream_octets_to_u16(&mut value) },
        "u32" => quote! { stream_octets_to_u32(&mut value) },
        "u64" => quote! { stream_octets_to_u64(&mut value) },
        ty => panic!("i was not prepared to handle the {} type", ty),
    }
}

fn parse_field(ident: Ident, value: Type) -> TS2 {
    let value = parse_field_value(value);
    quote! { #ident: #value }
}

fn parse_fields(fields: Vec<(Ident, Type)>) -> Vec<TS2> {
    fields.into_iter().map(|(i, t)| parse_field(i, t)).collect()
}
