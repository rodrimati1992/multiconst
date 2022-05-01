use used_proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree};

use alloc::{
    collections::VecDeque,
    string::{String, ToString},
};

use core::iter::{FromIterator, Fuse};

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub(crate) struct PeekableN<I: Iterator> {
    iter: Fuse<I>,
    queue: VecDeque<I::Item>,
}

impl PeekableN<core::ops::Range<u8>> {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new<I: IntoIterator>(iter: I) -> PeekableN<I::IntoIter> {
        PeekableN {
            iter: iter.into_iter().fuse(),
            queue: VecDeque::new(),
        }
    }
}

impl<I: Iterator> PeekableN<I> {
    pub(crate) fn peek(&mut self) -> Option<&I::Item> {
        if self.queue.is_empty() {
            self.queue.push_back(self.iter.next()?);
        }
        Some(&self.queue[0])
    }
    pub(crate) fn peekn(&mut self, n: usize) -> &[I::Item] {
        while self.queue.len() < n {
            match self.iter.next() {
                Some(x) => self.queue.push_back(x),
                None => break,
            }
        }

        let end = core::cmp::min(self.queue.len(), n);
        &self.queue.make_contiguous()[..end]
    }
}

impl<I> Iterator for PeekableN<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        if let opt @ Some(_) = self.queue.pop_front() {
            opt
        } else {
            self.iter.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let len = self.queue.len();
        (low + len, high.map(|x| x.saturating_add(len)))
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) fn ident_to_string_no_raw(ident: &Ident) -> String {
    let mut string = ident.to_string();
    if string.starts_with("r#") {
        string.drain(..2);
    }
    string
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) trait TokenStreamExt: Sized {
    fn as_mut_token_stream(&mut self) -> &mut TokenStream;

    fn from_array<T, const N: usize>(array: [T; N]) -> Self
    where
        T: Into<TokenTree>;

    #[inline(always)]
    fn append_array<T, const N: usize>(&mut self, array: [T; N])
    where
        T: Into<TokenTree>,
    {
        // msrv is 1.51.0, before [T; N] impld IntoIterator
        #![allow(deprecated)]
        self.as_mut_token_stream()
            .extend(core::array::IntoIter::new(array).map(Into::<TokenTree>::into))
    }

    #[inline(always)]
    fn append_one<T>(&mut self, elem: T)
    where
        T: Into<TokenTree>,
    {
        let tt: TokenTree = elem.into();
        self.as_mut_token_stream().extend(core::iter::once(tt))
    }

    fn append_keyword(&mut self, kw: &str, span: Span) {
        self.append_one(Ident::new(kw, span))
    }
}

impl TokenStreamExt for TokenStream {
    fn from_array<T, const N: usize>(array: [T; N]) -> Self
    where
        T: Into<TokenTree>,
    {
        // msrv is 1.51.0, before [T; N] impld IntoIterator
        #![allow(deprecated)]

        TokenStream::from_iter(core::array::IntoIter::new(array).map(Into::<TokenTree>::into))
    }

    #[inline(always)]
    fn as_mut_token_stream(&mut self) -> &mut TokenStream {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) trait TokenTreeExt: Sized {
    fn as_token_tree(&self) -> &TokenTree;
    fn into_token_tree(self) -> TokenTree;

    fn is_punct(&self, c: char) -> bool {
        matches!(self.as_token_tree(), TokenTree::Punct(p)  if p.as_char() == c)
    }

    fn is_group(&self, delim: Delimiter) -> bool {
        matches!(
            self.as_token_tree(),
            TokenTree::Group(g) if g.delimiter() == delim
        )
    }
}

impl TokenTreeExt for TokenTree {
    fn as_token_tree(&self) -> &TokenTree {
        self
    }

    fn into_token_tree(self) -> TokenTree {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) trait WithSpan {
    // only changes where error messages are printed
    fn with_span(self, span: Span) -> Self;
}

macro_rules! impl_with_span {
    ($ty:ty) => {
        impl WithSpan for $ty {
            fn with_span(mut self, span: Span) -> Self {
                let old_span = self.span();
                self.set_span(old_span.located_at(span));
                self
            }
        }
    };
}
impl_with_span! {::used_proc_macro::Group}
impl_with_span! {::used_proc_macro::Ident}
impl_with_span! {::used_proc_macro::Literal}
impl_with_span! {::used_proc_macro::Punct}
impl_with_span! {::used_proc_macro::TokenTree}

impl WithSpan for TokenStream {
    fn with_span(self, span: Span) -> TokenStream {
        self.into_iter().map(|tt| tt.with_span(span)).collect()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) trait IsIdent: Sized {
    fn is_ident(&self, ident: &str) -> bool;

    fn which_ident_in(&self, idents: &[&str]) -> Option<usize>;
}

impl IsIdent for Ident {
    fn is_ident(&self, ident: &str) -> bool {
        ident_to_string_no_raw(self) == ident
    }

    fn which_ident_in(&self, idents: &[&str]) -> Option<usize> {
        let this = ident_to_string_no_raw(self);

        idents.iter().position(|x| **x == this)
    }
}

impl IsIdent for TokenTree {
    fn is_ident(&self, ident: &str) -> bool {
        matches!(
            self.as_token_tree(),
            TokenTree::Ident(x)
            if x.is_ident(ident)
        )
    }
    fn which_ident_in(&self, idents: &[&str]) -> Option<usize> {
        match self.as_token_tree() {
            TokenTree::Ident(x) => x.which_ident_in(idents),
            _ => None,
        }
    }
}
