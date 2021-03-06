//! Syntax-related types and functions, no extension traits.

use used_proc_macro::{
    Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree,
};

use alloc::{rc::Rc, string::ToString};

use crate::{
    parsing::ParseStream,
    utils::{self, IsIdent, TokenStreamExt, TokenTreeExt, WithSpan},
    Error,
};

#[cfg(test)]
mod testing;

///////////////////////////////////////////////////////////////////////////////

enum PathToken {
    Type,
    Colon,
    Other,
}

fn is_path_token(tt: Option<&TokenTree>) -> Option<PathToken> {
    match tt {
        Some(TokenTree::Ident(_)) => Some(PathToken::Other),
        Some(TokenTree::Punct(p)) => match p.as_char() {
            '<' => Some(PathToken::Type),
            ':' => Some(PathToken::Colon),
            _ => None,
        },
        _ => None,
    }
}

/// Loose path parsing
#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct Path {
    pub(crate) tokens: TokenStream,
    pub(crate) spans: Spans,
}

impl Path {
    pub(crate) fn parse(input: ParseStream<'_>) -> Result<Path, Error> {
        let mut out = TokenStream::new();
        let start = input.span();
        let mut prev_pt = PathToken::Other;

        while let Some(pt) = is_path_token(input.peek()) {
            match pt {
                PathToken::Type => {
                    if out.is_empty() {
                        return Err(Error::with_span(input.span(), "path can't start with `<`"));
                    }

                    let ot = input.parse_opaque_type_with(|_| true)?;

                    if !matches!(prev_pt, PathToken::Colon) {
                        let c2span = ot.spans.start;
                        out.append_one(Punct::new(':', Spacing::Joint).with_span(c2span));
                        out.append_one(Punct::new(':', Spacing::Alone).with_span(c2span));
                    }
                    out.extend(ot.ty)
                }
                PathToken::Colon | PathToken::Other => {
                    out.extend(input.next());
                }
            }
            prev_pt = pt;
        }

        if out.is_empty() {
            return Err(input.error("expected path after this token"));
        }
        let spans = Spans {
            start,
            end: input.span(),
        };

        Ok(Path { tokens: out, spans })
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) enum FieldName {
    Numeric(usize, Spans),
    Alphabetic(Rc<str>, Spans),
    /// A numeric identifier, determined by a constant in the expanded code.
    NumericConst(TokenStream, Spans),
}

impl FieldName {
    #[cfg(feature = "derive")]
    pub(crate) fn tokens(&self, crate_path: &syn::Path) -> TokenStream {
        use quote::ToTokens;

        let mut ts = TokenStream::new();
        self.to_token_stream_inner(&mut ts, &|item_name, spans, ts| {
            let item = Ident::new(item_name, spans.end);
            ts.extend(quote::quote_spanned!(spans.start=> #crate_path::__::));
            item.to_tokens(ts);
        });
        ts
    }

    fn to_token_stream_inner(
        &self,
        ts: &mut TokenStream,
        item_to_ts: &dyn Fn(&str, Spans, &mut TokenStream),
    ) {
        match *self {
            FieldName::Numeric(n, spans) => {
                item_to_ts("Usize", spans, ts);

                ts.append_one(Punct::new('<', Spacing::Joint).with_span(spans.start));
                ts.append_one(Literal::usize_unsuffixed(n).with_span(spans.end));
                ts.append_one(Punct::new('>', Spacing::Joint).with_span(spans.end));
            }
            FieldName::Alphabetic(ref str, spans) => {
                let mut chars = str.chars();
                let span = spans.start;

                item_to_ts("TIdent", spans, ts);

                ts.append_one(Punct::new('<', Spacing::Joint).with_span(span));

                tokenize_delim(Delimiter::Parenthesis, span, ts, |ts| {
                    while !chars.as_str().is_empty() {
                        item_to_ts("TChars", spans, ts);

                        ts.append_one(Punct::new('<', Spacing::Joint).with_span(span));

                        chars
                            .by_ref()
                            .chain(core::iter::repeat(' '))
                            .take(8)
                            .for_each(|c| {
                                ts.append_one(Literal::character(c).with_span(span));
                                ts.append_one(Punct::new(',', Spacing::Joint).with_span(span));
                            });

                        ts.append_one(Punct::new('>', Spacing::Joint).with_span(span));
                        ts.append_one(Punct::new(',', Spacing::Joint).with_span(span));
                    }
                });

                ts.append_one(Punct::new('>', Spacing::Joint).with_span(span));
            }
            FieldName::NumericConst(ref x, spans) => {
                item_to_ts("Usize", spans, ts);

                ts.append_one(Punct::new('<', Spacing::Joint).with_span(spans.start));
                ts.append_one(Group::new(Delimiter::Brace, x.clone()));
                ts.append_one(Punct::new('>', Spacing::Joint).with_span(spans.end));
            }
        }
    }
    pub(crate) fn to_token_stream(&self, crate_kw: &Crate, ts: &mut TokenStream) {
        self.to_token_stream_inner(ts, &|item_name, spans, ts| {
            crate_kw.item_to_ts(item_name, spans, ts)
        })
    }

    pub(crate) fn parse(input: ParseStream<'_>) -> Result<Self, Error> {
        const EXPECTED: &str = "expected either an untyped numeric literal or an identifier";

        match input.next() {
            Some(TokenTree::Literal(lit)) => match lit.to_string().parse::<usize>() {
                Ok(x) => Ok(FieldName::Numeric(x, Spans::from_one(lit.span()))),
                Err(_) => Err(Error::with_span(lit.span(), EXPECTED)),
            },
            Some(TokenTree::Ident(ident)) => Ok(Self::from_ident(&ident)),
            Some(tt) => Err(Error::with_span(tt.span(), EXPECTED)),
            None => Err(Error::with_span(Span::call_site(), EXPECTED)),
        }
    }

    pub(crate) fn from_ident(ident: &Ident) -> Self {
        let s = Rc::<str>::from(utils::ident_to_string_no_raw(&ident));
        FieldName::Alphabetic(s, Spans::from_one(ident.span()))
    }
}

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
    #[cfg(test)]
    pub(crate) fn new_dummy() -> Self {
        Crate {
            ident: Ident::new("crate", Span::call_site()),
        }
    }

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

#[cfg(feature = "derive")]
impl quote::ToTokens for Crate {
    fn to_tokens(&self, ts: &mut used_proc_macro::TokenStream) {
        self.ident.to_tokens(ts);
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "__dbg", derive(Debug))]
#[derive(Clone)]
pub(crate) struct OpaqueType {
    pub(crate) spans: Spans,
    pub(crate) ty: TokenStream,
}

///////////////////////////////////////////////////////////////////////////////

/// Loosely parsed `#[...]`-style attributes
#[cfg_attr(feature = "__dbg", derive(Debug))]
#[derive(Clone)]
pub(crate) struct Attributes {
    pub(crate) attrs: TokenStream,
    pub(crate) spans: Spans,
}

impl Attributes {
    pub(crate) fn new() -> Self {
        Self {
            attrs: TokenStream::new(),
            spans: Spans::from_one(Span::call_site()),
        }
    }

    pub(crate) fn take(&mut self) -> Self {
        let attrs = core::mem::replace(&mut self.attrs, TokenStream::new());
        Self {
            attrs,
            spans: self.spans,
        }
    }

    pub(crate) fn ensure_used(&self) -> Result<(), Error> {
        if self.attrs.is_empty() {
            Ok(())
        } else {
            self.unused_error()
        }
    }

    pub(crate) fn unused_error<T>(&self) -> Result<T, Error> {
        Err(Error::new(self.spans, "these attributes are unused"))
    }

    /// Appens an attribute that comes after `self` into `self`.
    pub(crate) fn append(&mut self, other: Attributes) {
        if self.attrs.is_empty() {
            *self = other;
        } else {
            self.attrs.extend(other.attrs);
            self.spans.end = other.spans.end;
        }
    }

    pub(crate) fn parse(input: ParseStream<'_>) -> Self {
        let mut ts = TokenStream::new();

        let start = input.span();

        while matches!(
            input.peekn(2),
            [tt0, tt1]
            if tt0.is_punct('#') && tt1.is_group(Delimiter::Bracket)
        ) {
            ts.extend(input.next());
            ts.extend(input.next());
        }

        let end = input.last_span();

        Self {
            attrs: ts,
            spans: Spans { start, end },
        }
    }
}
