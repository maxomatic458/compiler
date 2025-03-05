use crate::{
    lexer::{
        position::Spanned,
        tokens::{Punctuation, Token},
    },
    parser::{
        ast::{Block, DataType, Expr, TypedExpr},
        error::ParserError,
        parser_main::Parser,
    },
};

impl Parser {
    pub(in crate::parser) fn parse_reference(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        self.expect_next(&[Token::Punctuation(Punctuation::Ampersand)])?;
        let expr = self.parse_expression(scope)?;

        Ok(Spanned {
            value: TypedExpr {
                expression: Expr::Reference {
                    value: Box::new(expr.clone()),
                },
                _type: DataType::Pointer(Box::new(expr.value._type.clone())),
                raw: None,
            },
            span: expr.span,
        })
    }
}
