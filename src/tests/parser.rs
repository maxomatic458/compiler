#![cfg(test)]

use crate::{
    lexer::{
        lexer_main::lex,
        position::{Position, Span, Spanned},
        tokens::Literal,
    },
    parser::{
        ast::{BinaryOperator, DataType, Expr, Function, Statement, TypedExpr},
        error::ParserError,
        parser_main::Parser,
        utils::same_variant,
    },
};

use pretty_assertions::assert_eq;

#[test]
fn simple_spanned() {
    let tokens = lex("2").unwrap();
    let mut parser = Parser::new(tokens, None);

    let expr = parser.parse_expression(&mut Default::default()).unwrap();

    let expected = Spanned {
        value: TypedExpr {
            expression: Expr::Literal(Literal::Integer(2)),
            _type: DataType::Integer64,
            raw: None,
        },
        span: Span {
            start: Position {
                abs: 0,
                row: 0,
                column: 0,
            },
            end: Position {
                abs: 1,
                row: 0,
                column: 1,
            },
        },
    };

    assert_eq!(expr, expected);
}

#[test]
fn simple_spanned_2() {
    let tokens = lex("2 + 2").unwrap();
    let mut parser = Parser::new(tokens, None);

    let expr = parser.parse_expression(&mut Default::default()).unwrap();

    let expected = Spanned {
        value: TypedExpr {
            expression: Expr::Binary {
                lhs: Box::new(Spanned {
                    value: TypedExpr {
                        expression: Expr::Literal(Literal::Integer(2)),
                        _type: DataType::Integer64,
                        raw: None,
                    },
                    span: Span {
                        start: Position {
                            abs: 0,
                            row: 0,
                            column: 0,
                        },
                        end: Position {
                            abs: 1,
                            row: 0,
                            column: 1,
                        },
                    },
                }),
                op: Spanned {
                    value: BinaryOperator::Add,
                    span: Span {
                        start: Position {
                            abs: 2,
                            row: 0,
                            column: 2,
                        },
                        end: Position {
                            abs: 3,
                            row: 0,
                            column: 3,
                        },
                    },
                },
                rhs: Box::new(Spanned {
                    value: TypedExpr {
                        expression: Expr::Literal(Literal::Integer(2)),
                        _type: DataType::Integer64,
                        raw: None,
                    },
                    span: Span {
                        start: Position {
                            abs: 4,
                            row: 0,
                            column: 4,
                        },
                        end: Position {
                            abs: 5,
                            row: 0,
                            column: 5,
                        },
                    },
                }),
            },
            _type: DataType::Integer64,
            raw: None,
        },
        span: Span {
            start: Position {
                abs: 0,
                row: 0,
                column: 0,
            },
            end: Position {
                abs: 5,
                row: 0,
                column: 5,
            },
        },
    };

    assert_eq!(expected, expr)
}

#[test]
fn operator_priority() {
    let tokens = lex("1 + 2 * 3").unwrap();
    let mut parser = Parser::new(tokens, None);

    let expr = parser.parse_expression(&mut Default::default()).unwrap();

    let expected = Spanned {
        value: TypedExpr {
            expression: Expr::Binary {
                lhs: Box::new(Spanned {
                    value: TypedExpr {
                        expression: Expr::Literal(Literal::Integer(1)),
                        _type: DataType::Integer64,
                        raw: None,
                    },
                    span: Span {
                        start: Position {
                            abs: 0,
                            row: 0,
                            column: 0,
                        },
                        end: Position {
                            abs: 1,
                            row: 0,
                            column: 1,
                        },
                    },
                }),
                op: Spanned {
                    value: BinaryOperator::Add,
                    span: Span {
                        start: Position {
                            abs: 2,
                            row: 0,
                            column: 2,
                        },
                        end: Position {
                            abs: 3,
                            row: 0,
                            column: 3,
                        },
                    },
                },
                rhs: Box::new(Spanned {
                    value: TypedExpr {
                        expression: Expr::Binary {
                            lhs: Box::new(Spanned {
                                value: TypedExpr {
                                    expression: Expr::Literal(Literal::Integer(2)),
                                    _type: DataType::Integer64,
                                    raw: None,
                                },
                                span: Span {
                                    start: Position {
                                        abs: 4,
                                        row: 0,
                                        column: 4,
                                    },
                                    end: Position {
                                        abs: 5,
                                        row: 0,
                                        column: 5,
                                    },
                                },
                            }),
                            op: Spanned {
                                value: BinaryOperator::Multiply,
                                span: Span {
                                    start: Position {
                                        abs: 6,
                                        row: 0,
                                        column: 6,
                                    },
                                    end: Position {
                                        abs: 7,
                                        row: 0,
                                        column: 7,
                                    },
                                },
                            },
                            rhs: Box::new(Spanned {
                                value: TypedExpr {
                                    expression: Expr::Literal(Literal::Integer(3)),
                                    _type: DataType::Integer64,
                                    raw: None,
                                },
                                span: Span {
                                    start: Position {
                                        abs: 8,
                                        row: 0,
                                        column: 8,
                                    },
                                    end: Position {
                                        abs: 9,
                                        row: 0,
                                        column: 9,
                                    },
                                },
                            }),
                        },
                        _type: DataType::Integer64,
                        raw: None,
                    },
                    span: Span {
                        start: Position {
                            abs: 4,
                            row: 0,
                            column: 4,
                        },
                        end: Position {
                            abs: 9,
                            row: 0,
                            column: 9,
                        },
                    },
                }),
            },
            _type: DataType::Integer64,
            raw: None,
        },
        span: Span {
            start: Position {
                abs: 0,
                row: 0,
                column: 0,
            },
            end: Position {
                abs: 9,
                row: 0,
                column: 9,
            },
        },
    };

    assert_eq!(expr, expected);
}

