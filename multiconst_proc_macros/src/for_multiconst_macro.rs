use used_proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream};

use alloc::{string::String, vec::Vec};

use crate::{
    parsing::ParseStream,
    pattern::{BindingAndType, Pattern},
    pattern_processing::{CheckedLocal, ExtractConstCtx, FieldType, WholeFieldPat},
    syntax::{self, tokenize_delim, tokenize_iter_delim, Attributes, Crate, Spans},
    type_::Type,
    utils::{TokenStreamExt, TokenTreeExt, WithSpan},
    Error,
};

#[cfg(test)]
mod tests;

pub(crate) fn macro_impl(ts: TokenStream) -> Result<TokenStream, TokenStream> {
    // #[cfg(feature = "__dbg")]
    // std::println!("\n\n{:#?}\n\n", ts);

    let input = &mut crate::parsing::ParseBuffer::new(ts);
    let crate_kw = Crate::parse(input).unwrap();

    let ret = parse_all_constants(&crate_kw, input)
        .map_err(|e| Error::to_compile_error(&e, &crate_kw))?;

    // #[cfg(feature = "__dbg")]
    // std::println!("\n\n{}\n\n", ret);

    Ok(ret)
}

pub(crate) fn parse_all_constants(
    crate_kw: &Crate,
    input: ParseStream<'_>,
) -> Result<TokenStream, Error> {
    let mut out = TokenStream::new();

    while !input.is_empty() {
        parse_one_constant(crate_kw, input, &mut out)?;
    }

    // #[cfg(feature = "__dbg")]
    // ::std::println!("{}", out);

    Ok(out)
}

fn parse_one_constant(
    crate_kw: &Crate,
    input: ParseStream<'_>,
    ts: &mut TokenStream,
) -> Result<(), Error> {
    let outer_attrs = Attributes::parse(input);
    let vis = input.parse_vis();
    let const_token = input.parse_keyword("const")?;
    let pattern = Pattern::parse(input)?;
    let _colon = input.parse_punct(':')?;
    let type_ = Type::parse(input)?;
    let type_ = crate::pattern_processing::real_type_from(&pattern, type_)?;

    let const_span = const_token.span();
    let equals = input.parse_punct('=')?;

    let expr = input.tokens_until(|tt| tt.is_punct(';'));
    if expr.is_empty() {
        return Err(Error::with_span(
            equals.span(),
            "expected expression after this",
        ));
    }
    input.parse_punct(';')?;

    let const_prefix: String;
    let const_prefix: &str = match crate::pattern_processing::find_first_const_ident(&pattern) {
        Some(ident) => {
            const_prefix = alloc::format!("__PRIV_MULTICONST__{}", ident);
            &const_prefix
        }
        None => "_",
    };

    let mut bats: Vec<BindingAndType> = Vec::new();
    let mut tuple_rem_lens: Vec<TokenStream> = Vec::new();
    let mut checked_locals: Vec<CheckedLocal> = Vec::new();
    let tuple_rem_pat_const = Ident::new(&alloc::format!("{}_REM_LENS", const_prefix), const_span);

    crate::pattern_processing::extract_const_names_tys(
        &pattern,
        FieldType::Direct(&type_),
        WholeFieldPat::No,
        &mut ExtractConstCtx {
            bats: &mut bats,
            tuple_rem_lens: &mut tuple_rem_lens,
            tuple_rem_pat_const: &tuple_rem_pat_const,
            checked_locals: &mut checked_locals,
            crate_kw,
        },
    )?;

    {
        let no_contants = bats.is_empty();

        let priv_const_name =
            Ident::new(const_prefix, crate_kw.ident.span().located_at(const_span));

        // hack to assert that vis is valid syntax when it's not otherwise used
        if no_contants {
            ts.extend(vis.clone());
        }

        ///////////////////

        if !tuple_rem_lens.is_empty() {
            ts.append_keyword("const", const_span);
            ts.append_one(tuple_rem_pat_const);
            ts.append_one(Punct::new(':', Spacing::Alone).with_span(const_span));
            ts.append_one(Punct::new('&', Spacing::Alone).with_span(const_span));

            let mut usize_ = TokenStream::new();
            crate_kw.item_to_ts("usize", Spans::from_one(const_span), &mut usize_);
            ts.append_one(Group::new(Delimiter::Bracket, usize_));

            ts.append_one(Punct::new('=', Spacing::Alone).with_span(const_span));
            ts.append_one(Punct::new('&', Spacing::Alone).with_span(const_span));

            let mut lens_ts = TokenStream::new();
            for len in tuple_rem_lens {
                lens_ts.extend(len);
                lens_ts.append_one(Punct::new(',', Spacing::Alone).with_span(const_span));
            }
            ts.append_one(Group::new(Delimiter::Bracket, lens_ts));

            ts.append_one(Punct::new(';', Spacing::Alone).with_span(const_span));
        }

        ///////////////////

        ts.append_keyword("const", const_span);
        ts.append_one(priv_const_name.clone());
        ts.append_one(Punct::new(':', Spacing::Alone).with_span(const_span));
        tokenize_iter_delim(Delimiter::Parenthesis, const_span, &bats, ts, |ts, bat| {
            ts.extend(bat.type_.ty.clone());
            syntax::tokenize_comma(const_span, ts);
        });
        ts.append_one(Punct::new('=', Spacing::Alone));

        tokenize_delim(Delimiter::Brace, const_span, ts, |ts| {
            ts.append_keyword("let", const_span);
            pattern.to_token_stream(ts);
            ts.append_one(Punct::new(':', Spacing::Alone).with_span(const_span));
            ts.extend(type_.to_tokens());
            ts.append_one(Punct::new('=', Spacing::Alone).with_span(const_span));
            ts.extend(expr);
            ts.append_one(Punct::new(';', Spacing::Alone).with_span(const_span));

            for CheckedLocal {
                binding,
                type_: btype,
            } in checked_locals
            {
                let bspan = binding.span();
                ts.append_keyword("let", bspan);
                ts.append_one(Ident::new("_", bspan));
                ts.append_one(Punct::new(':', Spacing::Alone).with_span(bspan));
                ts.extend(btype.ty);
                ts.append_one(Punct::new('=', Spacing::Alone).with_span(bspan));
                ts.append_one(binding);
                ts.append_one(Punct::new(';', Spacing::Alone).with_span(bspan));
            }

            tokenize_iter_delim(Delimiter::Parenthesis, const_span, &bats, ts, |ts, bat| {
                ts.append_one(bat.local.clone());
                syntax::tokenize_comma(const_span, ts);
            });
        });

        ts.append_one(Punct::new(';', Spacing::Alone).with_span(const_span));

        ///////////////////

        for (i, bat) in bats.into_iter().enumerate() {
            let nconst_span = bat.constant.span();

            ts.extend(bat.attrs.attrs);
            ts.extend(outer_attrs.attrs.clone());
            ts.extend(vis.clone());
            ts.append_keyword("const", nconst_span);
            ts.append_one(bat.constant);
            ts.append_one(Punct::new(':', Spacing::Alone).with_span(nconst_span));
            ts.extend(bat.type_.ty.clone());
            ts.append_one(Punct::new('=', Spacing::Alone).with_span(nconst_span));
            ts.append_one(priv_const_name.clone());
            ts.append_one(Punct::new('.', Spacing::Alone).with_span(nconst_span));
            ts.append_one(Literal::usize_unsuffixed(i).with_span(nconst_span));
            ts.append_one(Punct::new(';', Spacing::Alone).with_span(nconst_span));
        }
    }

    Ok(())
}
