use crate::{
    lexer::{
        position::Spanned,
        tokens::{Keyword, Token},
    },
    parser::{
        ast::{Block, DataType, Statement},
        error::ParserError,
        parser_main::Parser,
    },
};

impl Parser {
    pub(in crate::parser) fn parse_while(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<Statement>, Spanned<ParserError>> {
        scope.function_depth += 1;
        let start = self.expect_next(&[Token::Keyword(Keyword::While)])?.span;
        let condition = self.parse_expression(scope)?;

        if condition.value._type != DataType::Boolean {
            return Err(Spanned {
                value: ParserError::WrongType(DataType::Boolean, condition.value._type),
                span: start.extend(&condition.span),
            });
        }

        let body = self.parse_block(scope)?;
        let span = start.extend(&body.span);

        Ok(Spanned {
            value: Statement::WhileLoop {
                condition,
                body: Box::new(body),
            },
            span,
        })
    }
}
