use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use crate::{
    lexer::{
        position::Spanned,
        tokens::{Literal, Operator, Punctuation, Token},
    },
    parser::{
        ast::{
            Block, ClassLiteral, CommonGeneric, CustomDataType, DataType, DataTypeGetter,
            DataTypeInfo, Expr, FunctionParam, InternalNameGetter, TypedExpr,
        },
        error::ParserError,
        parser_main::Parser,
    },
};

impl Parser {
    pub fn parse_class_def(&mut self) -> Result<Spanned<DataType>, Spanned<ParserError>> {
        self.next_token()?; // "class"
        let mut generics = vec![];
        let mut is_generic = false;

        if let Spanned {
            value: Token::Identifier(name),
            span,
        } = self.expect_next(&[Token::Identifier("class name".to_string())])?
        {
            if let Token::Operator(Operator::LessThan) = self.peek()?.value {
                is_generic = true;
                generics = self.parse_generics()?.value;
            }

            let fields = self.parse_class_fields(&generics)?;

            let class_span = span.extend(&fields.span);

            if let Some(class_already_exists) = self.program.custom_types.get(&name).cloned() {
                return Err(Spanned {
                    value: ParserError::ClassAlreadyExists(name),
                    span: class_already_exists.span,
                });
            }

            let class = DataType::Custom(CustomDataType {
                display_name: name.clone(),
                name: name.clone(),
                fields,
                // methods: vec![],
                subtypes: HashMap::new(),
                generics,
                subtype_of: None,
                is_generic,
            });

            self.program.custom_types.insert(
                name,
                Spanned {
                    value: class.clone(),
                    span: class_span,
                },
            );

            self.program.data_types.insert(
                class.internal_name(),
                DataTypeInfo {
                    parent_type: class.clone(),
                    methods: vec![],
                    traits: HashSet::new(),
                },
            );

            return Ok(Spanned {
                value: class,
                span: class_span,
            });
        }

        unreachable!()
    }

    fn parse_class_fields(
        &mut self,
        generics: &Vec<Spanned<DataType>>,
    ) -> Result<Spanned<Vec<FunctionParam>>, Spanned<ParserError>> {
        let mut fields: Vec<FunctionParam> = vec![]; // TODO: hashmap
        let mut span = self
            .expect_next(&[Token::Punctuation(Punctuation::OpenBrace)])?
            .span;

        let end = self.walk_separated_values(
            Token::Punctuation(Punctuation::Comma),
            Token::Punctuation(Punctuation::CloseBrace),
            |parser| {
                if let Spanned {
                    value: Token::Identifier(field_name),
                    span: field_name_span,
                } = parser.expect_next(&[Token::Identifier("class field name".to_string())])?
                {
                    parser.expect_next(&[Token::Punctuation(Punctuation::Colon)])?;
                    let data_type = parser.parse_data_type(Some(generics))?;

                    if fields.iter().any(|f| f.name.value == field_name) {
                        return Err(Spanned {
                            value: ParserError::FieldNameAlreadyExists(field_name),
                            span: field_name_span,
                        });
                    }

                    fields.push(FunctionParam {
                        name: Spanned {
                            value: field_name,
                            span: field_name_span,
                        },
                        _type: data_type,
                    });
                }

                Ok(())
            },
        )?;

        span = span.extend(&end);

        Ok(Spanned {
            value: fields,
            span,
        })
    }

