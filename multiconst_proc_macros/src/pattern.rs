use used_proc_macro::{
    Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree,
};

use alloc::{format, string::String, vec::Vec};

use crate::{
    parsing::{ParseBuffer, ParseStream},
    syntax::{self, tokenize_delim, Crate, OpaqueType, Spans},
    utils::{ident_to_string_no_raw, TokenStreamExt, WithSpan},
    Error,
};

pub(crate) enum FieldIdent {
    Numeric(usize, Spans),
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
            FieldIdent::NumericConst(ref x, spans) => {
                crate_kw.item_to_ts("Usize", spans, ts);

                ts.append_one(Punct::new('<', Spacing::Joint).with_span(spans.start));
                ts.append_one(Group::new(Delimiter::Brace, x.clone()));
                ts.append_one(Punct::new('>', Spacing::Joint).with_span(spans.end));
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct BindingAndType {
    // stores the ident of the user-defined constant
    //
    // if this is None, it's a destructured local,
    // that's not assigned to a constant.
    pub(crate) constant: Ident,
    // the generated identifier for the temporary variable
    // that the pattern is destructured into.
    pub(crate) local: Ident,
    pub(crate) type_: OpaqueType,
}

#[derive(Clone)]
#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct Binding {
    // stores the ident of the user-defined constant
    //
    // if this is None, it's a destructured local,
    // that's not assigned to a constant.
    pub(crate) constant: Ident,
    // the generated identifier for the temporary variable
    // that the pattern is destructured into.
    pub(crate) local: Ident,
}

impl Binding {
    pub(crate) fn with_type(&self, type_: OpaqueType) -> BindingAndType {
        let Self { constant, local } = self.clone();
        BindingAndType {
            constant,
            local,
            type_,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) enum Pattern {
    Underscore(Span),
    Rem(RemPat),
    Ident(Binding),
    Array(ArrayPat),
    Tuple(TuplePat),
}

/// Represents a `..`/`foo @ ..` pattern, with an optional type annotation
#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct RemPat {
    pub(crate) spans: Spans,
    /// the index at which the remaining elements are located.
    pub(crate) binding: Option<Binding>,
}

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct ArrayPat {
    pub(crate) brackets: Span,
    pub(crate) elems: Vec<Pattern>,
    /// the index at which `..` was written.
    pub(crate) rem: Option<usize>,
}

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct TuplePat {
    pub(crate) parentheses: Span,
    pub(crate) elems: Vec<Pattern>,
    /// the index at which `..` was written.
    pub(crate) rem: Option<usize>,
}

impl Pattern {
    pub(crate) fn parse(input: ParseStream<'_>) -> Result<Pattern, Error> {
        match Self::parse_inner(input)? {
            Pattern::Rem(rem_pat) => Err(Error::new(
                rem_pat.spans,
                "`..` patterns are not allowed here ",
            )),
            pat => Ok(pat),
        }
    }

