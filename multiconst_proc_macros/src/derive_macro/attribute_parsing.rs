use super::{AField, FTVis};

use alloc::vec::Vec;

use proc_macro2::TokenStream;

use syn::{
    parse::{ParseBuffer, ParseStream, Parser},
    Attribute, Error, Token,
};

pub(super) struct Config<'a> {
    pub(super) krate: syn::Path,
    pub(super) vis_override: Option<FTVis>,
    pub(super) fields_cfg: Vec<FieldCfg<'a>>,
}

pub(super) struct FieldCfg<'a> {
    pub(super) vis_override: Option<FTVis>,
    pub(super) field: &'a AField<'a>,
}

enum ParseCtx<'a, 'b> {
    Container(&'a mut Config<'b>),
    Field(&'a mut FieldCfg<'b>),
}

pub(super) fn parse_attributes<'a>(
    input: &'a syn::DeriveInput,
    fields: &'a [AField<'a>],
) -> Result<Config<'a>, Error> {
    let mut cfg = Config {
        krate: syn::parse_quote!(multiconst),
        vis_override: None,
        fields_cfg: Vec::new(),
    };

    let mut errs: Result<(), Error> = Ok(());

    parse_attributes_outer(&mut errs, &input.attrs, &mut ParseCtx::Container(&mut cfg));

    for field in fields.iter() {
        let mut field_cfg = FieldCfg {
            vis_override: cfg.vis_override,
            field,
        };

        parse_attributes_outer(&mut errs, field.attrs, &mut ParseCtx::Field(&mut field_cfg));

        cfg.fields_cfg.push(field_cfg);
    }

    errs.map(|_| cfg)
}

fn parse_attributes_outer(
    errs: &mut Result<(), Error>,
    attributes: &[Attribute],
    pctx: &mut ParseCtx<'_, '_>,
) {
    let mut closure = move |input: &'_ ParseBuffer<'_>| parse_attributes_inner(pctx, input);

    for attr in attributes {
        if attr.path.is_ident("field_type") {
            let res = if attr.tokens.is_empty() {
                Parser::parse2(&mut closure, TokenStream::new())
            } else {
                attr.parse_args_with(&mut closure)
            };

            if let Err(e) = res {
                match *errs {
                    ref mut x @ Ok(()) => *x = Err(e),
                    Err(ref mut x) => *x = e,
                }
            }
        }
    }
}

fn parse_attributes_inner(
    pctx: &mut ParseCtx<'_, '_>,
    input: ParseStream<'_>,
) -> Result<(), Error> {
    let lookahead = Lookhead {
        input,
        lookahead: input.lookahead1(),
    };

    let opt_crate = match pctx {
        ParseCtx::Container { .. } => lookahead.peek_parse(Token!(crate))?,
        _ => None,
    };

    if let Some(_) = opt_crate {
        input.parse::<Token!(=)>()?;
        let krate = input.parse::<syn::Path>()?;
        match pctx {
            ParseCtx::Container(cfg) => {
                cfg.krate = krate;
            }
            ParseCtx::Field { .. } => {
                unreachable!();
            }
        }
    } else if let Some(_) = lookahead.peek_parse(Token!(pub))? {
        let vo = match pctx {
            ParseCtx::Container(x) => &mut x.vis_override,
            ParseCtx::Field(x) => &mut x.vis_override,
        };
        *vo = Some(FTVis::Pub);
    } else if let Some(_) = lookahead.peek_parse(Token!(priv))? {
        let vo = match pctx {
            ParseCtx::Container(x) => &mut x.vis_override,
            ParseCtx::Field(x) => &mut x.vis_override,
        };
        *vo = Some(FTVis::Priv);
    } else {
        return Err(lookahead.error());
    }

    Ok(())
}

struct Lookhead<'a> {
    input: ParseStream<'a>,
    lookahead: syn::parse::Lookahead1<'a>,
}

impl Lookhead<'_> {
    fn peek_parse<F, X, P>(&self, f: F) -> Result<Option<P>, syn::Error>
    where
        F: FnOnce(X) -> P + syn::parse::Peek,
        P: syn::parse::Parse,
    {
        if self.lookahead.peek(f) {
            self.input.parse::<P>().map(Some)
        } else {
            Ok(None)
        }
    }

    fn error(self) -> syn::Error {
        self.lookahead.error()
    }
}
