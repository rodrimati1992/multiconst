use used_proc_macro::{Punct, Spacing, TokenStream, TokenTree};

use crate::{
    syntax::{Attributes, Crate, FieldName},
    utils::{TokenStreamExt, WithSpan},
    Error,
};

pub(crate) fn field_macro_impl(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let input = &mut crate::parsing::ParseBuffer::new(input);
    let crate_kw = Crate::parse(input).unwrap();
    (|| -> Result<TokenStream, Error> {
        let mut out = TokenStream::new();
        let field_ident = FieldName::parse(input)?;
        input.assert_empty()?;
        field_ident.to_token_stream(&crate_kw, &mut out);

        Ok(out)
    })()
    .map_err(|e| e.to_compile_error(&crate_kw))
}

pub(crate) fn field_name_aliases_macro_impl(
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let input = &mut crate::parsing::ParseBuffer::new(input);
    let crate_kw = Crate::parse(input).unwrap();

    let allows = "#[allow(non_camel_case_types)]"
        .parse::<TokenStream>()
        .unwrap();

    (|| -> Result<TokenStream, Error> {
        let mut out = TokenStream::new();

        while !input.is_empty() {
            let attrs = Attributes::parse(input);
            let vis = input.parse_vis();
            let ident = input.parse_ident()?;
            let span = ident.span();

            out.extend(allows.clone());
            out.extend(attrs.attrs);
            out.extend(vis);
            out.append_keyword("type", span);

            if matches!(input.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '=') {
                let equals = input.parse_punct('=')?;
                let eq_span = equals.span();

                out.append_one(ident);
                out.append_one(equals);

                if input.is_empty() {
                    return Err(Error::with_span(eq_span, "expected field name after this"));
                }

                FieldName::parse(input)?.to_token_stream(&crate_kw, &mut out);
            } else {
                let field_ident = FieldName::from_ident(&ident);

                out.append_one(ident);
                out.append_one(Punct::new('=', Spacing::Alone).with_span(span));
                field_ident.to_token_stream(&crate_kw, &mut out);
            }
            out.append_one(Punct::new(';', Spacing::Alone).with_span(span));

            input.parse_opt_punct(',')?;
        }

        Ok(out)
    })()
    .map_err(|e| e.to_compile_error(&crate_kw))
}
