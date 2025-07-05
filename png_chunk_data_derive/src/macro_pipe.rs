use proc_macro2::Literal;
use syn::{
    Attribute, Data, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Meta, Path, PathSegment,
    Type, TypePath, Variant,
};

pub(crate) struct StructField {
    ident: Ident,
    ty: Type,
    attrs: Vec<MacroAttr>,
}

impl From<(Ident, Type)> for StructField {
    fn from(value: (Ident, Type)) -> Self {
        Self {
            ident: value.0,
            ty: value.1,
            attrs: vec![],
        }
    }
}

impl StructField {
    pub(crate) fn into_ident(self) -> Ident {
        self.ident
    }

    pub(crate) fn into_ty(self) -> Type {
        self.ty
    }

    pub(crate) fn ident(&self) -> &Ident {
        &self.ident
    }

    pub(crate) fn ty(&self) -> &Type {
        &self.ty
    }
}

pub(crate) struct AnonField {
    ty: Type,
    attrs: Vec<MacroAttr>,
}

pub(crate) struct MacroAttr {
    ident: Ident,
    val: Literal,
}

pub(crate) struct MacroStruct {
    fields: Vec<StructField>,
}

impl From<Vec<(Ident, Type)>> for MacroStruct {
    fn from(value: Vec<(Ident, Type)>) -> Self {
        Self {
            fields: value
                .into_iter()
                .map(|(i, t)| (i, t).into())
                .collect::<Vec<StructField>>(),
        }
    }
}

pub(crate) struct MacroEnum {
    variants: Vec<MacroVariant>,
}

pub(crate) enum MacroVariant {
    Knwon { var: MacroKnownVariant },
    Anon { var: MacroAnonVariant },
}

pub(crate) struct MacroKnownVariant {
    ident: Ident,
    fields: Vec<StructField>,
}

pub(crate) struct MacroAnonVariant {
    ident: Ident,
    fields: Vec<AnonField>,
    attrs: Vec<MacroAttr>,
}
