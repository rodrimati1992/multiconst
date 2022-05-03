use alloc::format;

use used_proc_macro::{
    token_stream::IntoIter, Delimiter, Group, Ident, Punct, Span, TokenStream, TokenTree,
};

use crate::{
    syntax::{OpaqueType, Spans},
    utils::{IsIdent, PeekableN, TokenStreamExt, TokenTreeExt},
    Error,
};

pub(crate) type ParseStream<'a> = &'a mut ParseBuffer;

pub(crate) struct ParseBuffer {
    iter: PeekableN<IntoIter>,
    last_span: Span,
}

impl ParseBuffer {
    pub(crate) fn new(ts: TokenStream) -> Self {
        let iter = PeekableN::new(ts);
        let last_span = Span::call_site();
        Self { iter, last_span }
    }

    pub(crate) fn is_empty(&mut self) -> bool {
        self.iter.peek().is_none()
    }

    pub(crate) fn peek(&mut self) -> Option<&TokenTree> {
        self.iter.peek()
    }

    pub(crate) fn peekn(&mut self, n: usize) -> &[TokenTree] {
        self.iter.peekn(n)
    }

    pub(crate) fn last_span(&self) -> Span {
        self.last_span
    }

    /// The span of the next token.
    pub(crate) fn span(&mut self) -> Span {
        match self.peek() {
            Some(tt) => tt.span(),
            None => self.last_span,
        }
    }

    pub(crate) fn error<S>(&self, msg: S) -> crate::Error
    where
        S: core::fmt::Display,
    {
        Error::with_span(self.last_span(), msg)
    }

    fn is_type_terminator(tt: &TokenTree) -> bool {
        matches!(
            tt,
            TokenTree::Punct(punct)
            if {
                let c = punct.as_char();
                matches!(c, ';' | ',' | '=')
            }
        )
    }

    pub(crate) fn parse_vis(&mut self) -> TokenStream {
        match self.peek() {
            Some(tt) if tt.is_ident("pub") => {
                let mut ts = TokenStream::from(self.next().unwrap());

                if matches!(self.peek(), Some(tt) if tt.is_group(Delimiter::Parenthesis)) {
                    ts.extend(self.next());
                }

                ts
            }
            _ => TokenStream::new(),
        }
    }

    pub(crate) fn parse_opaque_type_with<F>(
        &mut self,
        mut is_type_terminator: F,
    ) -> Result<OpaqueType, crate::Error>
    where
        F: FnMut(&TokenTree) -> bool,
    {
        let mut level = 0usize;

        let mut out = TokenStream::new();

        if self.is_empty() {
            return Err(self.error("expected type after this"));
        }

        let start_span = self.span();
        loop {
            let tt = self.peek();

            match tt {
                Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => {
                    level += 1;
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => {
                    level = level
                        .checked_sub(1)
                        .ok_or_else(|| Error::with_span(punct.span(), "unexpected '>'"))?;
                }
                Some(tt) if level == 0 && is_type_terminator(tt) => {
                    if out.is_empty() {
                        return Err(Error::with_span(
                            tt.span(),
                            "expected type, found no tokens",
                        ));
                    } else {
                        break;
                    };
                }
                _ => {}
            }

            if let Some(tt) = self.next() {
                out.append_one(tt);
            } else {
                break;
            }
        }

        if level == 0 {
            Ok(OpaqueType {
                spans: Spans {
                    start: start_span,
                    end: self.last_span(),
                },
                ty: out,
            })
        } else {
            Err(Error::with_span(self.last_span(), "incomplete type"))
        }
    }

    /// Parses a type opaquely, stopping at `,` òr `;`.
    ///
    /// This doesn't handle None-delimited at all
    pub(crate) fn parse_opaque_type(&mut self) -> Result<OpaqueType, crate::Error> {
        self.parse_opaque_type_with(ParseBuffer::is_type_terminator)
    }

    pub(crate) fn assert_empty(&mut self) -> Result<(), crate::Error> {
        match self.peek() {
            Some(x) => Err(Error::with_span(x.span(), "expected no more tokens")),
            None => Ok(()),
        }
    }

    pub(crate) fn parse_punct(&mut self, c: char) -> Result<Punct, Error> {
        match self.next() {
            Some(TokenTree::Punct(x)) if x.as_char() == c => Ok(x),
            Some(tt) => Err(Error::with_span(tt.span(), format!("expected a `{}`", c))),
            None => Err(Error::with_span(
                self.last_span(),
                format!("expected a `{}` after this", c),
            )),
        }
    }
    pub(crate) fn parse_opt_punct(&mut self, c: char) -> Result<Option<Punct>, Error> {
        match self.next() {
            Some(TokenTree::Punct(x)) if x.as_char() == c => Ok(Some(x)),
            Some(tt) => Err(Error::with_span(tt.span(), &format!("expected a `{}`", c))),
            None => Ok(None),
        }
    }

    pub(crate) fn parse_group(&mut self) -> Result<Group, Error> {
        match self.next() {
            Some(TokenTree::Group(x)) => Ok(x),
            Some(tt) => Err(Error::with_span(
                tt.span(),
                "expected pairs of `()`, `[]`, `{}`, or a None-delimited group",
            )),
            None => Err(Error::with_span(
                self.last_span(),
                "expected pairs of `()`, `[]`, `{}`, or a None-delimited group after this",
            )),
        }
    }

    pub(crate) fn parse_ident(&mut self) -> Result<Ident, Error> {
        match self.next() {
            Some(TokenTree::Ident(x)) => Ok(x),
            Some(tt) => Err(Error::with_span(tt.span(), "expected an identifier")),
            None => Err(Error::with_span(
                self.last_span(),
                "expected an identifier after this",
            )),
        }
    }

    pub(crate) fn parse_keyword(&mut self, keyword: &str) -> Result<Ident, Error> {
        match self.next() {
            Some(TokenTree::Ident(x)) if x.is_ident(keyword) => Ok(x),
            Some(tt) => Err(Error::with_span(
                tt.span(),
                &format!("expected the `{}` keyword", keyword),
            )),
            None => Err(Error::with_span(
                self.last_span(),
                &format!("expected the `{}` keyword after this", keyword),
            )),
        }
    }

    pub(crate) fn peek_parse_keyword(&mut self, keyword: &str) -> Option<Ident> {
        match self.peek() {
            Some(TokenTree::Ident(x)) if x.is_ident(keyword) => {
                if let Some(TokenTree::Ident(x)) = self.next() {
                    Some(x)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn tokens_until<F>(&mut self, mut func: F) -> TokenStream
    where
        F: FnMut(&TokenTree) -> bool,
    {
        let mut ts = TokenStream::new();

        loop {
            match self.peek() {
                Some(tt) if !func(tt) => ts.extend(self.next()),
                _ => return ts,
            }
        }
    }
}

impl Iterator for ParseBuffer {
    type Item = TokenTree;

    fn next(&mut self) -> Option<TokenTree> {
        let next = self.iter.next();
        if let Some(x) = &next {
            self.last_span = x.span();
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////
