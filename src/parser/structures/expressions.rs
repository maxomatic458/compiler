use crate::{
    lexer::{
        position::{Span, Spanned},
        tokens::{Keyword, Literal, Operator, Punctuation, Token},
    },
    parser::{
        ast::{
            BinaryOperator, Block, DataType, DataTypeGetter, Expr, Trait, TypedExpr, UnaryOperator,
        },
        error::ParserError,
        parser_main::Parser,
    },
};
use std::str::FromStr;

impl Parser {
    pub fn parse_expression(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        let lhs = self.parse_primary_expression(None, scope)?;
        self.parse_binary_expression(lhs, 0, scope)
    }

    fn parse_primary_expression(
        &mut self,
        previous: Option<Spanned<TypedExpr>>,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        let next = self.peek()?;
        let expr: Result<Spanned<TypedExpr>, Spanned<ParserError>>;

        if let Some(previous) = previous {
            expr = Ok(previous);
        } else {
            expr = match next.value {
                Token::Punctuation(Punctuation::Ampersand) => self.parse_reference(scope),
                Token::Operator(_) => self.parse_unary_expression(scope),
                Token::Punctuation(Punctuation::OpenBrace) => {
                    let block = self.parse_block(scope)?;

                    Ok(Spanned {
                        value: TypedExpr {
                            expression: Expr::Block {
                                body: block.clone(),
                            },
                            _type: block.value.return_type,
                            raw: None,
                        },
                        span: block.span,
                    })
                }
                Token::Punctuation(Punctuation::OpenParen) => {
                    self.parse_parenthized_expression(scope)
                }
                Token::DataLiteral(_data_literal) => self.parse_literal(),
                Token::Punctuation(Punctuation::Tilde) => self.parse_deref(scope),
                Token::Identifier(_name) => {
                    let next = self.peek_nth(1)?;

                    match next.value {
                        Token::Punctuation(Punctuation::OpenParen) => {
                            self.parse_func_call(scope, None)
                        }
                        Token::Operator(Operator::LessThan)
                            if self
                                .find_ahead(vec![Token::Operator(Operator::GreaterThan)], |t| {
                                    matches!(
                                        t,
                                        Token::Punctuation(Punctuation::SemiColon)
                                            | Token::Punctuation(Punctuation::CloseBrace)
                                            | Token::Punctuation(Punctuation::OpenBrace) // | Token::Operator(Operator::LessThan)
                                    )
                                })?
                                .is_some() =>
                        {
                            let contains_open_brace = self
                                .find_ahead(
                                    vec![Token::Punctuation(Punctuation::OpenBrace)],
                                    |t| {
                                        matches!(
                                            t,
                                            Token::Punctuation(Punctuation::SemiColon)
                                                | Token::Punctuation(Punctuation::CloseBrace)
                                        )
                                    },
                                )?
                                .is_some();

                            if !contains_open_brace {
                                // panic!("yes");
                                self.parse_func_call(scope, None)
                            } else {
                                self.parse_class_literal(scope)
                            }
                        }
                        Token::Punctuation(Punctuation::OpenBrace)
                            if self.program.custom_types.contains_key(&_name) =>
                        {
                            self.parse_class_literal(scope)
                        }
                        // statische methode
                        Token::Punctuation(Punctuation::Colon)
                            if self.program.custom_types.contains_key(&_name) =>
                        {
                            self.next_token()?;
                            self.expect_next(&[Token::Punctuation(Punctuation::Colon)])?;
                            self.expect_next(&[Token::Punctuation(Punctuation::Colon)])?;

                            let data_type =
                                self.program.custom_types.get(&_name).unwrap().value.clone();

                            self.parse_func_call(
                                scope,
                                Some(&Spanned {
                                    value: TypedExpr {
                                        expression: Expr::ClassName(_name),
                                        _type: data_type,
                                        raw: None,
                                    },
                                    span: Span::default(),
                                }),
                            )
                        }

                        _ => {
                            if self.program.custom_types.get(&_name).is_some()
                                || DataType::from_str(&_name).is_ok()
                                || scope.generics.iter().any(|g| {
                                    if let DataType::Generic(name) = &g.value {
                                        *name == *_name
                                    } else {
                                        false
                                    }
                                })
                            {
                                self.parse_data_type_literal(scope)
                            } else {
                                self.parse_variable(scope)
                            }
                        }
                    }
                }

                Token::Punctuation(Punctuation::OpenBracket) => self.parse_array_literal(scope),

                _ => self.parse_macro(scope),
            };
        }

        let mut expr = expr?;
        // index
        if let Ok(Spanned {
            value: Token::Punctuation(Punctuation::OpenBracket),
            ..
        }) = self.peek()
        {
            // check in parse_indexing
            let index_expr = self.parse_indexing(&expr, scope)?;
            expr = self.parse_primary_expression(Some(index_expr), scope)?;
        }
        // methode
        if let Ok(Spanned {
            value: Token::Punctuation(Punctuation::Period),
            ..
        }) = self.peek()
        {
            // panic!("eeeeee");
            // if let DataType::Custom { .. } = expr.value._type {

            // let is_call = self.find_ahead(
            //     vec![Token::Punctuation(Punctuation::OpenParen)],
            //     vec![Token::Punctuation(Punctuation::SemiColon), Token::Punctuation(Punctuation::CloseBrace), Token::Punctuation(Punctuation::OpenBrace)])?.is_some();
            // let is_call = self.peek_nth(2)?.value == Token::Punctuation(Punctuation::OpenParen);

            let is_call = self.peek()?.value == Token::Punctuation(Punctuation::Period)
                && (self.peek_nth(2)?.value == Token::Punctuation(Punctuation::OpenParen)
                    || self.peek_nth(2)?.value == Token::Operator(Operator::LessThan));

            // let is_call = self
            //     .find_ahead(vec![Token::Punctuation(Punctuation::OpenParen)], |t| {
            //         if matches!(
            //             t,
            //             Token::Operator(Operator::LessThan)
            //                 | Token::Operator(Operator::GreaterThan)
            //         ) {
            //             return false;
            //         }

            //         matches!(
            //             t,
            //             Token::Punctuation(Punctuation::SemiColon)
            //                 | Token::Operator(_)
            //                 | Token::Punctuation(Punctuation::Colon)
            //                 | Token::Punctuation(Punctuation::OpenBrace)
            //                 | Token::Assignment
            //         )
            //     })?
            //     .is_some();

            if is_call {
                self.next_token()?;
                let function = self.parse_func_call(scope, Some(&expr))?;
                expr = self.parse_primary_expression(Some(function), scope)?;
            } else {
                let field_access = self.parse_field_access(&expr)?;
                expr = self.parse_primary_expression(Some(field_access), scope)?;
            }
        }

        // type cast
        if let Ok(Spanned {
            value: Token::Keyword(Keyword::As),
            ..
        }) = self.peek()
        {
            let type_cast = self.parse_type_cast(expr.clone(), scope)?;
            expr = self.parse_primary_expression(Some(type_cast), scope)?;
        }

        Ok(expr)
    }
    // https://en.wikipedia.org/wiki/Operator-precedence_parser
    fn parse_binary_expression(
        &mut self,
        mut lhs: Spanned<TypedExpr>,
        min_precedence: u8,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        while let Ok(Spanned {
            value: Token::Operator(first_operator),
            span,
        }) = self.peek()
        {
            // let mut result_type = lhs.value._type.clone();
            let first_operator = match first_operator.to_binary_op() {
                Some(op) => op,
                None => {
                    return Err(Spanned {
                        value: ParserError::InvalidOperator(first_operator),
                        span,
                    })
                }
            };

            if first_operator.precedence() < min_precedence {
                break;
            }

            self.tokens.next();
            let mut rhs = self.parse_primary_expression(None, scope)?;

            let mut expr_span = lhs.span.extend(&rhs.span);

            let operation_trait = Trait::from_binary_operator(&first_operator);

            while let Ok(Spanned {
                value: Token::Operator(second_operator),
                ..
            }) = self.peek()
            {
                let second_operator = match second_operator.to_binary_op() {
                    Some(op) => op,
                    None => {
                        return Err(Spanned {
                            value: ParserError::InvalidOperator(second_operator),
                            span,
                        })
                    }
                };

                if second_operator.precedence() <= first_operator.precedence() {
                    break;
                }

                let mut next_precedence = first_operator.precedence();

                if second_operator.precedence() > first_operator.precedence() {
                    next_precedence += 1;
                }

                rhs = self.parse_binary_expression(rhs, next_precedence, scope)?;
            }

            if !self.implements_trait(
                &lhs.value._type,
                &operation_trait,
                &[lhs.value._type.clone(), rhs.value._type.clone()],
            ) {
                return Err(Spanned {
                    value: ParserError::WrongType(lhs.value._type, rhs.value._type), // TODO: anderer fehler? trait not impl?
                    span: expr_span,
                });
            }

            let type_info = self.get_type_info(&lhs.value._type);
            // sollte nicht panicen, siehe check oben
            let result_type = type_info
                .get_trait_return_type(
                    &operation_trait,
                    &[lhs.value._type.clone(), rhs.value._type.clone()],
                )
                .unwrap();

            expr_span = lhs.span.extend(&rhs.span);

            lhs = Spanned {
                value: TypedExpr {
                    expression: Expr::Binary {
                        lhs: Box::new(lhs.clone()),
                        op: Spanned {
                            value: first_operator.clone(),
                            span,
                        },
                        rhs: Box::new(rhs.clone()),
                    },
                    _type: result_type,
                    raw: None,
                },
                span: expr_span,
            };
        }

        // if let Expr::Binary { op, .. } = &lhs.value.expression {
        //     if op.value.is_ordering() {
        //         lhs.value._type = DataType::Boolean
        //     }
        // }

        Ok(lhs)
    }

