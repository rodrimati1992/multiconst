use alloc::vec::Vec;

use proc_macro2::{Span, TokenStream};

use quote::quote_spanned;

use syn::{punctuated::Punctuated, Data, Error, Field, Fields};

use crate::syntax::{FieldName, Spans};

#[cfg(test)]
mod testing;

mod attribute_parsing;

use attribute_parsing::FieldCfg;

pub(crate) fn derive_macro_impl(ts: TokenStream) -> Result<TokenStream, Error> {
    let input = syn::parse2::<syn::DeriveInput>(ts)?;
    let name = &input.ident;

    let struct_ = if let Data::Struct(x) = &input.data {
        x
    } else {
        return Err(Error::new(
            Span::call_site(),
            "can only derive `FieldType` on structs",
        ));
    };

    let punct: Punctuated<syn::Field, syn::Token!(,)>;
    let fields: Vec<AField<'_>> = {
        let fields = match &struct_.fields {
            Fields::Named(x) => &x.named,
            Fields::Unnamed(x) => &x.unnamed,
            Fields::Unit => {
                punct = Punctuated::default();
                &punct
            }
        };

        fields
            .iter()
            .enumerate()
            .map(|(i, f)| AField::from_field(i, f))
            .collect()
    };

    let cfg = attribute_parsing::parse_attributes(&input, &fields)?;

    let krate = &cfg.krate;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let cont_vis = FTVis::new(&input.vis);

    let out = cfg
        .fields_cfg
        .iter()
        .filter_map(|cfg| {
            let FieldCfg {
                vis_override,
                field,
            } = cfg;
            let AField {
                name: field_name,
                ty,
                ..
            } = field;
            let vis = vis_override.unwrap_or(field.vis);
            let span = field.ty_span;
            let field_name = field_name.tokens(krate);

            if matches!(vis_override, Some(FTVis::Priv))
                || matches!((cont_vis, vis), (FTVis::Pub, FTVis::Priv))
            {
                return None;
            }

            Some(quote_spanned! {span=>
                impl #impl_generics
                    #krate::FieldType<#field_name>
                for #name #ty_generics #where_clause
                {
                    type Type = #ty;
                }
            })
        })
        .collect::<TokenStream>();

    Ok(out)
}

#[derive(Copy, Clone)]
enum FTVis {
    Pub,
    Priv,
}

impl FTVis {
    fn new(vis: &syn::Visibility) -> Self {
        match vis {
            syn::Visibility::Public { .. } => FTVis::Pub,
            _ => FTVis::Priv,
        }
    }
}

struct AField<'a> {
    name: FieldName,
    attrs: &'a [syn::Attribute],
    vis: FTVis,
    ty: &'a syn::Type,
    ty_span: Span,
}

impl<'a> AField<'a> {
    fn from_field(i: usize, f: &'a Field) -> Self {
        let ty_span = syn::spanned::Spanned::span(&f.ty);
        Self {
            name: match &f.ident {
                Some(x) => FieldName::from_ident(x),
                None => FieldName::Numeric(i, Spans::from_one(ty_span)),
            },
            attrs: &f.attrs,
            vis: FTVis::new(&f.vis),
            ty: &f.ty,
            ty_span,
        }
    }
}
