use proc_macro::TokenStream as TS;
use std::collections::HashMap;

use proc_macro2::Literal;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::punctuated::{Pair, Punctuated};
use syn::token::{Comma, PathSep};
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, Field, Fields, FieldsNamed, FieldsUnnamed,
    GenericArgument, Ident, Meta, Path, PathArguments, PathSegment, Type, TypePath, Variant,
};
use syn::{DeriveInput, parse_macro_input, parse_str};

#[proc_macro_derive(ChunkData, attributes(color_type, len, delimiter))]
pub fn derive(input: TS) -> TS {
    let ast: DeriveInput = parse_macro_input!(input);
    eprintln!("{:#?}", ast);

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
        .map(|(i, attrs, fields)| parse_var(&name, attrs, i, fields))
        .collect();

    quote! {
        impl<'a> TryFrom<ChunkProcess<'a>> for #name {
            type Error = PNGError;
            fn try_from(mut value: ChunkProcess<'a>) -> Result<Self, Self::Error> {
                let mut ct = value.ct();
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

fn field_has_attrs(f: &Field) -> bool {
    !f.attrs.is_empty()
}

fn extract_attrs(f: &Field) -> Vec<Attribute> {
    f.attrs.to_vec()
}

fn dump_attr(attr: Attribute) -> (Ident, Literal) {
    let Meta::List(ml) = attr.meta else {
        unreachable!("ChunkData derive macro only takes list type attributes")
    };

    let name = ml.path.segments.into_iter().next().unwrap().ident;
    let val = parse_str::<Literal>(&ml.tokens.into_iter().next().unwrap().to_string()).unwrap();

    (name, val)
}

fn dump_field(f: Field) -> (Ident, Type, Vec<(Ident, Literal)>) {
    let attrs = extract_attrs(&f)
        .into_iter()
        .map(|a| dump_attr(a))
        .collect();

    (f.ident.unwrap(), f.ty, attrs)
}

fn extract_variants(data: Data) -> Vec<(Ident, Vec<(Ident, Literal)>, Fields)> {
    let Data::Enum(de) = data else {
        unreachable!("already on enum data match arm")
    };

    dump_variants(de.variants)
}

fn dump_variants(
    variants: Punctuated<Variant, Comma>,
) -> Vec<(Ident, Vec<(Ident, Literal)>, Fields)> {
    variants
        .into_iter()
        .map(|v| {
            (
                v.ident,
                v.attrs.into_iter().map(|a| dump_attr(a)).collect(),
                v.fields,
            )
        })
        .collect()
}

fn parse_var(ident: &Ident, attrs: Vec<(Ident, Literal)>, name: Ident, fields: Fields) -> TS2 {
    let ct = attrs
        .into_iter()
        .find(|(i, _)| &i.to_string()[..] == "color_type")
        .map(|(_, l)| l);

    match fields {
        Fields::Named(fiena) => {
            let f = spread_named_fields(fiena);
            let parsed = parse_fields(f);

            quote! { #ct => #ident::#name { #(#parsed,)* } }
        }
        Fields::Unnamed(fieun) => {
            let f = spread_unnamed_fields(fieun);
            let parsed = parse_anon_fields(&f);

            quote! { #ct => #ident::#name (#(#parsed,)* ) }
        }
        Fields::Unit => quote! {},
    }
}

fn spread_named_fields(fiena: FieldsNamed) -> Vec<(Ident, Type, Vec<(Ident, Literal)>)> {
    dump_fields(fiena.named)
}

fn spread_unnamed_fields(fieun: FieldsUnnamed) -> Vec<(Type, Vec<(Ident, Literal)>)> {
    dump_anon_fields(fieun.unnamed)
}

fn dump_anon_fields(fields: Punctuated<Field, Comma>) -> Vec<(Type, Vec<(Ident, Literal)>)> {
    fields.into_iter().map(|f| dump_anon_field(f)).collect()
}

fn dump_anon_field(field: Field) -> (Type, Vec<(Ident, Literal)>) {
    let attrs = extract_attrs(&field)
        .into_iter()
        .map(|a| dump_attr(a))
        .collect();

    (field.ty, attrs)
}

fn parse_anon_fields(fields: &[(Type, Vec<(Ident, Literal)>)]) -> Vec<TS2> {
    fields
        .into_iter()
        .map(|f| {
            let f = f.clone();

            parse_field_ty(f.0, f.1)
        })
        .collect()
}

// parse the data
fn extract_fields(data: Data) -> Vec<(Ident, Type, Vec<(Ident, Literal)>)> {
    let Data::Struct(ds) = data else {
        unreachable!("already on struct pipe")
    };
    let Fields::Named(fiena) = ds.fields else {
        unreachable!("struct can only have named fields")
    };

    dump_fields(fiena.named)
}

fn dump_fields(fields: Punctuated<Field, Comma>) -> Vec<(Ident, Type, Vec<(Ident, Literal)>)> {
    fields.into_iter().map(|f| dump_field(f)).collect()
    // .fold((vec![], vec![]), |mut vecs, next| {
    //     vecs.0.push(next.0);
    //     vecs.1.push(next.1);
    //     vecs
    // })
}

fn pathseg_from_ty(ty: Type) -> Option<Pair<PathSegment, PathSep>> {
    if let Type::Path(tp) = ty {
        let TypePath { path, .. } = tp;
        let Path { mut segments, .. } = path;

        return segments.pop();
    }

    None
}

fn parse_field_ty(ty: Type, attrs: Vec<(Ident, Literal)>) -> TS2 {
    let ty = pathseg_from_ty(ty).unwrap().into_value();
    let arg = match &ty.ident.to_string()[..] == "Vec" {
        true => Some({
            let PathArguments::AngleBracketed(abga) = ty.arguments else {
                unreachable!("only handling vecs of u8/16 for now")
            };
            let AngleBracketedGenericArguments { mut args, .. } = abga;
            let GenericArgument::Type(ga) = args.pop().unwrap().into_value() else {
                panic!()
            };
            let Type::Path(tp) = ga else { panic!() };
            let Path { mut segments, .. } = tp.path;

            segments.pop().unwrap().into_value()
        }),
        false => None,
    };

    let delim = attrs
        .into_iter()
        .find(|(i, _)| &i.to_string()[..] == "delimiter")
        .map(|(_, l)| l);
    let delim = if delim.is_some() {
        quote! { Some(#delim) }
    } else {
        quote! {None  }
    };

    match &ty.ident.to_string()[..] {
        "u8" => quote! {*value.next().unwrap() },
        "u16" => quote! { stream_octets_to_u16(&mut value) },
        "u32" => quote! { stream_octets_to_u32(&mut value) },
        "u64" => quote! { stream_octets_to_u64(&mut value) },
        "Vec" => match &arg.unwrap().ident.to_string()[..] {
            "u8" => quote! { stream_vecu8(&mut value, #delim)},
            "u16" => quote! { stream_vecu16(&mut value, #delim)},
            v => panic!("unprepared to handle vec of {}, expected u8/16", v),
        },
        ty => panic!("i was not prepared to handle the {} type", ty),
    }
}

fn parse_field(ident: Ident, ty: Type, attrs: Vec<(Ident, Literal)>) -> TS2 {
    let value = parse_field_ty(ty, attrs);
    quote! { #ident: #value }
}

fn parse_fields(fields: Vec<(Ident, Type, Vec<(Ident, Literal)>)>) -> Vec<TS2> {
    fields
        .into_iter()
        .map(|(i, t, a)| parse_field(i, t, a))
        .collect()
}

type KnownField = (Ident, Type, MacroAttr);
type MacroAttr = Vec<(Ident, Literal)>;
