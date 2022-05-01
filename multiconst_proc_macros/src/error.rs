use used_proc_macro::{
    Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree,
};

use core::fmt::Display;

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    syntax::{Crate, Spans},
    utils::{TokenStreamExt, WithSpan},
};

#[derive(Debug)]
pub(crate) struct Error {
    messages: Vec<CompileError>,
}

#[derive(Debug)]
struct CompileError {
    spans: Spans,
    msg: String,
}

impl Error {
    pub(crate) fn new<T: Display>(spans: Spans, msg: T) -> Self {
        Error {
            messages: vec![CompileError {
                spans,
                msg: msg.to_string(),
            }],
        }
    }
    pub(crate) fn with_span<T: Display>(span: Span, msg: T) -> Self {
        Self::new(
            Spans {
                start: span,
                end: span,
            },
            msg,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn join(mut self, mut other: Error) -> Error {
        self.messages.append(&mut other.messages);
        self
    }

    pub(crate) fn to_compile_error(&self, crate_kw: &Crate) -> TokenStream {
        self.messages
            .iter()
            .map(|CompileError { spans, msg }| {
                TokenStream::from_array([
                    TokenTree::Ident(crate_kw.ident.clone().with_span(spans.start)),
                    Punct::new(':', Spacing::Joint)
                        .with_span(spans.start)
                        .into(),
                    Punct::new(':', Spacing::Alone)
                        .with_span(spans.start)
                        .into(),
                    TokenTree::Ident(Ident::new("__", spans.start)),
                    Punct::new(':', Spacing::Joint)
                        .with_span(spans.start)
                        .into(),
                    Punct::new(':', Spacing::Alone)
                        .with_span(spans.start)
                        .into(),
                    TokenTree::Ident(Ident::new("compile_error", spans.start)),
                    Punct::new('!', Spacing::Alone)
                        .with_span(spans.start)
                        .into(),
                    {
                        let x = Literal::string(msg).with_span(spans.end);
                        let x = TokenTree::Literal(x);
                        let x = TokenStream::from(x);
                        let x = Group::new(Delimiter::Brace, x);
                        x.with_span(spans.end).into()
                    },
                ])
            })
            .collect()
    }
}
