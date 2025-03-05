#![cfg(test)]
use crate::lexer::{
    lexer_main::{lex, lex_unspanned},
    position::{Position, Span, Spanned},
    tokens::{Keyword, Literal, Operator, Punctuation, Token},
};
use pretty_assertions::assert_eq;

#[test]
pub fn unspanned_basic() {
    let input = ";..})";

    let expected = vec![
        Token::Punctuation(Punctuation::SemiColon),
        Token::Punctuation(Punctuation::Period),
        Token::Punctuation(Punctuation::Period),
        Token::Punctuation(Punctuation::CloseBrace),
        Token::Punctuation(Punctuation::CloseParen),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected)
}

#[test]
pub fn unspanned_mixed() {
    let input = "if true false {return;}; .";

    let expected = vec![
        Token::Keyword(Keyword::If),
        Token::DataLiteral(Literal::Boolean(true)),
        Token::DataLiteral(Literal::Boolean(false)),
        Token::Punctuation(Punctuation::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::Punctuation(Punctuation::SemiColon),
        Token::Punctuation(Punctuation::CloseBrace),
        Token::Punctuation(Punctuation::SemiColon),
        Token::Punctuation(Punctuation::Period),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected)
}

#[test]
pub fn unspanned_int() {
    let input = "123 1_000 1__2__3";

    let expected = vec![
        Token::DataLiteral(Literal::Integer(123)),
        Token::DataLiteral(Literal::Integer(1000)),
        Token::DataLiteral(Literal::Integer(123)),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_int_2() {
    let input = "12 _1_2_3"; // _ am anfang nicht erlaubt

    let expected = vec![
        Token::DataLiteral(Literal::Integer(12)),
        Token::Identifier("_1_2_3".to_string()),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_float() {
    let input = "1.23 1.0 0.555";

    let expected = vec![
        Token::DataLiteral(Literal::Float(1.23)),
        Token::DataLiteral(Literal::Float(1.0)),
        Token::DataLiteral(Literal::Float(0.555)),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_float_2() {
    let input = "0.23 1. .555";

    let expected = vec![
        Token::DataLiteral(Literal::Float(0.23)),
        Token::DataLiteral(Literal::Float(1.)),
        Token::DataLiteral(Literal::Float(0.555)),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_mixed_2() {
    let input = "123 0.15 1. .5 true false";

    let expected = vec![
        Token::DataLiteral(Literal::Integer(123)),
        Token::DataLiteral(Literal::Float(0.15)),
        Token::DataLiteral(Literal::Float(1.0)),
        Token::DataLiteral(Literal::Float(0.5)),
        Token::DataLiteral(Literal::Boolean(true)),
        Token::DataLiteral(Literal::Boolean(false)),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_mixed_3() {
    let input = r#"if 0 == 1.
    {return 24+5;}
    return true;"#;

    let expected = vec![
        Token::Keyword(Keyword::If),
        Token::DataLiteral(Literal::Integer(0)),
        Token::Operator(Operator::Equal),
        Token::DataLiteral(Literal::Float(1.0)),
        Token::Punctuation(Punctuation::OpenBrace),
        Token::Keyword(Keyword::Return),
        Token::DataLiteral(Literal::Integer(24)),
        Token::Operator(Operator::Add),
        Token::DataLiteral(Literal::Integer(5)),
        Token::Punctuation(Punctuation::SemiColon),
        Token::Punctuation(Punctuation::CloseBrace),
        Token::Keyword(Keyword::Return),
        Token::DataLiteral(Literal::Boolean(true)),
        Token::Punctuation(Punctuation::SemiColon),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_mixed_4() {
    let input = "foo.inner<int>();";

    let expected = vec![
        Token::Identifier("foo".to_string()),
        Token::Punctuation(Punctuation::Period),
        Token::Identifier("inner".to_string()),
        Token::Operator(Operator::LessThan),
        Token::Identifier("int".to_string()),
        Token::Operator(Operator::GreaterThan),
        Token::Punctuation(Punctuation::OpenParen),
        Token::Punctuation(Punctuation::CloseParen),
        Token::Punctuation(Punctuation::SemiColon),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_string() {
    let input = r#"
        "test 123"
    "#;

    let expected = vec![Token::String("test 123".to_string())];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_ident() {
    let input = r#"def main() {
        let x: int = 10;
    }"#;

    let expected = vec![
        Token::Keyword(Keyword::Def),
        Token::Identifier("main".to_string()),
        Token::Punctuation(Punctuation::OpenParen),
        Token::Punctuation(Punctuation::CloseParen),
        Token::Punctuation(Punctuation::OpenBrace),
        Token::Keyword(Keyword::Let),
        Token::Identifier("x".to_string()),
        Token::Punctuation(Punctuation::Colon),
        Token::Identifier("int".to_string()),
        Token::Assignment,
        Token::DataLiteral(Literal::Integer(10)),
        Token::Punctuation(Punctuation::SemiColon),
        Token::Punctuation(Punctuation::CloseBrace),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn unspanned_similar_tokens() {
    let input = "== = = == != < <= > >=";

    let expected = vec![
        Token::Operator(Operator::Equal),
        Token::Assignment,
        Token::Assignment,
        Token::Operator(Operator::Equal),
        Token::Operator(Operator::NotEqual),
        Token::Operator(Operator::LessThan),
        Token::Operator(Operator::LessThanOrEqual),
        Token::Operator(Operator::GreaterThan),
        Token::Operator(Operator::GreaterThanOrEqual),
    ];

    assert_eq!(lex_unspanned(input).unwrap(), expected);
}

#[test]
pub fn spanned_string() {
    let input = r#""test 123""#;

    let expected = vec![Spanned {
        value: Token::String("test 123".to_string()),
        span: Span {
            start: Position {
                abs: 0,
                row: 0,
                column: 0,
            },
            end: Position {
                abs: 10,
                row: 0,
                column: 10,
            },
        },
    }];

    assert_eq!(lex(input).unwrap(), expected)
}

#[test]
pub fn spanned_basic() {
    let input = ";  ;    .  ( ) ;";

    let expected = vec![
        Spanned {
            value: Token::Punctuation(Punctuation::SemiColon),
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
        },
        Spanned {
            value: Token::Punctuation(Punctuation::SemiColon),
            span: Span {
                start: Position {
                    abs: 3,
                    row: 0,
                    column: 3,
                },
                end: Position {
                    abs: 4,
                    row: 0,
                    column: 4,
                },
            },
        },
        Spanned {
            value: Token::Punctuation(Punctuation::Period),
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
        },
        Spanned {
            value: Token::Punctuation(Punctuation::OpenParen),
            span: Span {
                start: Position {
                    abs: 11,
                    row: 0,
                    column: 11,
                },
                end: Position {
                    abs: 12,
                    row: 0,
                    column: 12,
                },
            },
        },
        Spanned {
            value: Token::Punctuation(Punctuation::CloseParen),
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
        },
        Spanned {
            value: Token::Punctuation(Punctuation::SemiColon),
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
    ];

    assert_eq!(lex(input).unwrap(), expected)
}

#[test]
pub fn spanned_basic_multiline() {
    let input = "def main() {
    return 0 == 0;
}";

    let expected = vec![
        Spanned {
            value: Token::Keyword(Keyword::Def),
            span: Span {
                start: Position {
                    abs: 0,
                    row: 0,
                    column: 0,
                },
                end: Position {
                    abs: 3,
                    row: 0,
                    column: 3,
                },
            },
        },
        Spanned {
            value: Token::Identifier("main".to_string()),
            span: Span {
                start: Position {
                    abs: 4,
                    row: 0,
                    column: 4,
                },
                end: Position {
                    abs: 8,
                    row: 0,
                    column: 8,
                },
            },
        },
        Spanned {
            value: Token::Punctuation(Punctuation::OpenParen),
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
        },
        Spanned {
            value: Token::Punctuation(Punctuation::CloseParen),
            span: Span {
                start: Position {
                    abs: 9,
                    row: 0,
                    column: 9,
                },
                end: Position {
                    abs: 10,
                    row: 0,
                    column: 10,
                },
            },
        },
        Spanned {
            value: Token::Punctuation(Punctuation::OpenBrace),
            span: Span {
                start: Position {
                    abs: 11,
                    row: 0,
                    column: 11,
                },
                end: Position {
                    abs: 12,
                    row: 0,
                    column: 12,
                },
            },
        },
        Spanned {
            value: Token::Keyword(Keyword::Return),
            span: Span {
                start: Position {
                    abs: 17,
                    row: 1,
                    column: 4,
                },
                end: Position {
                    abs: 23,
                    row: 1,
                    column: 10,
                },
            },
        },
        Spanned {
            value: Token::DataLiteral(Literal::Integer(0)),
            span: Span {
                start: Position {
                    abs: 24,
                    row: 1,
                    column: 11,
                },
                end: Position {
                    abs: 25,
                    row: 1,
                    column: 12,
                },
            },
        },
        Spanned {
            value: Token::Operator(Operator::Equal),
            span: Span {
                start: Position {
                    abs: 26,
                    row: 1,
                    column: 13,
                },
                end: Position {
                    abs: 28,
                    row: 1,
                    column: 15,
                },
            },
        },
        Spanned {
            value: Token::DataLiteral(Literal::Integer(0)),
            span: Span {
                start: Position {
                    abs: 29,
                    row: 1,
                    column: 16,
                },
                end: Position {
                    abs: 30,
                    row: 1,
                    column: 17,
                },
            },
        },
        Spanned {
            value: Token::Punctuation(Punctuation::SemiColon),
            span: Span {
                start: Position {
                    abs: 30,
                    row: 1,
                    column: 17,
                },
                end: Position {
                    abs: 31,
                    row: 1,
                    column: 18,
                },
            },
        },
        Spanned {
            value: Token::Punctuation(Punctuation::CloseBrace),
            span: Span {
                start: Position {
                    abs: 32,
                    row: 2,
                    column: 0,
                },
                end: Position {
                    abs: 33,
                    row: 2,
                    column: 1,
                },
            },
        },
    ];

    assert_eq!(lex(input).unwrap(), expected)
}

// #[test]
// fn lexer_lines() {
//     let input = "hello
// world
//     test
// 1
// 2
//     3
// ";

//     let lexer = Lexer::new(input);

//     let expected = vec![
//         "hello".to_string(),
//         "world".to_string(),
//         "    test".to_string(),
//         "1".to_string(),
//         "2".to_string(),
//         "    3".to_string(),
//     ];

//     let real = vec![
//         lexer.line(0).unwrap(),
//         lexer.line(1).unwrap(),
//         lexer.line(2).unwrap(),
//         lexer.line(3).unwrap(),
//         lexer.line(4).unwrap(),
//         lexer.line(5).unwrap(),
//     ];

//     assert_eq!(real, expected);
// }
