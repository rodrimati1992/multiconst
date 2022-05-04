use used_proc_macro::{
    Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree,
};

use alloc::{format, string::String, vec::Vec};

use crate::{
    parsing::{ParseBuffer, ParseStream},
    syntax::{self, tokenize_delim, Attributes, FieldName, OpaqueType, Path, Spans},
    type_::{ParsedType, RealType},
    utils::{ident_to_string_no_raw, TokenStreamExt, TokenTreeExt, WithSpan},
    Error,
};

////////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct BindingAndType {
    pub(crate) attrs: Attributes,
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
    pub(crate) attrs: Attributes,
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
        let Self {
            attrs,
            constant,
            local,
        } = self.clone();
        BindingAndType {
            attrs,
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
    Struct(StructPat),
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

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct StructPat {
    pub(crate) path: Path,
    pub(crate) group_span: Span,
    pub(crate) elems: Vec<FieldPat>,
    pub(crate) rem: Option<Spans>,
    pub(crate) spans: Spans,
}

#[cfg_attr(feature = "__dbg", derive(Debug))]
pub(crate) struct FieldPat {
    // The name used in patterns
    pub(crate) pat_ident: TokenTree,
    // The name used in FieldType
    pub(crate) name: FieldName,
    pub(crate) pattern: Pattern,
    pub(crate) type_annotation: Option<RealType>,
}

struct ParseState {
    var_index: usize,
}

impl ParseState {
    fn new() -> Self {
        Self { var_index: 0 }
    }

    fn next_var_index(&mut self) -> usize {
        self.var_index += 1;
        self.var_index
    }
}

impl Pattern {
    pub(crate) fn parse(input: ParseStream<'_>) -> Result<Pattern, Error> {
        match Self::parse_inner(input, &mut ParseState::new(), Attributes::new())? {
            Pattern::Rem(rem_pat) => Err(Error::new(
                rem_pat.spans,
                "`..` patterns are not allowed here ",
            )),
            pat => Ok(pat),
        }
    }

    /// looser parsing that allows `..` patterns
    fn parse_inner(
        input: ParseStream<'_>,
        state: &mut ParseState,
        mut attrs: Attributes,
    ) -> Result<Pattern, Error> {
        use TokenTree as TT;

        let start_span = input.span();
        let peeked = input.peekn(3);

        let make_err = || -> Result<Pattern, Error> {
            Err(Error::with_span(
                start_span,
                "invalid pattern starting here",
            ))
        };

        let ret = match peeked {
            [TT::Punct(p), rem @ ..] => match p.as_char() {
                '#' => {
                    attrs.append(Attributes::parse(input));
                    return Self::parse_inner(input, state, attrs);
                }
                '.' => {
                    input.parse_punct('.')?;
                    input.parse_punct('.')?;
                    let spans = Spans {
                        start: start_span,
                        end: input.last_span(),
                    };

                    Ok(Pattern::Rem(RemPat {
                        spans,
                        binding: None,
                    }))
                }
                ':' if p.spacing() == Spacing::Joint
                    && matches!(
                        rem,
                        [TokenTree::Punct(p2), ..]
                        if p2.as_char() == ':' && p2.spacing() == Spacing::Alone
                    ) =>
                {
                    parse_struct_pat(input, state, attrs.take())
                }
                _ => make_err(),
            },
            [TT::Ident(ident), rem @ ..] => {
                match rem {
                    [TT::Punct(p0), ..] if p0.as_char() == '@' => {
                        let mut as_string = ident_to_string_no_raw(ident);
                        let binding =
                            Some(make_binding(ident, state, attrs.take(), &mut as_string));
                        input.next(); // skips the ident
                        input.next(); // skips the @
                        input.parse_punct('.')?;
                        input.parse_punct('.')?;

                        let spans = Spans {
                            start: start_span,
                            end: input.last_span(),
                        };

                        Ok(Pattern::Rem(RemPat { spans, binding }))
                    }
                    [TT::Punct(p0), TT::Punct(p1), ..]
                        if p0.as_char() == ':'
                            && p0.spacing() == Spacing::Joint
                            && p1.as_char() == ':'
                            && p1.spacing() == Spacing::Alone =>
                    {
                        parse_struct_pat(input, state, attrs.take())
                    }
                    [TT::Group(_), ..] => parse_struct_pat(input, state, attrs.take()),
                    _ => {
                        let mut as_string = ident_to_string_no_raw(ident);
                        if as_string == "_" {
                            input.next(); // skips the ident
                            Ok(Pattern::Underscore(start_span))
                        } else {
                            let binding = make_binding(ident, state, attrs.take(), &mut as_string);
                            input.next(); // skips the ident
                            Ok(Pattern::Ident(binding))
                        }
                    }
                }
            }
            [TT::Group(_), ..] => {
                let group = input.parse_group()?;
                match group.delimiter() {
                    Delimiter::None => Self::parse_inner(
                        &mut ParseBuffer::new(group.stream()),
                        state,
                        Attributes::new(),
                    ),
                    Delimiter::Parenthesis => parse_tuple(&group, state),
                    Delimiter::Bracket => parse_array(&group, state).map(Pattern::Array),
                    Delimiter::Brace => make_err(),
                }
            }

            [] => Err(Error::with_span(
                start_span,
                "expected a pattern after this",
            )),
            _ => make_err(),
        };

        attrs.ensure_used()?;

        ret
    }
}

fn parse_struct_pat(
    input: ParseStream<'_>,
    state: &mut ParseState,
    attrs: Attributes,
) -> Result<Pattern, Error> {
    let path = Path::parse(input)?;
    let group_span: Span;

    const ERR: &str = "expected `{}`/`()`-delimited struct fields";

    let (elems, rem) = match input.next() {
        Some(TokenTree::Group(group)) => {
            group_span = group.span();

            match group.delimiter() {
                Delimiter::Parenthesis => {
                    parse_struct_fields(&group, state, &attrs, |i, input| {
                        Ok((FieldName::Numeric(i, Spans::from_one(input.span())), {
                            let lit = Literal::usize_unsuffixed(i).with_span(input.span());
                            TokenTree::Literal(lit)
                        }))
                    })?
                }
                Delimiter::Brace => parse_struct_fields(&group, state, &attrs, |_, input| {
                    let ident = input.parse_ident()?;
                    input.parse_punct(':')?;
                    Ok((FieldName::from_ident(&ident), TokenTree::Ident(ident)))
                })?,
                _ => return Err(Error::with_span(group.span(), ERR)),
            }
        }
        Some(tt) => return Err(Error::with_span(tt.span(), ERR)),
        None => {
            return Err(input.error("expected `{}`/`()`-delimited struct fields after this token"))
        }
    };

    if elems.is_empty() {
        return attrs.unused_error();
    }

    Ok(Pattern::Struct(StructPat {
        spans: Spans {
            start: path.spans.start,
            end: group_span,
        },
        path,
        group_span,
        elems,
        rem,
    }))
}

fn parse_struct_fields<F>(
    group: &Group,
    state: &mut ParseState,
    attrs: &Attributes,
    mut field_name_parser: F,
) -> Result<(Vec<FieldPat>, Option<Spans>), Error>
where
    F: FnMut(usize, ParseStream<'_>) -> Result<(FieldName, TokenTree), Error>,
{
    let input = &mut ParseBuffer::with_span(group.stream(), group.span());

    let mut fields = Vec::<FieldPat>::new();
    let mut i = 0;

    while !input.is_empty() {
        enum EmptyTrailing {
            Yes,
            No,
        }

        fn is_rem_end(tts: &[TokenTree]) -> Option<EmptyTrailing> {
            let rem = match tts {
                [p0, p1, rem @ ..] if p0.is_punct('.') && p1.is_punct('.') => rem,
                _ => return None,
            };

            match rem {
                [p2, rem @ ..] if p2.is_punct(',') => {
                    let x = if rem.is_empty() {
                        EmptyTrailing::Yes
                    } else {
                        EmptyTrailing::No
                    };
                    Some(x)
                }
                [] => Some(EmptyTrailing::Yes),
                _ => None,
            }
        }

        if let Some(trailing) = is_rem_end(input.peekn(4)) {
            let start = input.span();
            let _ = input.next();
            let end = input.span();
            let spans = Spans { start, end };

            return if let EmptyTrailing::Yes = trailing {
                Ok((fields, Some(spans)))
            } else {
                Err(Error::new(
                    spans,
                    "only trailing `..` patterns are supported in structs",
                ))
            };
        }

        let mut attrs = attrs.clone();
        attrs.append(Attributes::parse(input));

        let (name, pat_ident) = field_name_parser(i, input)?;
        let pattern = Pattern::parse_inner(input, state, attrs)?;

        let type_annotation = if matches!(
            input.peek(),
            Some(TokenTree::Punct(p))
            if p.as_char() == ':'
        ) {
            let _ = input.parse_punct(':');
            let type_ = ParsedType::parse(input)?;
            Some(crate::pattern_processing::real_type_from(&pattern, type_)?)
        } else {
            None
        };

        fields.push(FieldPat {
            pat_ident,
            name,
            pattern,
            type_annotation,
        });

        i += 1;

        input.parse_opt_punct(',')?;
    }

    Ok((fields, None))
}

struct Sequence {
    elems: Vec<Pattern>,
    rem: Option<usize>,
    comma_sep: bool,
}

fn parse_sequence(
    type_constr: &'static str,
    state: &mut ParseState,
    input: ParseStream<'_>,
    rem_checker: &mut dyn FnMut(&RemPat) -> Result<(), Error>,
) -> Result<Sequence, Error> {
    let mut elems = Vec::new();
    let mut rem = None::<usize>;
    let mut i = 0usize;
    let mut comma_sep = false;

    while !input.is_empty() {
        let elem = Pattern::parse_inner(input, state, Attributes::new())?;

        if let Pattern::Rem(rempat) = &elem {
            if let Some(_) = rem {
                return Err(Error::new(
                    elem.spans(),
                    format!("cannot use `..` multiple times in {}", type_constr),
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

fn parse_array(group: &Group, state: &mut ParseState) -> Result<ArrayPat, Error> {
    let brackets = group.span();
    let Sequence { elems, rem, .. } = parse_sequence(
        "array patterns",
        state,
        &mut ParseBuffer::new(group.stream()),
        &mut |_| Ok(()),
    )?;

    Ok(ArrayPat {
        brackets,
        elems,
        rem,
    })
}

fn parse_tuple(group: &Group, state: &mut ParseState) -> Result<Pattern, Error> {
    let parentheses = group.span();

    let mut has_remainder = false;

    let Sequence {
        mut elems,
        rem,
        comma_sep,
    } = parse_sequence(
        "tuple patterns",
        state,
        &mut ParseBuffer::new(group.stream()),
        &mut |rem| {
            has_remainder = true;
            match rem.binding {
                Some(_) => {
                    let msg = "cannot use `@ ..` in tuple patterns";
                    Err(Error::new(rem.spans, msg))
                }
                None => Ok(()),
            }
        },
    )?;

    if comma_sep || has_remainder || elems.is_empty() {
        Ok(Pattern::Tuple(TuplePat {
            parentheses,
            elems,
            rem,
        }))
    } else {
        Ok(elems.pop().unwrap())
    }
}

fn make_binding(
    ident: &Ident,
    state: &mut ParseState,
    attrs: Attributes,
    as_string: &mut String,
) -> Binding {
    use core::fmt::Write;

    as_string.push_str("__local_variable");
    let _ = write!(as_string, "{}", state.next_var_index());

    Binding {
        attrs,
        local: Ident::new(as_string, Span::mixed_site()).with_span(ident.span()),
        constant: ident.clone(),
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
            Pattern::Rem(RemPat { spans, .. }) | Pattern::Struct(StructPat { spans, .. }) => *spans,
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
            Pattern::Rem(RemPat { spans, .. }) | Pattern::Struct(StructPat { spans, .. }) => {
                spans.end
            }
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
            Pattern::Struct(StructPat {
                path,
                group_span,
                elems,
                rem,
                ..
            }) => {
                ts.extend(path.tokens.clone());
                tokenize_delim(Delimiter::Brace, *group_span, ts, |ts| {
                    for FieldPat {
                        pat_ident, pattern, ..
                    } in elems
                    {
                        ts.append_one(pat_ident.clone());
                        ts.append_one(Punct::new(':', Spacing::Alone).with_span(pat_ident.span()));
                        pattern.to_token_stream(ts);
                        syntax::tokenize_comma(pattern.end_span(), ts);
                    }

                    if let Some(Spans { start, end }) = *rem {
                        ts.append_one(Punct::new('.', Spacing::Joint).with_span(start));
                        ts.append_one(Punct::new('.', Spacing::Alone).with_span(end));
                    }
                })
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
