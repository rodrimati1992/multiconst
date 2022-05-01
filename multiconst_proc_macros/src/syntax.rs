//! Syntax-related types and functions, no extension traits.

use used_proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::{
    parsing::ParseStream,
    utils::{IsIdent, TokenStreamExt, WithSpan},
    Error,
};

///////////////////////////////////////////////////////////////////////////////

pub(crate) fn tokenize_comma(span: Span, ts: &mut TokenStream) {
    ts.append_one(Punct::new(',', Spacing::Alone).with_span(span));
}

pub(crate) fn tokenize_iter_delim<I, F>(
    delimiter: Delimiter,
    delim_span: Span,
    iter: I,
    ts: &mut TokenStream,
    mut closure: F,
) where
    I: IntoIterator,
    F: FnMut(&mut TokenStream, I::Item),
{
    let mut inner_ts = TokenStream::new();
    for x in iter {
        closure(&mut inner_ts, x);
    }
    let mut group = Group::new(delimiter, inner_ts);
    group.set_span(delim_span);
    ts.append_one(group)
}

pub(crate) fn tokenize_delim<F>(
    delimiter: Delimiter,
    delim_span: Span,
    ts: &mut TokenStream,
    closure: F,
) where
    F: FnOnce(&mut TokenStream),
{
    let mut inner_ts = TokenStream::new();
    closure(&mut inner_ts);
    let mut group = Group::new(delimiter, inner_ts);
    group.set_span(delim_span);
    ts.append_one(group)
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub(crate) struct Spans {
    pub(crate) start: Span,
    pub(crate) end: Span,
}

impl Spans {
    pub(crate) fn from_one(span: Span) -> Self {
        Self {
            start: span,
            end: span,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) fn tokenize_seq_length_assoc_const(
    crate_kw: &Crate,
    spans: Spans,
    type_: TokenStream,
) -> TokenStream {
    let mut ts = TokenStream::new();

    ts.append_one(Punct::new('<', Spacing::Joint).with_span(spans.start));
    ts.append_one(Group::new(Delimiter::Parenthesis, type_).with_span(spans.start));
    ts.append_one(Ident::new("as", spans.start));
    crate_kw.item_to_ts("SeqLength", spans, &mut ts);
    ts.append_one(Punct::new('>', Spacing::Alone).with_span(spans.end));
    ts.append_one(Punct::new(':', Spacing::Joint).with_span(spans.end));
    ts.append_one(Punct::new(':', Spacing::Alone).with_span(spans.end));
    ts.append_one(Ident::new("LENGTH", spans.end));

    ts
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) struct Crate {
    pub(crate) ident: Ident,
}

impl Crate {
    /// outputs the path to the item at `multiconst::__::{item}`
    pub(crate) fn item_to_ts(&self, item: &str, spans: Spans, ts: &mut TokenStream) {
        ts.append_array([
            TokenTree::Ident(self.ident.clone().with_span(spans.start)),
            Punct::new(':', Spacing::Joint)
                .with_span(spans.start)
                .into(),
            Punct::new(':', Spacing::Alone)
                .with_span(spans.start)
                .into(),
            Ident::new("__", spans.start).into(),
            Punct::new(':', Spacing::Joint)
                .with_span(spans.start)
                .into(),
            Punct::new(':', Spacing::Alone)
                .with_span(spans.start)
                .into(),
            Ident::new(item, spans.end).into(),
        ]);
    }

    pub(crate) fn parse(input: ParseStream<'_>) -> Result<Self, Error> {
        match input.parse_ident() {
            Ok(ident) if ident.which_ident_in(&["$crate", "crate"]).is_some() => {
                Ok(Crate { ident })
            }
            Ok(x) => Err(Error::new(
                Spans::from_one(x.span()),
                "Expected `$crate` or `crate`",
            )),
            Err(e) => Err(e),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "__dbg", derive(Debug))]
#[derive(Clone)]
pub(crate) struct OpaqueType {
    pub(crate) spans: Spans,
    pub(crate) ty: TokenStream,
}
