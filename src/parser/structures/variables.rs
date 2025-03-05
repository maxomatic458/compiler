use crate::{
    lexer::{
        position::Spanned,
        tokens::{Keyword, Punctuation, Token},
    },
    parser::{
        ast::{Block, DataType, Expr, Statement, TypedExpr, Variable},
        error::ParserError,
        parser_main::Parser,
    },
};

impl Parser {
    pub(in crate::parser) fn parse_variable_decl(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<Statement>, Spanned<ParserError>> {
        let start = self.expect_next(&[Token::Keyword(Keyword::Let)])?.span;

        let mut next = self.expect_next(&[
            Token::Keyword(Keyword::Mut),
            Token::Identifier("variable Name".to_string()),
        ])?;
        let mut is_mutable = false;
        let mut _type = None;

        if Token::Keyword(Keyword::Mut) == next.value {
            is_mutable = true;
            next = self.expect_next(&[Token::Identifier("variable name".to_string())])?;
        }

        let name = Spanned {
            value: next.value.to_string(),
            span: next.span,
        };

        if let Some(Spanned {
            value: Token::Punctuation(Punctuation::Colon),
            ..
        }) = self.tokens.peek()
        {
            self.next_token()?;

            _type = Some(self.parse_data_type(Some(&scope.generics))?);
        }

        self.expect_next(&[Token::Assignment])?; // TODO ?
        let value = self.parse_expression(scope)?;

        match _type {
            None => {
                _type = Some(Spanned {
                    value: value.value._type.clone(),
                    span: Default::default(),
                })
            }

            Some(type_hint) if type_hint.value != value.value._type => {
                return Err(Spanned {
                    value: ParserError::WrongType(type_hint.value, value.value._type),
                    span: type_hint.span.extend(&value.span),
                })
            }

            _ => {}
        }

        if let Some(ref _type) = _type {
            if matches!(_type.value, DataType::None) {
                return Err(Spanned {
                    value: ParserError::VoidVariable,
                    span: name.span,
                });
            }
        }

        let span = start.extend(&value.span);

        scope.variables.insert(
            name.value.clone(),
            Spanned {
                value: Variable {
                    name: name.clone(),
                    is_mutable,
                    _type: _type.clone().unwrap().value,
                },
                span: name.span,
            },
        );

        Ok(Spanned {
            value: Statement::VariableDecl {
                is_mutable,
                name,
                _type,
                value,
            },
            span,
        })
    }

    pub(in crate::parser) fn parse_variable(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        if let Spanned {
            value: Token::Identifier(variable_name),
            span,
        } = self.expect_next(&[Token::Identifier("variable name".to_string())])?
        {
            let variable = scope.get_variable(&Spanned {
                value: variable_name.clone(),
                span,
            })?;

            return Ok(Spanned {
                value: TypedExpr {
                    expression: Expr::Variable(Variable {
                        name: Spanned {
                            value: variable_name,
                            span,
                        },
                        _type: variable.value._type.clone(),
                        is_mutable: variable.value.is_mutable,
                    }),
                    _type: variable.value._type,
                    raw: None,
                },
                span,
            });
        }

        unreachable!()
    }

    pub(in crate::parser) fn parse_variable_reassignment(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<Statement>, Spanned<ParserError>> {
        let to_mutate = self.parse_expression(scope)?;
        let base_var = Self::find_base_variable(&to_mutate)?;

        let variable = scope.get_variable(&Spanned {
            value: base_var.name.value,
            span: to_mutate.span,
        })?;

        if !variable.value.is_mutable {
            return Err(Spanned {
                value: ParserError::VariableNotMutable(variable.value.name.value.clone()),
                span: variable.span,
            });
        }

        let op = self.next_token()?;
        // TODO check ob es sich überhaupt um eine variable handelt
        if !op.value.is_reassignment_operator() {
            return Err(Spanned {
                value: ParserError::UnexpectedToken(op.value),
                span: op.span,
            });
        }

        let new_value = self.parse_expression(scope)?;

        if new_value.value._type != to_mutate.value._type {
            return Err(Spanned {
                value: ParserError::WrongType(base_var._type, new_value.value._type),
                span: op.span,
            });
        }

        Ok(Spanned {
            value: Statement::VariableMutation {
                variable: to_mutate.clone(),
                new_value: new_value.clone(),
            },
            span: to_mutate.span.extend(&new_value.span),
        })
    }

    fn find_base_variable(expr: &Spanned<TypedExpr>) -> Result<Variable, Spanned<ParserError>> {
        match &expr.value.expression {
            Expr::Variable(variable) => Ok(variable.clone()),
            Expr::Index { base, .. } | Expr::FieldAccess { base, .. } | Expr::Deref(base) => {
                if let Expr::Variable(variable) = &base.value.expression {
                    Ok(variable.clone())
                } else {
                    Self::find_base_variable(base)
                }
            }
            _ => Err(Spanned {
                value: ParserError::InvalidReassign,
                span: expr.span,
            }),
        }
    }
}
