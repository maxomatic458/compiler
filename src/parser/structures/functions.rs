use crate::{
    lexer::{
        position::{Span, Spanned},
        tokens::{Keyword, Operator, Punctuation, Token},
    },
    parser::{
        ast::{
            Block, CallArg, CommonGeneric, DataType, Expr, Function, FunctionParam,
            InternalNameGetter, TypedExpr, Variable, CLASS_SELF_ARG_NAME, TRAIT_NAMES_MAP,
        },
        error::ParserError,
        parser_main::Parser,
        utils::handle_generics,
        // utils::,
    },
};
use ahash::AHashMap;
use itertools::Itertools;

use std::collections::BTreeMap;

impl Parser {
    pub fn parse_func_def(&mut self) -> Result<Spanned<Function>, Spanned<ParserError>> {
        let mut is_extern = false;
        let mut parent_class = None;
        let mut generics = vec![];

        let start = self.next_token()?; // def/extern

        if let Token::Keyword(Keyword::Extern) = start.value {
            is_extern = true;
            self.next_token()?; // def
        }

        if let Spanned {
            value: Token::Identifier(name),
            span,
        } = self.expect_next(&[Token::Identifier("function name".to_string())])?
        {
            let span = start.span.extend(&span);

            if Token::Operator(Operator::LessThan) == self.peek()?.value {
                generics = self.parse_generics()?.value;
            }

            let mut params = self.parse_func_params(&generics)?;

            let next = self.peek()?;

            match next.value {
                Token::Keyword(Keyword::For) => {
                    // method
                    self.next_token()?;
                    parent_class = Some(self.parse_data_type(Some(&generics))?);
                    // self parameter
                    if let Some(self_param) = params.value.get_mut(0) {
                        if self_param.value.name.value == *CLASS_SELF_ARG_NAME {
                            self_param.value._type.value = parent_class.clone().unwrap().value;
                        }
                    }
                }
                Token::Punctuation(Punctuation::ThinArrow)
                | Token::Punctuation(Punctuation::OpenBrace) => {}

                _ => unreachable!(),
            }

            let return_type = self.parse_func_return_type(&generics)?.unwrap_or_default();

            let mut variables = BTreeMap::new();

            // TODO: verschieben nach parse_func_params
            for param in params.value.iter_mut() {
                let converted_to_spanned_var: Spanned<Variable> = Spanned {
                    value: param.value.clone().into(),
                    span: param.span,
                };

                if let Some(already_exists) =
                    variables.insert(param.value.name.value.clone(), converted_to_spanned_var)
                {
                    return Err(Spanned {
                        value: ParserError::ParamNameAlreadyExists(param.value.name.value.clone()),
                        span: already_exists.span,
                    });
                }
            }
            // panic!("generics: {:?}", generics);
            let mut function = Function {
                display_name: Spanned {
                    value: name.clone(),
                    span,
                },
                name: name.clone(),
                params,
                body: Spanned {
                    value: Block {
                        statements: vec![],
                        variables,
                        closure_params: vec![],
                        generics,
                        return_type: return_type.value.clone(),
                        function_depth: 0,
                    },
                    span: Span::default(),
                },
                return_type,
                is_extern,
                method_of: parent_class.clone().map(|c| c.value),
                generic_subtypes: AHashMap::new(),
                is_builtin: false,
                trait_of: None,
            };

            // if let Some(class) = &parent_class {
            //     println!("class: {:?}", class.value);
            // }

            match parent_class.as_ref().map(|c| &c.value) {
                Some(inner) => {
                    let data_type_info = self.get_type_info_mut(inner);

                    if data_type_info.methods.contains(&function.name) {
                        return Err(Spanned {
                            value: ParserError::MethodAlreadyExists(name, inner.to_string()),
                            span,
                        });
                    }

                    let mut function_name = function.name.clone();

                    // Funktion hier noch nicht vollständig, ist aber egal wahrscheinlich?
                    // für trait ist eigentlich nur signatur wichtig, vielleicht function.signature()
                    if let Some(trait_) = TRAIT_NAMES_MAP.get(&function.name) {
                        if function.is_static_method() {
                            return Err(Spanned {
                                value: ParserError::TraitIsStaticMethod,
                                span,
                            });
                        }

                        let param_types: Vec<DataType> = function
                            .params
                            .value
                            .iter()
                            .skip(1) // skip entfernen wegen generics?
                            .map(|p| p.value._type.value.clone())
                            .collect();

                        if data_type_info
                            .get_trait_override_function_name(trait_, &param_types)
                            .is_some()
                        {
                            return Err(Spanned {
                                value: ParserError::TraitAlreadyImplemented(
                                    trait_.clone(),
                                    inner.clone(),
                                ),
                                span,
                            });
                        }

                        if let Some(err) = self.check_trait_reqs(inner, trait_, &function) {
                            return Err(Spanned { value: err, span });
                        }

                        // self wird sonst doppelt geborrowed
                        let data_type_info = self.get_type_info_mut(inner);

                        let params = trait_.param_len();

                        // -1 weil jedes trait self als parameter hat
                        if function.params.value.len() - 1 != params {
                            return Err(Spanned {
                                value: ParserError::TraitParamCountMismatch(
                                    trait_.clone(),
                                    params,
                                    function.params.value.len() - 1,
                                ),
                                span,
                            });
                        }

                        function_name = format!(
                            "{}_{}",
                            trait_,
                            function
                                .params
                                .value
                                .iter()
                                .map(|p| p.value._type.value.clone())
                                .join("_")
                        );

                        let name = format!("{}_{}", inner, &function_name);
                        if !data_type_info.traits.insert((
                            trait_.clone(),
                            function
                                .params
                                .value
                                .iter()
                                // .skip(1)
                                .map(|p| p.value._type.value.clone())
                                .collect(),
                            Some(name),
                            function.return_type.value.clone(),
                        )) {
                            return Err(Spanned {
                                value: ParserError::MethodAlreadyExists(
                                    function_name,
                                    inner.to_string(),
                                ),
                                span,
                            });
                        }

                        function.trait_of = Some(inner.clone());
                    } else {
                        data_type_info.methods.push(function.name.clone());
                    }

                    function.name = format!("{}_{}", inner.internal_name(), &function_name);
                }
                None => {
                    if let Some(function_already_exists) =
                        self.program.functions.get(&function.name).cloned()
                    {
                        return Err(Spanned {
                            value: ParserError::FunctionAlreadyExists(name),
                            span: function_already_exists.span,
                        });
                    }
                }
            }

            self.program.functions.insert(
                function.name.clone(),
                Spanned {
                    value: function.clone(),
                    span: function.display_name.span,
                },
            );

            let body = match is_extern {
                true => Spanned {
                    ..Default::default()
                },
                false => self.parse_block(&mut function.body.value)?,
            };

            function.body = body;

            let function_span = span.extend(&function.body.span);

            // prüfe ob return mit return_type übereinstimmt
            if !is_extern && function.body.value.return_type != function.return_type.value {
                return Err(Spanned {
                    value: ParserError::WrongReturnType(
                        function.return_type.value,
                        function.body.value.return_type.clone(),
                    ),
                    span: function_span,
                });
            }

            self.program.functions.insert(
                function.name.clone(),
                Spanned {
                    value: function.clone(),
                    span: function.display_name.span,
                },
            );

            return Ok(Spanned {
                value: function,
                span: function_span,
            });
        }

        unreachable!()
    }

