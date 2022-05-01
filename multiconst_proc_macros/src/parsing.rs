use alloc::format;

use used_proc_macro::{
    token_stream::IntoIter, Delimiter, Ident, Punct, Span, TokenStream, TokenTree,
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

    fn span_or_last(&self, span: Option<TokenTree>) -> Span {
        match span {
            Some(x) => x.span(),
            None => self.last_span,
        }
    }

    pub(crate) fn error<S>(&self, msg: S) -> crate::Error
    where
        S: core::fmt::Display,
    {
        Error::with_span(self.last_span(), msg)
    }

    fn is_type_terminator(punct: &Punct) -> bool {
        let c = punct.as_char();
        matches!(c, ';' | ',' | '=')
    }

    pub(crate) fn parse_attributes(&mut self) -> TokenStream {
        let mut ts = TokenStream::new();

        while matches!(
            self.peekn(2),
            [tt0, tt1]
            if tt0.is_punct('#') && tt1.is_group(Delimiter::Bracket)
        ) {
            ts.extend(self.next());
            ts.extend(self.next());
        }

        ts
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

    /// Parses a type opaquely, stopping at `,` òr `;`.
    ///
    /// This doesn't handle None-delimited at all
    pub(crate) fn parse_opaque_type(&mut self) -> Result<OpaqueType, crate::Error> {
        let mut level = 0usize;

        let mut out = TokenStream::new();

        if self.is_empty() {
            return Err(self.error("expected type, found no tokens"));
        }

        let start_span = self.span();
        loop {
            let tt = self.peek();

            if let Some(TokenTree::Punct(punct)) = &tt {
                if level == 0 && Self::is_type_terminator(&punct) {
                    if out.is_empty() {
                        return Err(Error::with_span(
                            punct.span(),
                            "expected type, found no tokens",
                        ));
                    } else {
                        break;
                    };
                } else if punct.as_char() == '<' {
                    level += 1;
                } else if punct.as_char() == '>' {
                    level = level
                        .checked_sub(1)
                        .ok_or_else(|| Error::with_span(punct.span(), "unexpected '>'"))?;
                }
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

    pub(crate) fn assert_empty(&mut self) -> Result<(), crate::Error> {
        match self.peek() {
            Some(x) => Err(Error::with_span(x.span(), "expected no more tokens")),
            None => Ok(()),
        }
    }

    pub(crate) fn parse_punct(&mut self, c: char) -> Result<Punct, Error> {
        match self.next() {
            Some(TokenTree::Punct(x)) if x.as_char() == c => Ok(x),
            Some(x) => Err(Error::with_span(
                x.span(),
                &format!("expected a '{}' token", c),
            )),
            None => Err(Error::with_span(
                self.last_span(),
                &format!("expected a '{}' token", c),
            )),
        }
    }
    pub(crate) fn parse_opt_punct(&mut self, c: char) -> Result<Option<Punct>, Error> {
        match self.next() {
            Some(TokenTree::Punct(x)) if x.as_char() == c => Ok(Some(x)),
            Some(x) => Err(Error::with_span(
                x.span(),
                &format!("expected a '{}' token", c),
            )),
            None => Ok(None),
        }
    }

    pub(crate) fn peek_parse_punct(&mut self, c: char) -> Option<Punct> {
        match self.peek() {
            Some(TokenTree::Punct(x)) if x.as_char() == c => {
                if let Some(TokenTree::Punct(x)) = self.next() {
                    Some(x)
                } else {
                    unreachable!("{}", core::panic::Location::caller())
                }
            }
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn parse_ident(&mut self) -> Result<Ident, Error> {
        match self.next() {
            Some(TokenTree::Ident(x)) => Ok(x),
            x => Err(Error::with_span(
                self.span_or_last(x),
                "expected an identifier",
            )),
        }
    }

    pub(crate) fn parse_keyword(&mut self, keyword: &str) -> Result<Ident, Error> {
        match self.next() {
            Some(TokenTree::Ident(x)) if x.is_ident(keyword) => Ok(x),
            x => Err(Error::with_span(
                self.span_or_last(x),
                &format!("expected the `{}` keyword", keyword),
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