    /// looser parsing that allows `..` patterns
    fn parse_inner(input: ParseStream<'_>) -> Result<Pattern, Error> {
        let first_tt = input.next().ok_or_else(|| {
            Error::with_span(input.last_span(), "expected a pattern, found nothing")
        })?;

        match first_tt {
            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                Self::parse_inner(&mut ParseBuffer::new(group.stream()))
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Bracket => {
                parse_array(&group).map(Pattern::Array)
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
                parse_tuple(&group)
            }
            TokenTree::Ident(ident) => {
                let mut as_string = ident_to_string_no_raw(&ident);

                if let Some(_) = peek_parse_at_remainder(input)? {
                    let binding = Some(make_binding(&ident, &mut as_string));
                    let spans = Spans {
                        start: ident.span(),
                        end: input.last_span(),
                    };
                    Ok(Pattern::Rem(RemPat { spans, binding }))
                } else {
                    if as_string == "_" {
                        Ok(Pattern::Underscore(ident.span()))
                    } else {
                        Ok(Pattern::Ident(make_binding(&ident, &mut as_string)))
                    }
                }
            }
            TokenTree::Punct(punct) if punct.as_char() == '.' => {
                input.parse_punct('.')?;
                let spans = Spans {
                    start: punct.span(),
                    end: input.last_span(),
                };

                Ok(Pattern::Rem(RemPat {
                    spans,
                    binding: None,
                }))
            }
            tt => Err(Error::with_span(
                input.last_span(),
                alloc::format!("expected a pattern, found: {:?}", tt),
            )),
        }
    }
}

struct Sequence {
    elems: Vec<Pattern>,
    rem: Option<usize>,
    comma_sep: bool,
}

fn parse_sequence(
    type_constr: &'static str,
    input: ParseStream<'_>,
    rem_checker: &mut dyn FnMut(&RemPat) -> Result<(), Error>,
) -> Result<Sequence, Error> {
    let mut elems = Vec::new();
    let mut rem = None::<usize>;
    let mut i = 0usize;
    let mut comma_sep = false;

    while !input.is_empty() {
        let elem = Pattern::parse_inner(input)?;

        if let Pattern::Rem(rempat) = &elem {
            if let Some(_) = rem {
                return Err(Error::new(
                    elem.spans(),
                    format!("cannot use `..` patterns multiple times in {}", type_constr),
                ));
            }

            rem_checker(rempat)?;

            rem = Some(i);
        }

        elems.push(elem);

        comma_sep = comma_sep | input.parse_opt_punct(',')?.is_some();
        i += 1;
    }

    Ok(Sequence {
        elems,
        rem,
        comma_sep,
    })
}

fn parse_array(group: &Group) -> Result<ArrayPat, Error> {
    let brackets = group.span();
    let Sequence { elems, rem, .. } = parse_sequence(
        "array patterns",
        &mut ParseBuffer::new(group.stream()),
        &mut |_| Ok(()),
    )?;

    Ok(ArrayPat {
        brackets,
        elems,
        rem,
    })
}

fn parse_tuple(group: &Group) -> Result<Pattern, Error> {
    let parentheses = group.span();

    let mut has_remainder = false;

    let Sequence {
        mut elems,
        rem,
        comma_sep,
    } = parse_sequence(
        "array patterns",
        &mut ParseBuffer::new(group.stream()),
        &mut |rem| {
            has_remainder = true;
            match rem.binding {
                Some(_) => {
                    let msg = "cannot use `@ ..` patterns in tuple patterns";
                    Err(Error::new(rem.spans, msg))
                }
                None => Ok(()),
            }
        },
    )?;

    if comma_sep || has_remainder {
        Ok(Pattern::Tuple(TuplePat {
            parentheses,
            elems,
            rem,
        }))
    } else {
        Ok(elems.pop().unwrap())
    }
}

fn make_binding(ident: &Ident, as_string: &mut String) -> Binding {
    as_string.push_str("__local_variable");

    Binding {
        local: Ident::new(as_string, Span::mixed_site()).with_span(ident.span()),
        constant: ident.clone(),
    }
}

fn peek_parse_at_remainder(input: ParseStream<'_>) -> Result<Option<()>, Error> {
    if let Some(_) = input.peek_parse_punct('@') {
        input.parse_punct('.')?;
        input.parse_punct('.')?;

        Ok(Some(()))
    } else {
        Ok(None)
    }
}

impl Pattern {
    pub(crate) fn spans(&self) -> Spans {
        match self {
            Pattern::Array(ArrayPat { brackets: span, .. })
            | Pattern::Tuple(TuplePat {
                parentheses: span, ..
            })
            | Pattern::Underscore(span) => Spans::from_one(*span),
            Pattern::Rem(RemPat { spans, .. }) => *spans,
            Pattern::Ident(binding) => Spans::from_one(binding.local.span()),
        }
    }

    pub(crate) fn end_span(&self) -> Span {
        match self {
            Pattern::Array(ArrayPat { brackets: span, .. })
            | Pattern::Tuple(TuplePat {
                parentheses: span, ..
            })
            | Pattern::Underscore(span) => *span,
            Pattern::Rem(RemPat { spans, .. }) => spans.end,
            Pattern::Ident(binding) => binding.local.span(),
        }
    }

    pub(crate) fn is_not_rem(&self) -> bool {
        !matches!(self, Pattern::Rem { .. })
    }

    pub(crate) fn to_token_stream(&self, ts: &mut TokenStream) {
        match self {
            Pattern::Array(arr_pat) => {
                tokenize_delim(Delimiter::Bracket, arr_pat.brackets, ts, |ts| {
                    for elem in &arr_pat.elems {
                        elem.to_token_stream(ts);
                        syntax::tokenize_comma(elem.end_span(), ts);
                    }
                });
            }
            Pattern::Tuple(tup_pat) => {
                tokenize_delim(Delimiter::Parenthesis, tup_pat.parentheses, ts, |ts| {
                    for elem in &tup_pat.elems {
                        elem.to_token_stream(ts);
                        syntax::tokenize_comma(elem.end_span(), ts);
                    }
                });
            }
            Pattern::Rem(rem) => {
                let end_span = rem.spans.end;
                if let Some(Binding { local, .. }) = &rem.binding {
                    ts.append_one(local.clone());
                    ts.append_one(Punct::new('@', Spacing::Alone).with_span(end_span))
                }
                ts.append_one(Punct::new('.', Spacing::Joint).with_span(end_span));
                ts.append_one(Punct::new('.', Spacing::Alone).with_span(end_span));
            }
            Pattern::Ident(binding) => {
                ts.append_one(binding.local.clone());
            }
            Pattern::Underscore(span) => {
                ts.append_one(Ident::new("_", *span));
            }
        }
    }
}
