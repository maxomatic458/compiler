use crate::{
    lexer::{
        position::Spanned,
        tokens::{Literal, Punctuation, Token},
    },
    parser::{
        ast::{ArrayLiteral, Block, DataType, Expr, Trait, TypedExpr},
        error::ParserError,
        parser_main::Parser,
        utils::check_all_types_same,
    },
};

impl Parser {
    pub(in crate::parser) fn parse_array_literal(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        let mut elements = vec![];
        let mut span = self
            .expect_next(&[Token::Punctuation(Punctuation::OpenBracket)])?
            .span;

        let end = self.walk_separated_values(
            Token::Punctuation(Punctuation::Comma),
            Token::Punctuation(Punctuation::CloseBracket),
            |parser| {
                elements.push(parser.parse_expression(scope)?);

                Ok(())
            },
        )?;

        span = span.extend(&end);

        if elements.is_empty() {
            return Err(Spanned {
                value: ParserError::EmptyArray,
                span,
            });
        };

        let _type = check_all_types_same(&elements)?;

        Ok(Spanned {
            value: TypedExpr {
                expression: Expr::Literal(Literal::ArrayLiteral(ArrayLiteral {
                    value_type: _type.clone(),
                    values: Spanned {
                        value: elements.clone(),
                        span,
                    },
                })),
                _type: DataType::Array {
                    value_type: Box::new(_type),
                    len: elements.len(),
                },
                raw: None,
            },
            span,
        })
    }

    pub(in crate::parser) fn parse_indexing(
        &mut self,
        base: &Spanned<TypedExpr>,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        let _idx_start = self
            .expect_next(&[Token::Punctuation(Punctuation::OpenBracket)])?
            .span;
        let idx = self.parse_expression(scope)?;
        let idx_stop = self
            .expect_next(&[Token::Punctuation(Punctuation::CloseBracket)])?
            .span;

        if !self.implements_trait(
            &base.value._type,
            &Trait::Index,
            &[base.value._type.clone(), idx.value._type.clone()],
        ) {
            return Err(Spanned {
                value: ParserError::IndexError(base.value._type.clone()),
                span: base.span.extend(&idx_stop),
            });
        }

        let type_info = self.get_type_info(&base.value._type);
        // panic!("type_info: {:?}", type_info);
        // sollte nicht panicen, siehe check oben
        let result_type = {
            let result_type = type_info
                .get_trait_return_type(
                    &Trait::Index,
                    &[base.value._type.clone(), idx.value._type.clone()],
                )
                .unwrap();

            // eigene Implementierung, muss einen pointer zurückgeben
            // um den syntax einfacher zu halten, wird dieser wie ein normaler wert behandelt
            if type_info
                .get_trait_override_function_name(
                    &Trait::Index,
                    &[base.value._type.clone(), idx.value._type.clone()],
                )
                .is_some()
            {
                if let DataType::Pointer(value_type) = result_type {
                    *value_type
                } else {
                    unreachable!()
                }
            } else {
                result_type
            }
        };

        Ok(Spanned {
            value: TypedExpr {
                expression: Expr::Index {
                    base: Box::new(base.clone()),
                    idx: Box::new(idx.clone()),
                },
                _type: result_type,
                raw: None,
            },
            span: base.span.extend(&idx_stop),
        })
    }
}
