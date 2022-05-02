use used_proc_macro::TokenStream;

use crate::{
    syntax::{Crate, FieldIdent},
    Error,
};

pub(crate) fn field_macro_impl(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let input = &mut crate::parsing::ParseBuffer::new(input);
    let crate_kw = Crate::parse(input).unwrap();
    (|| -> Result<TokenStream, Error> {
        let mut out = TokenStream::new();
        let field_ident = FieldIdent::parse(input)?;
        input.assert_empty()?;
        field_ident.to_token_stream(&crate_kw, &mut out);

        Ok(out)
    })()
    .map_err(|e| e.to_compile_error(&crate_kw))
}