    fn parse_func_params(
        &mut self,
        generics: &Vec<Spanned<DataType>>,
    ) -> Result<Spanned<Vec<Spanned<FunctionParam>>>, Spanned<ParserError>> {
        let mut params = vec![];
        let mut span = self
            .expect_next(&[Token::Punctuation(Punctuation::OpenParen)])?
            .span;

        let end_span = self.walk_separated_values(
            Token::Punctuation(Punctuation::Comma),
            Token::Punctuation(Punctuation::CloseParen),
            |parser| {
                let name =
                    parser.expect_next(&[Token::Identifier("parameter name".to_string())])?;
                if let Spanned {
                    value: Token::Identifier(param_name),
                    span: param_span,
                } = name
                {
                    if param_name != *CLASS_SELF_ARG_NAME {
                        let _ = parser.expect_next(&[Token::Punctuation(Punctuation::Colon)])?;
                        let _type = parser.parse_data_type(Some(generics))?;

                        let param = Spanned {
                            value: FunctionParam {
                                name: Spanned {
                                    value: param_name,
                                    span: param_span,
                                },
                                _type,
                            },
                            span: param_span,
                        };

                        params.push(param);
                    } else {
                        let self_param = Spanned {
                            value: FunctionParam {
                                name: Spanned {
                                    value: param_name,
                                    span: param_span,
                                },
                                _type: Spanned {
                                    value: DataType::None,
                                    span: param_span,
                                },
                            },
                            span: param_span,
                        };

                        params.push(self_param);
                    }
                }

                Ok(())
            },
        )?;

        span = span.extend(&end_span);

        Ok(Spanned {
            value: params,
            span,
        })
    }

