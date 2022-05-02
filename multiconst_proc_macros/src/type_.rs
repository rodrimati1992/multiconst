use used_proc_macro::{Delimiter, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use core::marker::PhantomData;

use alloc::{boxed::Box, vec, vec::Vec};

use crate::{
    parsing::{ParseBuffer, ParseStream},
    syntax::{tokenize_delim, tokenize_iter_delim, OpaqueType, Spans},
    utils::{TokenStreamExt, WithSpan},
    Error,
};

#[cfg_attr(feature = "__dbg", derive(Debug))]
#[derive(Clone)]
pub(crate) enum Type<L> {
    Underscore(Span),
    Array(ArrayType<L>),
    Tuple(TupleType<L>),
    Opaque(OpaqueType),
}

pub(crate) type ParsedType = Type<Option<TokenStream>>;
pub(crate) type RealType = Type<TokenStream>;

#[cfg_attr(feature = "__dbg", derive(Debug))]
#[derive(Clone)]
pub(crate) struct ArrayType<L = TokenStream> {
    pub(crate) brackets: Span,
    pub(crate) elem_ty: Box<Type<L>>,
    pub(crate) len: L,
}

#[cfg_attr(feature = "__dbg", derive(Debug))]
#[derive(Clone)]
pub(crate) struct TupleType<L = TokenStream> {
    pub(crate) parentheses: Span,
    pub(crate) elem_tys: Vec<Type<L>>,
    pub(crate) _marker: PhantomData<L>,
}

impl<L> Type<L> {
    pub(crate) fn spans(&self) -> Spans {
        match self {
            Type::Underscore(span)
            | Type::Array(ArrayType { brackets: span, .. })
            | Type::Tuple(TupleType {
                parentheses: span, ..
            }) => Spans::from_one(*span),
            Type::Opaque(x) => x.spans,
        }
    }
    pub(crate) fn end_span(&self) -> Span {
        match self {
            Type::Underscore(span)
            | Type::Array(ArrayType { brackets: span, .. })
            | Type::Tuple(TupleType {
                parentheses: span, ..
            }) => *span,
            Type::Opaque(x) => x.spans.end,
        }
    }
}

impl ParsedType {
    pub(crate) fn definite_length_real_type(self) -> Result<RealType, Error> {
        match self {
            Type::Array(ArrayType {
                brackets,
                len,
                elem_ty,
            }) => {
                const ERR: &str = "expected non-inferred length";
                let len = len.ok_or_else(|| Error::with_span(brackets, ERR))?;

                Ok(Type::Array(ArrayType {
                    brackets,
                    elem_ty: Box::new(elem_ty.definite_length_real_type()?),
                    len,
                }))
            }
            Type::Tuple(tup_ty) => Ok(Type::Tuple(TupleType {
                parentheses: tup_ty.parentheses,
                elem_tys: tup_ty
                    .elem_tys
                    .into_iter()
                    .map(Type::definite_length_real_type)
                    .collect::<Result<Vec<RealType>, Error>>()?,
                _marker: PhantomData,
            })),
            Type::Underscore(span) => Ok(Type::Underscore(span)),
            Type::Opaque(opaque) => Ok(Type::Opaque(opaque)),
        }
    }
    pub(crate) fn parse(input: ParseStream<'_>) -> Result<ParsedType, Error> {
        if matches!(input.peek(), Some(TokenTree::Group { .. })) {
            let group = if let Some(TokenTree::Group(group)) = input.next() {
                group
            } else {
                unreachable!("{}", core::panic::Location::caller())
            };

            let input = &mut ParseBuffer::new(group.stream());

            match group.delimiter() {
                Delimiter::Bracket => {
                    let elem_ty = Box::new(Type::parse(input)?);
                    input.parse_punct(';')?;

                    let len = if input.peek_parse_keyword("_").is_some() {
                        input.assert_empty()?;
                        None
                    } else {
                        Some(input.collect::<TokenStream>())
                    };

                    Ok(Type::Array(ArrayType {
                        brackets: group.span(),
                        elem_ty,
                        len,
                    }))
                }
                Delimiter::Brace => {
                    return Err(Error::with_span(
                        group.span(),
                        "expected type, found braces",
                    ));
                }
                Delimiter::Parenthesis => {
                    if input.is_empty() {
                        return Ok(Type::Tuple(TupleType {
                            parentheses: group.span(),
                            elem_tys: Vec::new(),
                            _marker: PhantomData,
                        }));
                    }

                    let first_type = Type::parse(input)?;

                    if input.is_empty() {
                        Ok(first_type)
                    } else {
                        input.parse_punct(',')?;

                        let mut elem_tys = vec![first_type];

                        while !input.is_empty() {
                            elem_tys.push(Type::parse(input)?);

                            input.parse_opt_punct(',')?;
                        }

                        Ok(Type::Tuple(TupleType {
                            parentheses: group.span(),
                            elem_tys,
                            _marker: PhantomData,
                        }))
                    }
                }
                Delimiter::None => return Type::parse(&mut ParseBuffer::new(group.stream())),
            }
        } else if let Some(TokenTree::Literal(lit)) = input.peek() {
            Err(Error::with_span(lit.span(), "expected type, found literal"))
        } else if let Some(ident) = input.peek_parse_keyword("_") {
            Ok(Type::Underscore(ident.span()))
        } else {
            input.parse_opaque_type().map(Type::Opaque)
        }
    }
}

impl RealType {
    pub(crate) fn to_opaque(&self) -> OpaqueType {
        OpaqueType {
            spans: self.spans(),
            ty: self.to_tokens(),
        }
    }

    pub(crate) fn to_token_stream(&self, ts: &mut TokenStream) {
        match self {
            Type::Array(arr_ty) => arr_ty.to_token_stream(ts),
            Type::Tuple(tup_ty) => tup_ty.to_token_stream(ts),
            Type::Opaque(OpaqueType { ty, .. }) => ts.extend(ty.clone()),
            Type::Underscore(span) => ts.append_one(Ident::new("_", *span)),
        }
    }
    pub(crate) fn to_tokens(&self) -> TokenStream {
        let mut ts = TokenStream::new();
        self.to_token_stream(&mut ts);
        ts
    }
}

impl ArrayType {
    pub(crate) fn to_token_stream(&self, ts: &mut TokenStream) {
        let ArrayType {
            brackets,
            elem_ty,
            len,
        } = self;

        tokenize_delim(Delimiter::Bracket, *brackets, ts, |ts| {
            elem_ty.to_token_stream(ts);
            ts.append_one(Punct::new(';', Spacing::Alone).with_span(*brackets));
            ts.extend(len.clone());
        });
    }
}

impl TupleType {
    pub(crate) fn to_token_stream(&self, ts: &mut TokenStream) {
        let TupleType {
            parentheses,
            elem_tys,
            ..
        } = self;
        tokenize_iter_delim(
            Delimiter::Parenthesis,
            *parentheses,
            elem_tys,
            ts,
            |ts, elem| {
                elem.to_token_stream(ts);
                ts.append_one(Punct::new(',', Spacing::Alone).with_span(*parentheses));
            },
        );
    }
}