// #[test]
#[allow(dead_code)]
fn variable_decl() {
    let tokens = lex("let x: int64 = 5 + 5;").unwrap();
    let mut parser = Parser::new(tokens, None);
    let mut scope = Function::default();

    let variable_decl = parser.parse_statement(&mut scope.body.value).unwrap();

    let expected = Spanned {
        value: Statement::VariableDecl {
            is_mutable: false,
            name: Spanned {
                value: "x".to_string(),
                span: Span {
                    start: Position {
                        abs: 4,
                        row: 0,
                        column: 4,
                    },
                    end: Position {
                        abs: 5,
                        row: 0,
                        column: 5,
                    },
                },
            },
            _type: Some(Spanned {
                value: DataType::Integer64,
                span: Span {
                    start: Position {
                        abs: 7,
                        row: 0,
                        column: 7,
                    },
                    end: Position {
                        abs: 13,
                        row: 0,
                        column: 12,
                    },
                },
            }),
            value: Spanned {
                value: TypedExpr {
                    expression: Expr::Binary {
                        lhs: Box::new(Spanned {
                            value: TypedExpr {
                                expression: Expr::Literal(Literal::Integer(5)),
                                _type: DataType::Integer64,
                                raw: None,
                            },
                            span: Span {
                                start: Position {
                                    abs: 13,
                                    row: 0,
                                    column: 13,
                                },
                                end: Position {
                                    abs: 14,
                                    row: 0,
                                    column: 14,
                                },
                            },
                        }),
                        op: Spanned {
                            value: BinaryOperator::Add,
                            span: Span {
                                start: Position {
                                    abs: 15,
                                    row: 0,
                                    column: 15,
                                },
                                end: Position {
                                    abs: 16,
                                    row: 0,
                                    column: 16,
                                },
                            },
                        },
                        rhs: Box::new(Spanned {
                            value: TypedExpr {
                                expression: Expr::Literal(Literal::Integer(5)),
                                _type: DataType::Integer64,
                                raw: None,
                            },
                            span: Span {
                                start: Position {
                                    abs: 17,
                                    row: 0,
                                    column: 17,
                                },
                                end: Position {
                                    abs: 18,
                                    row: 0,
                                    column: 18,
                                },
                            },
                        }),
                    },
                    _type: DataType::Integer64,
                    raw: None,
                },
                span: Span {
                    start: Position {
                        abs: 13,
                        row: 0,
                        column: 13,
                    },
                    end: Position {
                        abs: 18,
                        row: 0,
                        column: 18,
                    },
                },
            },
        },
        span: Span {
            start: Position {
                abs: 0,
                row: 0,
                column: 0,
            },
            end: Position {
                abs: 18,
                row: 0,
                column: 18,
            },
        },
    };

    assert_eq!(variable_decl, expected);
    assert!(scope.body.value.variables.contains_key("x"))
}