    pub(in crate::parser) fn parse_class_literal(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        if let Spanned {
            value: Token::Identifier(class_name),
            span,
        } = self.expect_next(&[Token::Identifier("class name".to_string())])?
        {
            let mut span = span;

            let mut generic_annotations = vec![];
            let block_start = match self.peek()?.value {
                Token::Operator(Operator::LessThan) => {
                    generic_annotations =
                        self.collect_generic_annotations(Some(&scope.generics))?;
                    self.next_token()?.span
                }
                Token::Punctuation(Punctuation::OpenBrace) => self.next_token()?.span,
                _ => {
                    self.expect_next(&[
                        Token::Punctuation(Punctuation::OpenBrace),
                        Token::Operator(Operator::LessThan),
                    ])?;
                    unreachable!()
                }
            };

            let mut class = self
                .get_type_from_name(&Spanned {
                    value: class_name,
                    span,
                })?
                .clone();

            // panic!("{:?}", class.value._type());

            let mut fields: Vec<(Spanned<String>, Spanned<TypedExpr>)> = vec![];

            let end = self.walk_separated_values(
                Token::Punctuation(Punctuation::Comma),
                Token::Punctuation(Punctuation::CloseBrace),
                |parser| {
                    if let Spanned {
                        value: Token::Identifier(field_name),
                        span: field_name_span,
                    } = parser.expect_next(&[Token::Identifier("field name".to_string())])?
                    {
                        parser.expect_next(&[Token::Punctuation(Punctuation::Colon)])?;
                        let field_value = parser.parse_expression(scope)?;

                        block_start.extend(&field_value.span);

                        fields.push((
                            Spanned {
                                value: field_name,
                                span: field_name_span,
                            },
                            field_value,
                        ));
                    }

                    Ok(())
                },
            )?;

            if let DataType::Custom(ref mut custom_type) = class.value {
                generic_annotations = if !generic_annotations.is_empty() {
                    generic_annotations.clone()
                } else {
                    // generics ermitteln
                    fields
                        .iter()
                        .map(|(_, field)| Spanned {
                            value: field.value._type.clone(),
                            span: field.span,
                        })
                        .collect()
                };
                // if generic_annotations.is_empty() && custom_type.is_generic() {
                //     // return Err(Spanned {
                //     //     value: ParserError::MissingTypeA
                //     // })
                //     panic!("");
                // }

                if custom_type.is_generic() {
                    // panic!("fdf");
                    let mut unspanned_specific_types: Vec<DataType> = generic_annotations
                        .iter()
                        .map(|x| x.value.clone())
                        .collect();
                    unspanned_specific_types.append(
                        &mut scope
                            .generics
                            .clone()
                            .into_iter()
                            .map(|x| x.value)
                            .collect(),
                    );

                    let subtype = match custom_type.subtypes.get(&unspanned_specific_types) {
                        Some(subtype) => subtype.clone(),
                        None => {
                            // let _custom_type_ref =
                            //     self.program.custom_types.get_mut(&custom_type.name).unwrap();
                            custom_type.subtype(&unspanned_specific_types, &mut self.program, true)
                        }
                    };
                    // panic!("{}", subtype);
                    class = Spanned {
                        value: DataType::Custom(subtype),
                        span: class.span,
                    }
                }
            }

            // panic!("f");

            span = span.extend(&end);

            if let DataType::Custom(CustomDataType {
                fields: defined_fields,
                ..
            }) = &class.value
            {
                for (param, arg) in defined_fields.value.iter().zip(fields.iter()) {
                    if param.name.value != arg.0.value {
                        return Err(Spanned {
                            value: ParserError::WrongClassFields(
                                defined_fields
                                    .value
                                    .iter()
                                    .map(|param| param.name.value.clone())
                                    .collect(),
                                fields.iter().map(|arg| arg.0.value.clone()).collect(),
                            ),
                            span: block_start,
                        });
                    }

                    if param._type.value != arg.1.value._type {
                        return Err(Spanned {
                            value: ParserError::WrongType(
                                param._type.value.clone(),
                                arg.1.value._type.clone(),
                            ),
                            span: arg.1.span,
                        });
                    }
                }

                if defined_fields.value.len() != fields.len() {
                    return Err(Spanned {
                        value: ParserError::WrongClassFields(
                            defined_fields
                                .value
                                .iter()
                                .map(|param| param.name.value.clone())
                                .collect(),
                            fields.iter().map(|(name, _)| name.value.clone()).collect(),
                        ),
                        span: block_start,
                    });
                }
            }

            return Ok(Spanned {
                value: TypedExpr {
                    expression: Expr::Literal(Literal::Custom(ClassLiteral {
                        _type: class.value._type(),
                        fields: Spanned {
                            value: fields,
                            span: block_start,
                        },
                    })),
                    _type: class.value._type(),
                    raw: None,
                },
                span,
            });
        }

        unreachable!()
    }

    pub(in crate::parser) fn parse_field_access(
        &mut self,
        base: &Spanned<TypedExpr>,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        self.expect_next(&[Token::Punctuation(Punctuation::Period)])?;
        let class = base.value._type.clone();
        if let DataType::Custom(CustomDataType {
            display_name: name,
            fields,
            ..
        }) = &class
        {
            if let Spanned {
                value: Token::Identifier(field_name),
                span,
            } = self.expect_next(&[Token::Identifier("field name".to_string())])?
            {
                if let Some(field) = fields
                    .value
                    .iter()
                    .find_position(|f| f.name.value == field_name)
                {
                    return Ok(Spanned {
                        value: TypedExpr {
                            expression: Expr::FieldAccess {
                                base: Box::new(base.clone()),
                                field: Spanned {
                                    value: field_name,
                                    span,
                                },
                                field_idx: field.0,
                            },
                            _type: field.1._type.value.clone(),
                            raw: None,
                        },
                        span: base.span.extend(&span),
                    });
                } else {
                    return Err(Spanned {
                        value: ParserError::ClassFieldDoesNotExist(field_name, name.clone()),
                        span,
                    });
                }
            }
        }

        Err(Spanned {
            value: ParserError::CannotAccessFields(class),
            span: base.span,
        })
    }

    pub(in crate::parser) fn parse_generics(
        &mut self,
    ) -> Result<Spanned<Vec<Spanned<DataType>>>, Spanned<ParserError>> {
        let mut span = self.next_token()?.span; // skip "<"
        let mut generics = vec![];

        let end = self.walk_separated_values(
            Token::Punctuation(Punctuation::Comma),
            Token::Operator(Operator::GreaterThan),
            |parser| {
                if let Spanned {
                    value: Token::Identifier(generic_name),
                    span: generic_span,
                } = parser.expect_next(&[Token::Identifier("generic name".to_string())])?
                {
                    let generic = Spanned {
                        value: DataType::Generic(generic_name),
                        span: generic_span,
                    };

                    span = span.extend(&generic.span);
                    generics.push(generic)
                }

                Ok(())
            },
        )?;

        span = span.extend(&end);

        Ok(Spanned {
            value: generics,
            span,
        })
    }
}
