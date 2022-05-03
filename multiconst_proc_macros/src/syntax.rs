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

pub(crate) fn is_path_separator(p0: &Punct, p1: &Punct) -> bool {
    p0.as_char() == ':'
        && p0.spacing() == Spacing::Joint
        && p1.as_char() == ':'
        && p1.spacing() == Spacing::Alone
}

enum PathToken {
    Type,
    Other,
}

fn is_path_token(tt: Option<&TokenTree>) -> Option<PathToken> {
    match tt {
        Some(TokenTree::Ident(_)) => Some(PathToken::Other),
        Some(TokenTree::Punct(p)) => match p.as_char() {
            '<' => Some(PathToken::Type),
            ':' => Some(PathToken::Other),
            _ => None,
        },
        _ => None,
    }
}

/// Loose path parsing
pub(crate) struct Path {
    pub(crate) tokens: TokenStream,
}

impl Path {
    pub(crate) fn parse(input: ParseStream<'_>) -> Result<Path, Error> {
        let mut out = TokenStream::new();

        while let Some(pt) = is_path_token(input.peek()) {
            match pt {
                PathToken::Type => {
                    let ot = input.parse_opaque_type_with(|_| true)?;
                    out.extend(ot.ty)
                }
                PathToken::Other => {
                    out.extend(input.next());
                }
            }
        }

        if out.is_empty() {
            return Err(input.error("expected path after this token"));
        }

        Ok(Path { tokens: out })
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) enum FieldIdent {
    Numeric(usize, Spans),
    Alphabetic(Rc<str>, Spans),
    /// A numeric identifier, determined by a constant in the expanded code.
    NumericConst(TokenStream, Spans),
}

impl FieldIdent {
    pub(crate) fn to_token_stream(&self, crate_kw: &Crate, ts: &mut TokenStream) {
        match *self {
            FieldIdent::Numeric(n, spans) => {
                crate_kw.item_to_ts("Usize", spans, ts);

                ts.append_one(Punct::new('<', Spacing::Joint).with_span(spans.start));
                ts.append_one(Literal::usize_unsuffixed(n).with_span(spans.end));
                ts.append_one(Punct::new('>', Spacing::Joint).with_span(spans.end));
            }
            FieldIdent::Alphabetic(ref str, spans) => {
                let mut chars = str.chars();
                let span = spans.start;

                crate_kw.item_to_ts("TIdent", spans, ts);

                ts.append_one(Punct::new('<', Spacing::Joint).with_span(span));

                tokenize_delim(Delimiter::Parenthesis, span, ts, |ts| {
                    while !chars.as_str().is_empty() {
                        crate_kw.item_to_ts("TChars", spans, ts);

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
            FieldIdent::NumericConst(ref x, spans) => {
                crate_kw.item_to_ts("Usize", spans, ts);

                ts.append_one(Punct::new('<', Spacing::Joint).with_span(spans.start));
                ts.append_one(Group::new(Delimiter::Brace, x.clone()));
                ts.append_one(Punct::new('>', Spacing::Joint).with_span(spans.end));
            }
        }
    }

    pub(crate) fn parse(input: ParseStream<'_>) -> Result<Self, Error> {
        const EXPECTED: &str = "expected either an untyped numeric literal or an identifier";

        match input.next() {
            Some(TokenTree::Literal(lit)) => match lit.to_string().parse::<usize>() {
                Ok(x) => Ok(FieldIdent::Numeric(x, Spans::from_one(lit.span()))),
                Err(_) => Err(Error::with_span(lit.span(), EXPECTED)),
            },
            Some(TokenTree::Ident(ident)) => Ok(Self::from_ident(&ident)),
            Some(tt) => Err(Error::with_span(tt.span(), EXPECTED)),
            None => Err(Error::with_span(Span::call_site(), EXPECTED)),
        }
    }

    pub(crate) fn from_ident(ident: &Ident) -> Self {
        let s = Rc::<str>::from(utils::ident_to_string_no_raw(&ident));
        FieldIdent::Alphabetic(s, Spans::from_one(ident.span()))
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
            Err(Error::new(self.spans, "these attributes are unused"))
        }
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