    fn parse_unary_expression(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        // unary operator konsumieren
        if let Spanned {
            value: Token::Operator(operator),
            span,
        } = self.expect_next(&[Token::Operator(Operator::default())])?
        {
            let unary = match operator.to_unary_op() {
                Some(operator) => operator,
                None => {
                    return Err(Spanned {
                        value: ParserError::InvalidOperator(operator),
                        span,
                    })
                }
            };

            let expr = self.parse_expression(scope)?;
            let expr_type = expr.value._type.clone();

            match unary {
                // TODO: vielleicht zu codegen https://llvm.org/docs/LangRef.html#i-fneg
                UnaryOperator::Minus => {
                    if expr_type.is_integer() || expr_type.is_float() {
                        return Ok(Spanned {
                            value: TypedExpr {
                                expression: Expr::Binary {
                                    lhs: Box::new(Spanned {
                                        value: TypedExpr {
                                            expression: Expr::Literal(if expr_type.is_integer() {
                                                Literal::Integer(-1)
                                            } else {
                                                Literal::Float(-1.)
                                            }),
                                            _type: expr_type.clone(),
                                            raw: None,
                                        },
                                        span: Default::default(),
                                    }),
                                    rhs: Box::new(expr),
                                    op: Spanned {
                                        value: BinaryOperator::Multiply,
                                        span: Default::default(),
                                    },
                                },
                                _type: expr_type,
                                raw: None,
                            },
                            span: Default::default(),
                        });
                    }
                }
                UnaryOperator::Not => todo!(),
            }
            unreachable!()
        }
        todo!()
    }

