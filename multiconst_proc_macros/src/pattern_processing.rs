use used_proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    pattern::{ArrayPat, BindingAndType, FieldIdent, Pattern, RemPat, TuplePat},
    syntax::{self, Crate, OpaqueType, Spans},
    type_::{ArrayType, ParsedType, RealType, TupleType, Type},
    utils::{TokenStreamExt, WithSpan},
    Error,
};

#[derive(Copy, Clone)]
pub(crate) enum FieldType<'a> {
    Direct(&'a RealType),
    Derived {
        spans: Spans,
        field_name: &'a FieldIdent,
        inside: &'a FieldType<'a>,
    },
}

impl<'a> FieldType<'a> {
    fn spans(self) -> Spans {
        match self {
            FieldType::Direct(x) => x.spans(),
            FieldType::Derived { spans, .. } => spans,
        }
    }
    fn to_opaque(self, crate_kw: &Crate) -> OpaqueType {
        let mut ty = TokenStream::new();
        let spans = self.to_token_stream(crate_kw, &mut ty);
        OpaqueType { ty, spans }
    }

    fn to_tokens(self, crate_kw: &Crate) -> TokenStream {
        let mut ts = TokenStream::new();
        self.to_token_stream(crate_kw, &mut ts);
        ts
    }

    fn to_token_stream(self, crate_kw: &Crate, ts: &mut TokenStream) -> Spans {
        match self {
            FieldType::Direct(x) => {
                x.to_token_stream(ts);
                x.spans()
            }
            FieldType::Derived { spans, .. } => {
                let mut path = TokenStream::new();

                crate_kw.item_to_ts("GetFieldType", spans, ts);
                ts.append_one(Punct::new('<', Spacing::Alone).with_span(spans.start));
                self.to_token_stream_inner(crate_kw, ts, &mut path);
                ts.append_one(Group::new(Delimiter::Parenthesis, path).with_span(spans.end));
                ts.append_one(Punct::new('>', Spacing::Alone).with_span(spans.end));

                spans
            }
        }
    }

    fn to_token_stream_inner(
        self,
        crate_kw: &Crate,
        ts: &mut TokenStream,
        path_ts: &mut TokenStream,
    ) {
        match self {
            FieldType::Direct(x) => {
                ts.extend(x.to_tokens());
                ts.append_one(Punct::new(',', Spacing::Alone).with_span(x.end_span()));
            }
            FieldType::Derived {
                field_name,
                inside,
                spans,
            } => {
                inside.to_token_stream_inner(crate_kw, ts, path_ts);
                field_name.to_token_stream(crate_kw, path_ts);
                path_ts.append_one(Punct::new(',', Spacing::Alone).with_span(spans.end));
            }
        }
    }
}

pub(crate) struct ExtractConstCtx<'a> {
    pub(crate) bats: &'a mut Vec<BindingAndType>,
    /// The length of the `..` pattern in tuple patterns
    pub(crate) tuple_rem_lens: &'a mut Vec<TokenStream>,
    pub(crate) tuple_rem_pat_const: &'a Ident,
    pub(crate) crate_kw: &'a Crate,
}

pub(crate) fn find_first_const_ident(pattern: &Pattern) -> Option<&Ident> {
    match pattern {
        Pattern::Array(ArrayPat { elems, .. }) | Pattern::Tuple(TuplePat { elems, .. }) => {
            elems.iter().find_map(find_first_const_ident)
        }
        Pattern::Underscore(_) => None,
        Pattern::Rem(RemPat { binding, .. }) => binding.as_ref().map(|b| &b.constant),
        Pattern::Ident(binding) => Some(&binding.constant),
    }
}

/// Finds the first element pattern in the array pattern that can infer its own length.
fn find_first_ok_real_type(elems: &[Pattern], type_: ParsedType) -> Result<RealType, Error> {
    if elems.is_empty() {
        return type_.definite_length_real_type();
    }

    let mut last_res = None;

    for elem in elems.iter().filter(|p| p.is_not_rem()) {
        match real_type_from(elem, type_.clone()) {
            Ok(x) => return Ok(x),
            Err(e) => last_res = Some(e),
        }
    }

    Err(last_res.unwrap())
}

