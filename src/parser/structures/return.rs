use crate::{
    lexer::{
        position::Spanned,
        tokens::{Keyword, Token},
    },
    parser::{
        ast::{Block, Statement},
        error::ParserError,
        parser_main::Parser,
    },
};

impl Parser {
    pub(in crate::parser) fn parse_return(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<Statement>, Spanned<ParserError>> {
        let start = self.expect_next(&[Token::Keyword(Keyword::Return)])?.span;
        let value = self.parse_expression(scope)?;

        let span = start.extend(&value.span);

        Ok(Spanned {
            value: Statement::Return { value },
            span,
        })
    }
}