    fn parse_parenthized_expression(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        self.expect_next(&[Token::Punctuation(Punctuation::OpenParen)])?;
        let expr = self.parse_expression(scope)?;
        self.expect_next(&[Token::Punctuation(Punctuation::CloseParen)])?;
        Ok(expr)
    }

    fn parse_literal(&mut self) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        if let Spanned {
            value: Token::DataLiteral(literal),
            span,
        } = self.next_token()?
        {
            return Ok(Spanned {
                value: TypedExpr {
                    expression: Expr::Literal(literal.clone()),
                    _type: literal._type(),
                    raw: None,
                },
                span,
            });
        }

        unreachable!()
    }

    fn parse_data_type_literal(
        &mut self,
        scope: &Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        if let Spanned {
            value: Token::Identifier(literal),
            span,
        } = self.next_token()?
        {
            let mut _type = None;

            _type = self
                .program
                .custom_types
                .get(&literal)
                .map(|c| c.value.clone());

            if _type.is_none() {
                // println!("{:?}", literal);
                _type = DataType::from_str(&literal).ok();
            }

            if _type.is_none() {
                _type = scope
                    .generics
                    .iter()
                    .find(|g| {
                        if let DataType::Generic(name) = &g.value {
                            name == &literal
                        } else {
                            false
                        }
                    })
                    .map(|g| g.value.clone());
            }

            return Ok(Spanned {
                value: TypedExpr {
                    expression: Expr::Literal(Literal::DataType {
                        value_type: Box::new(_type.unwrap()),
                    }),
                    _type: DataType::DataType,
                    raw: None,
                },
                span,
            });
        }

        unreachable!()
    }

    fn parse_deref(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        self.expect_next(&[Token::Punctuation(Punctuation::Tilde)])?;

        let mut expr = self.parse_expression(scope)?;
        if let DataType::Pointer(value_type) = expr.value._type {
            expr.value._type = *value_type;
        } else {
            return Err(Spanned {
                value: ParserError::CannotDerefType(expr.value._type),
                span: expr.span,
            });
        }

        Ok(Spanned {
            value: TypedExpr {
                expression: Expr::Deref(Box::new(expr.clone())),
                _type: expr.value._type,
                raw: None,
            },
            span: expr.span,
        })
    }
}