pub(crate) fn real_type_from(pattern: &Pattern, type_: ParsedType) -> Result<RealType, Error> {
    match (pattern, type_) {
        (Pattern::Underscore { .. }, ty) | (Pattern::Ident { .. }, ty) => {
            ty.definite_length_real_type()
        }
        (Pattern::Array(ArrayPat { rem, elems, .. }), Type::Array(arr_ty)) => {
            let len = match (rem, arr_ty.len) {
                (_, Some(len)) => len,
                (Some(_), None) => {
                    let msg = "cannot infer length when the array pattern contains a `..`";
                    return Err(Error::with_span(arr_ty.brackets, msg));
                }
                (None, None) => {
                    let x = Literal::usize_unsuffixed(elems.len()).with_span(arr_ty.brackets);
                    let x = TokenTree::Literal(x);
                    TokenStream::from(x)
                }
            };

            Ok(Type::Array(ArrayType {
                brackets: arr_ty.brackets,
                elem_ty: Box::new(find_first_ok_real_type(elems, *arr_ty.elem_ty)?),
                len,
            }))
        }
        (Pattern::Tuple(tup_pat), Type::Tuple(tup_ty)) => {
            let mut elem_tys = Vec::new();

            let (before_elems, taken, skipped, after_elems) = if let Some(pos) = tup_pat.rem {
                let skipped = tup_ty
                    .elem_tys
                    .len()
                    .saturating_sub(tup_pat.elems.len() - 1);
                (
                    &tup_pat.elems[..pos],
                    pos,
                    skipped,
                    &tup_pat.elems[pos + 1..],
                )
            } else {
                (&tup_pat.elems[..], tup_ty.elem_tys.len(), 0, &[][..])
            };

            let mut tys_iter = tup_ty.elem_tys.into_iter();

            for (elem, elem_ty) in before_elems.iter().zip(tys_iter.by_ref().take(taken)) {
                elem_tys.push(real_type_from(elem, elem_ty)?);
            }
            for elem_ty in tys_iter.by_ref().take(skipped) {
                elem_tys.push(elem_ty.definite_length_real_type()?);
            }
            for (elem, elem_ty) in after_elems.iter().zip(tys_iter.by_ref()) {
                elem_tys.push(real_type_from(elem, elem_ty)?);
            }

            Ok(Type::Tuple(TupleType {
                parentheses: tup_ty.parentheses,
                elem_tys,
                _marker: PhantomData,
            }))
        }
        (_, type_ @ Type::Opaque { .. }) | (_, type_ @ Type::Underscore { .. }) => {
            type_.definite_length_real_type()
        }
        (pat, Type::Array { .. }) | (pat, Type::Tuple { .. }) => {
            let s = "mismatched pattern and type";
            Err(Error::new(pat.spans(), s))
        }
    }
}

pub(crate) fn extract_const_names_tys(
    pattern: &Pattern,
    type_: FieldType<'_>,
    pctx: &mut ExtractConstCtx<'_>,
) -> Result<(), Error> {
    let ExtractConstCtx { crate_kw, .. } = *pctx;

    match pattern {
        Pattern::Ident(pat_ident) => {
            let type_ = type_.to_opaque(crate_kw);

            pctx.bats.push(pat_ident.with_type(type_));
            Ok(())
        }
        Pattern::Array(arr_pat) => {
            let spans = Spans::from_one(arr_pat.brackets);
            let field_name;
            let subfield_ty = match type_ {
                FieldType::Direct(Type::Array(ArrayType { elem_ty, .. })) => {
                    FieldType::Direct(elem_ty)
                }
                FieldType::Direct(Type::Opaque { .. }) | FieldType::Derived { .. } => {
                    field_name = FieldIdent::Numeric(0, spans);
                    FieldType::Derived {
                        spans,
                        field_name: &field_name,
                        inside: &type_,
                    }
                }
                FieldType::Direct(ty) => {
                    return Err(Error::new(ty.spans(), "expected array type"));
                }
            };

            let rem_length = || {
                let mut ts = match type_ {
                    FieldType::Direct(Type::Array(ArrayType { len, .. })) => len.clone(),
                    FieldType::Direct(_) | FieldType::Derived { .. } => {
                        syntax::tokenize_seq_length_assoc_const(
                            crate_kw,
                            type_.spans(),
                            type_.to_tokens(crate_kw),
                        )
                    }
                };

                let count = arr_pat.elems.len() - 1; // 1 being the remainder pattern

                ts.append_one(Punct::new('-', Spacing::Alone).with_span(arr_pat.brackets));
                ts.append_one(Literal::usize_unsuffixed(count).with_span(arr_pat.brackets));

                ts
            };

            for elem in &arr_pat.elems {
                match elem {
                    Pattern::Rem(RemPat {
                        binding: Some(binding),
                        ..
                    }) => {
                        let elem_ty = Type::Array(ArrayType {
                            brackets: binding.constant.span(),
                            elem_ty: Box::new(Type::Opaque(subfield_ty.to_opaque(crate_kw))),
                            len: rem_length(),
                        })
                        .to_opaque();

                        pctx.bats.push(binding.with_type(elem_ty));
                    }
                    Pattern::Rem(_) => {}
                    _ => extract_const_names_tys(elem, subfield_ty, pctx)?,
                }
            }
            Ok(())
        }
        Pattern::Tuple(tup_pat) => process_tup_pat(tup_pat, type_, pctx),
        Pattern::Rem { .. } => unreachable!("{}", core::panic::Location::caller()),
        Pattern::Underscore { .. } => Ok(()),
    }
}