    fn parse_func_return_type(
        &mut self,
        generics: &Vec<Spanned<DataType>>,
    ) -> Result<Option<Spanned<DataType>>, Spanned<ParserError>> {
        match self.peek().ok() {
            Some(Spanned {
                value: Token::Punctuation(Punctuation::ThinArrow),
                ..
            }) => self.next_token()?,
            Some(Spanned {
                value: Token::Punctuation(Punctuation::OpenBrace),
                ..
            }) => return Ok(None),

            _ => self.expect_next(&[Token::Punctuation(Punctuation::ThinArrow)])?,
        };

        Ok(Some(self.parse_data_type(Some(generics))?)) // TODO
                                                        // if let Spanned { value: Token::Identifier(data_type_name), span: data_type_span } = self.expect_next(&[Token::Identifier("datatype".to_string())])? {

        //     let data_type = DataType::from_str(&data_type_name)
        //         .map_err(|x| Spanned { value: x, span: data_type_span.clone() })?;

        //     return Ok(Some(Spanned {
        //         value: data_type,
        //         span: data_type_span,
        //     }))
        // }

        // unreachable!()
    }

    pub(in crate::parser) fn parse_func_call(
        &mut self,
        scope: &mut Block,
        caller: Option<&Spanned<TypedExpr>>,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        if let Spanned {
            value: Token::Identifier(mut function_name),
            span,
        } = self.expect_next(&[Token::Identifier("function name".to_string())])?
        {
            let mut generic_annotations = vec![];
            let mut span = span;

            if Token::Operator(Operator::LessThan) == self.peek()?.value {
                generic_annotations = self.collect_generic_annotations(Some(&scope.generics))?;
            }

            let mut args_span = self
                .expect_next(&[Token::Punctuation(Punctuation::OpenParen)])?
                .span;

            let mut function: Option<Result<Spanned<Function>, Spanned<ParserError>>> = None;

            let display_name = function_name.clone();

            if let Some(caller) = &caller {
                if let DataType::Custom(inner) = &caller.value._type {
                    let class_name = inner.name.clone();
                    let raw_function_name = function_name.clone();
                    // zuerst nicht generische implementierung bekommen
                    // foo(self) for Foo<int>
                    function_name = format!("{}_{}", class_name, raw_function_name);
                    let mut func_res = self
                        .get_function(&Spanned {
                            value: function_name.clone(),
                            span,
                        })
                        .cloned();

                    // wenn es keine implementierung gibt,
                    // dann generische implementierung
                    // foo(self) for Foo<T>
                    if func_res.is_err() {
                        if let Some(subtype_of) = &inner.subtype_of {
                            function_name = format!("{}_{}", subtype_of, raw_function_name);

                            func_res = self
                                .get_function(&Spanned {
                                    value: function_name.clone(),
                                    span,
                                })
                                .cloned();
                        }
                    };
                    function = Some(func_res);
                } else {
                    // eingebauter datentyp
                    function_name =
                        format!("{}_{}", caller.value._type.internal_name(), function_name);
                    function = Some(
                        self.get_function(&Spanned {
                            value: function_name.clone(),
                            span,
                        })
                        .cloned(),
                    );
                }
            }

            let mut function = match function {
                Some(Ok(function)) => function,
                Some(Err(function_does_not_exist)) => {
                    if let Some(caller) = caller {
                        return Err(Spanned {
                            value: ParserError::MethodDoesNotExist(
                                display_name,
                                caller.value._type.to_string(),
                            ),
                            span,
                        });
                    }

                    return Err(function_does_not_exist);
                }
                // keine methode
                None => self
                    .get_function(&Spanned {
                        value: function_name.clone(),
                        span,
                    })?
                    .to_owned(),
            };

            let mut args = vec![];

            if let Some(class) = caller {
                if let Some(self_param) = function.value.params.value.get_mut(0) {
                    // Generics
                    if class.value._type == self_param.value._type.value
                        || if let (DataType::Custom(self_arg), DataType::Custom(self_param)) =
                            (&class.value._type, &self_param.value._type.value)
                        {
                            self_arg.subtype_of == Some(self_param.name.clone())
                        } else {
                            false
                        }
                    {
                        if let Expr::ClassName(_) = class.value.expression {
                            return Err(Spanned {
                                value: ParserError::MethodIsNotStatic(
                                    function_name,
                                    class.value._type.to_string(),
                                ),
                                span,
                            });
                        }

                        args.push(Spanned {
                            value: CallArg(class.value.clone()),
                            span: class.span,
                        });
                    }
                } else if function.value.is_static_method()
                    && !matches!(class.value.expression, Expr::ClassName(_))
                {
                    return Err(Spanned {
                        value: ParserError::MethodIsStatic(
                            function_name,
                            class.value._type.to_string(),
                        ),
                        span,
                    });
                }
            }

            let end = self.walk_separated_values(
                Token::Punctuation(Punctuation::Comma),
                Token::Punctuation(Punctuation::CloseParen),
                |parser| {
                    let value = parser.parse_expression(scope)?;
                    let arg = Spanned {
                        value: CallArg(value.value),
                        span: value.span,
                    };

                    args_span = span.extend(&arg.span);
                    args.push(arg);

                    Ok(())
                },
            )?;

            span = span.extend(&end);

            if function.value.is_generic() {
                //     generics_annotations = if !generics_annotations.is_empty() {
                //         generics_annotations
                //     } else {
                //         let mut generics: Vec<DataType> = function
                //             .value
                //             .params
                //             .value
                //             .iter()
                //             .map(|p| p.value._type.value.clone())
                //             .collect();

                //         if let Some(ref caller) = function.value.method_of {
                //             generics.push(caller.clone());
                //         }

                //         let mut specifics: Vec<DataType> =
                //             args.iter().map(|a| a.value.0._type.clone()).collect();

                //         specifics.push(function.value.return_type.value.clone());

                //         let w = inferred_generic_annotations(&specifics, &generics)
                //             .into_iter()
                //             .map(|a| Spanned {
                //                 value: a,
                //                 span: Span::default(),
                //             })
                //             .collect_vec();

                //         // println!("w: {:?}", &w);
                //         w
                //     };

                // if generics_annotations.len() != function.value.generic_param_count() {
                //     return Err(Spanned {
                //         value: ParserError::WrongGenericParamCount(
                //             function.value.generic_param_count(),
                //             generics_annotations.len(),
                //         ),
                //         span,
                //     });
                // }

                // let mut specific_types: Vec<DataType> = generics_annotations
                //     .iter()
                //     .map(|g| g.value.clone())
                //     .collect();

                let mut extra_annotations = None;

                let mut param_types = function
                    .value
                    .params
                    .value
                    .iter()
                    .map(|p| p.value._type.value.clone())
                    .collect::<Vec<DataType>>();

                let return_type_generics = function.value.return_type.value.generics();
                if !return_type_generics.is_empty() {
                    param_types.push(function.value.return_type.value.clone());

                    if generic_annotations.len() != return_type_generics.len() {
                        // Return type sollte der einzige fall sein wo man annotations braucht
                        // hoffentlich
                        return Err(Spanned {
                            value: ParserError::WrongGenericParamCount(
                                return_type_generics.len(),
                                generic_annotations.len(),
                            ),
                            span,
                        });
                    }

                    let mut annotation_map: AHashMap<String, DataType> = AHashMap::new();

                    for (annotated, return_type_generic_name) in
                        generic_annotations.iter().zip(return_type_generics.iter())
                    {
                        annotation_map
                            .insert(return_type_generic_name.clone(), annotated.value.clone());
                    }

                    extra_annotations = Some(annotation_map);
                }

                // if !generic_annotations.is_empty() {
                //     if generic_annotations.len() != function.value.generic_param_count() {
                //         return Err(Spanned {
                //             value: ParserError::WrongGenericParamCount(
                //                 function.value.generic_param_count(),
                //                 generic_annotations.len(),
                //             ),
                //             span,
                //         });
                //     }

                //     for (param_type, annotation) in param_types.iter_mut().zip(generic_annotations.iter()) {
                //         let to_specify = param_type._type_mut();
                //         // specify_generics(type_settables, generics, self, true);
                //     }
                // }

                let mut specific_types: Vec<DataType> =
                    args.iter().map(|a| a.value.0._type.clone()).collect();
                specific_types.push(function.value.return_type.value.clone());

                let (specified_generics, mut map) =
                    handle_generics(&specific_types, &param_types, extra_annotations.as_ref());

                if let Some(extra_annotations) = extra_annotations {
                    for (key, value) in extra_annotations {
                        map.insert(key, value);
                    }
                }

                // } else {
                //     if generics_annotations.len() != function.value.generic_param_count() {
                //         return Err(Spanned {
                //             value: ParserError::WrongGenericParamCount(
                //                 function.value.generic_param_count(),
                //                 generics_annotations.len(),
                //             ),
                //             span,
                //         });
                //     }

                //     println!("GEN: {:?}", generics_annotations);
                //     println!("PARAM: {:?}", param_types);

                //     handle_generics(&generics_annotations.iter().map(|g| g.value.clone()).collect::<Vec<DataType>>(), &param_types)
                // };

                // println!("MAP: {:?}", map);

                // specific_types.push(function.value.return_type.value.clone());

                // println!("FF: {:?}", specific_types);

                // println!("specific types: {:?}", specific_types);

                let subtype = match function
                    .value
                    .generic_subtypes
                    .get(&specific_types)
                    .cloned()
                {
                    Some(subtype) => subtype,
                    None => {
                        // println!("SDASDSADAS");
                        // println!("QDQDQ: {:?}", specific_types);
                        let caller_type = caller.as_ref().map(|c| &c.value._type);
                        let subtype =
                            function
                                .value
                                .subtype(&map, caller_type, &mut self.program, true);

                        let function_ref = self.program.functions.get_mut(&function_name).unwrap();

                        function_ref
                            .value
                            .generic_subtypes
                            .insert(specified_generics, subtype.clone());

                        subtype
                    }
                };

                function.value = subtype;
            }

            // TODO: https://stackoverflow.com/questions/38163675/iterate-two-vectors-with-different-lengths
            for (param, arg) in function
                .value
                .params
                .value
                .iter()
                .map(|p| p.value.clone())
                .zip(args.iter())
            {
                if param._type.value != arg.value.0._type
                // && !arg.value.0._type.can_be_converted_to(&param._type.value)
                {
                    return Err(Spanned {
                        value: ParserError::WrongType(param._type.value, arg.value.0._type.clone()),
                        span: arg.span,
                    });
                }
            }

            if function.value.params.value.len() != args.len() {
                return Err(Spanned {
                    value: ParserError::WrongArguments(
                        function
                            .value
                            .params
                            .value
                            .into_iter()
                            .map(|p| p.value._type.value.to_string())
                            .collect(),
                        args.into_iter()
                            .map(|a| a.value.0._type.to_string())
                            .collect(),
                    ),
                    span: args_span,
                });
            }

            let out = Ok(Spanned {
                value: TypedExpr {
                    expression: Expr::Call {
                        function: function.clone(),
                        args: Spanned {
                            value: args,
                            span: args_span,
                        },
                    },
                    _type: function.value.return_type.value,
                    raw: None,
                },
                span,
            });

            return out;
        }

        unreachable!()
    }
}