#[test]
fn variable_type_mismatch() {
    let tokens = lex("let x: int64 = 1.0;").unwrap(); // falsche typ angabe
    let mut parser = Parser::new(tokens, None);

    let variable_decl = parser.parse_statement(&mut Default::default());

    assert_eq!(
        variable_decl,
        Err(Spanned {
            value: ParserError::WrongType(DataType::Integer64, DataType::Float),
            span: Span {
                start: Position {
                    abs: 7,
                    row: 0,
                    column: 7
                },
                end: Position {
                    abs: 18,
                    row: 0,
                    column: 18
                },
            },
        })
    )
}

#[test]
fn undefined_variable_in_expr() {
    let tokens = lex("let x: int64 = y + 1").unwrap(); // falsche typ angabe
    let mut parser = Parser::new(tokens, None);

    let variable_decl = parser.parse_statement(&mut Default::default());

    let expected = Err(Spanned {
        value: ParserError::VariableNotFound("y".to_string()),
        span: Span {
            start: Position {
                abs: 15,
                row: 0,
                column: 15,
            },
            end: Position {
                abs: 16,
                row: 0,
                column: 16,
            },
        },
    });

    assert_eq!(variable_decl, expected)
}

#[test]
fn function_decl() {
    let tokens = lex("
    def main(x: int64, y: int64) {
        let z = x + y;
    }")
    .unwrap();

    let mut parser = Parser::new(tokens, None);

    let func = parser.parse_func_def();

    assert!(func.is_ok())
}

#[test]
fn function_decl_missing_var() {
    let tokens = lex("
    def main(x: int64) {
        let z = x + y;
    }")
    .unwrap();

    let mut parser = Parser::new(tokens, None);

    let func = parser.parse_func_def();

    assert!(same_variant(
        &func.err().unwrap().value,
        &ParserError::VariableNotFound("".to_string())
    ))
}

#[test]
fn function_decl_missing_return() {
    let tokens = lex("
    def main(x: int64, y: int64) -> int64 {
        let z = x + y;
    }")
    .unwrap();

    let mut parser = Parser::new(tokens, None);

    let func = parser.parse_func_def();

    assert!(same_variant(
        &func.err().unwrap().value,
        &ParserError::WrongReturnType(DataType::None, DataType::None)
    ))
}

#[test]
fn wrong_return_type() {
    let tokens = lex("
    def main(x: float) -> int64 {
        return x;
    }
    ")
    .unwrap();

    let mut parser = Parser::new(tokens, None);

    let func = parser.parse_func_def();

    assert!(same_variant(
        &func.err().unwrap().value,
        &ParserError::WrongReturnType(DataType::Integer64, DataType::Float)
    ))
}

#[test]
fn optional_semicolons() {
    let tokens1 = lex("
    def main() -> bool {
        if true {
            return true;
        };
        return false
    } 
    ")
    .unwrap();

    let tokens2 = lex("
    def main() -> bool {
        if true {
            return true 
        } 
        return false
    } 
    ")
    .unwrap();

    let mut parser = Parser::new(tokens1, None);
    let func1 = parser.parse_func_def();

    let mut parser = Parser::new(tokens2, None);
    let func2 = parser.parse_func_def();

    assert_eq!(func1, func2)
}

#[test]
fn condition() {
    let tokens = lex("0 == 0").unwrap();
    let mut parser = Parser::new(tokens, None);
    let expr = parser.parse_expression(&mut Default::default()).unwrap();
    assert_eq!(expr.value._type, DataType::Boolean);

    if let Expr::Binary { op, .. } = expr.value.expression {
        assert_eq!(op.value, BinaryOperator::Equal)
    } else {
        panic!()
    }

    let tokens = lex("(( 3 + 20) * 28) <= (((20 + 10) + 10) * 20) *3").unwrap();
    let mut parser = Parser::new(tokens, None);
    let expr = parser.parse_expression(&mut Default::default()).unwrap();
    assert_eq!(expr.value._type, DataType::Boolean);

    if let Expr::Binary { op, .. } = expr.value.expression {
        assert_eq!(op.value, BinaryOperator::LessThanOrEqual)
    } else {
        panic!()
    }

    let tokens = lex("(( 3 + 20) * 28.0) <= (((20 + 10) + 10) * 20) *3").unwrap();
    let mut parser = Parser::new(tokens, None);
    let expr = parser.parse_expression(&mut Default::default());
    assert!(expr.is_err());
}

// #[test]
// fn function_call() {
//     let program = Program {
//         classes: btreemap! {},
//         functions: btreemap! {
//             "foo".to_string() => Spanned {
//                 value: Function {
//                     name: "".to_string(),
//                     // generics: vec![],
//                     is_builtin: false,
//                     generic_subtypes: HashMap::new(),
//                 display_name: Spanned { value: "foo".to_string(), span: Default::default() },
//                 method_of: None,
//                 params: Spanned {
//                     value: vec![
//                         Spanned {
//                             value: FunctionParam {
//                                 name: Spanned { value: "param1".to_string(), span: Default::default() },
//                                 _type: Spanned { value: DataType::Integer64, span: Default::default() }
//                             },
//                             span: Default::default(),
//                         },
//                         Spanned {
//                             value: FunctionParam {
//                                 name: Spanned { value: "param2".to_string(), span: Default::default() },
//                                 _type: Spanned { value: DataType::Boolean, span: Default::default() }
//                             },
//                             span: Default::default(),
//                         },
//                     ],
//                     span: Default::default(),
//                 },
//                 // local_variables: hashmap! {
//                 //     "param1".to_string() => Spanned {
//                 //         value: Variable { name: Spanned { value: "param1".to_string(), span: Default::default() }, _type: DataType::Integer64, is_mutable: false },
//                 //         span: Default::default(),
//                 //     },
//                 //     "param2".to_string() => Spanned {
//                 //         value: Variable { name: Spanned { value: "param2".to_string(), span: Default::default() }, _type: DataType::Boolean, is_mutable: false },
//                 //         span: Default::default(),
//                 //     },
//                 // },
//                 body: Spanned { value: Default::default(), span: Default::default() },
//                 return_type: Spanned { value: DataType::Integer64, span: Default::default() },
//                 is_extern: false,
//             },
//             span: Default::default(),
//             },
//         },
//         require_main: true,
//     };

//     let tokens = lex("let x = foo(5, 6 > 3) + 3;").unwrap();
//     let mut parser = Parser::new(tokens, None);
//     parser.program = program.clone();

//     let var_decl = parser.parse_statement(&mut Default::default());

//     // println!("{var_decl:?}");
//     assert!(var_decl.is_ok());

//     let tokens = lex("let x = foo(5, 6 > 3) + 3.5;").unwrap();
//     let mut parser = Parser::new(tokens, None);
//     parser.program = program.clone();

//     let var_decl = parser.parse_statement(&mut Default::default());

//     assert!(var_decl.is_err());

//     let tokens = lex("let x = foo(6 > 3, 5);").unwrap();
//     let mut parser = Parser::new(tokens, None);
//     parser.program = program.clone();

//     let var_decl = parser.parse_statement(&mut Default::default());

//     assert!(var_decl.is_err());

//     let tokens = lex("let x = foo(5, 6 > 3, 5) + 3;").unwrap();
//     let mut parser = Parser::new(tokens, None);
//     parser.program = program;

//     let var_decl = parser.parse_statement(&mut Default::default());

//     // println!("{var_decl:?}");
//     assert!(var_decl.is_err());
// }

#[test]
fn custom_data_types() {
    let tokens = lex("class Foo { x: int64, y: int64, }").unwrap();

    let mut parser = Parser::new(tokens, None);

    let class = parser.parse_class_def();

    assert!(class.is_ok());
    assert!(parser.program.custom_types.contains_key("Foo"));

    let tokens = lex("let x = Foo { x: 5, y: 3, };").unwrap();
    let mut parser_new = Parser::new(tokens, None);
    parser_new.program.custom_types = parser.program.custom_types.clone();

    let var_decl = parser_new.parse_statement(&mut Default::default());

    assert!(var_decl.is_ok());

    let tokens = lex("let x = Foo { x: 5, y: 3.0, };").unwrap();
    let mut parser_new = Parser::new(tokens, None);
    parser_new.program.custom_types = parser.program.custom_types.clone();

    let var_decl = parser_new.parse_statement(&mut Default::default());

    assert!(var_decl.is_err());

    let tokens = lex("let x = Foo { x: 5, z: 3, };").unwrap();
    let mut parser_new = Parser::new(tokens, None);
    parser_new.program.custom_types = parser.program.custom_types;

    let var_decl = parser_new.parse_statement(&mut Default::default());

    assert!(var_decl.is_err())
}

#[test]
fn type_inference() {
    let tokens = lex("def foo() -> int64 {
        return 0;
    }
    
    def main() -> int64 {
        let x: f32 = foo();
        return 0;
    }
    ")
    .unwrap();

    let mut parser = Parser::new(tokens, None);

    let res = parser.parse();

    assert!(res.is_err());
}