fn process_tup_pat(
    tup_pat: &TuplePat,
    type_: FieldType<'_>,
    pctx: &mut ExtractConstCtx<'_>,
) -> Result<(), Error> {
    let ExtractConstCtx { crate_kw, .. } = *pctx;

    let spans = Spans::from_one(tup_pat.parentheses);

    let rem_pos = tup_pat.rem.unwrap_or_else(|| tup_pat.elems.len());
    let trailing_pattern_count = tup_pat.elems.len() - rem_pos;

    let mut i = 0;
    for elem in tup_pat.elems.iter() {
        let field_name;
        let subfield_ty = match type_ {
            FieldType::Direct(Type::Tuple(TupleType { elem_tys, .. })) => {
                if let Pattern::Rem(_) = elem {
                    i += elem_tys.len().saturating_sub(tup_pat.elems.len() - 1);
                    continue;
                } else {
                    let elem_ty = elem_tys.get(i).ok_or_else(|| {
                        Error::new(elem.spans(), "tuple element does not exist in the type")
                    })?;
                    FieldType::Direct(elem_ty)
                }
            }
            FieldType::Direct(Type::Opaque { .. }) | FieldType::Derived { .. } => {
                if let Pattern::Rem(_) = elem {
                    continue;
                }
                field_name = if tup_pat.rem.is_some() {
                    let sspan = elem.spans().start;
                    let mut trail_off = syntax::tokenize_seq_length_assoc_const(
                        crate_kw,
                        type_.spans(),
                        type_.to_tokens(crate_kw),
                    );

                    trail_off.append_one(Punct::new('-', Spacing::Alone).with_span(sspan));
                    trail_off.append_one(
                        Literal::usize_unsuffixed(trailing_pattern_count).with_span(sspan),
                    );

                    let i = pctx.tuple_rem_lens.len();
                    pctx.tuple_rem_lens.push(trail_off);

                    let num_const = TokenStream::from_array([
                        TokenTree::Ident(pctx.tuple_rem_pat_const.clone()),
                        {
                            let x = Literal::usize_unsuffixed(i).with_span(sspan);
                            let x = TokenTree::Literal(x).with_span(sspan);
                            let x = TokenStream::from(x).with_span(sspan);
                            Group::new(Delimiter::Bracket, x).with_span(sspan).into()
                        },
                        Punct::new('+', Spacing::Alone).with_span(sspan).into(),
                        Literal::usize_unsuffixed(i).with_span(sspan).into(),
                    ]);

                    FieldIdent::NumericConst(num_const, elem.spans())
                } else {
                    FieldIdent::Numeric(i, spans)
                };
                FieldType::Derived {
                    spans,
                    field_name: &field_name,
                    inside: &type_,
                }
            }
            FieldType::Direct(ty) => {
                return Err(Error::new(ty.spans(), "expected tuple type"));
            }
        };

        extract_const_names_tys(elem, subfield_ty, pctx)?;
        i += 1;
    }
    Ok(())
}
