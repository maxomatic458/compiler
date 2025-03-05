use crate::{
    lexer::{
        position::Spanned,
        tokens::{Keyword, Token},
    },
    parser::{
        ast::{Block, DataType, Expr, TypedExpr},
        error::ParserError,
        parser_main::Parser,
    },
};

impl Parser {
    pub(in crate::parser) fn parse_type_cast(
        &mut self,
        base: Spanned<TypedExpr>,
        scope: &Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        self.expect_next(&[Token::Keyword(Keyword::As)])?;
        let cast_to = self.parse_data_type(Some(&scope.generics))?;

        // wahrscheinlich entfernen TODO
        let _base_type_info = self.get_type_info(&base.value._type);

        if DataType::Pointer(Box::new(base.value._type.clone())) == cast_to.value
            || base.value._type.can_be_converted_to(&cast_to.value)
        // TODO: traits
        // || base_type_info.implements_trait(&Trait::Cast, &[base.value._type.clone(), cast_to.value.clone()])
        {
            return Ok(Spanned {
                value: TypedExpr {
                    expression: Expr::Cast {
                        value: Box::new(base.clone()),
                        to_type: cast_to.clone(),
                    },
                    _type: cast_to.value,
                    raw: None,
                },
                span: base.span.extend(&cast_to.span),
            });
        }

        Err(Spanned {
            value: ParserError::InvalidCast(base.value._type, cast_to.value),
            span: base.span.extend(&cast_to.span),
        })
    }
}
