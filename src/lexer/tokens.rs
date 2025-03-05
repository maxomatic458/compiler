use super::error::TokenError;
use crate::parser::ast::{
    ArrayLiteral, ClassLiteral, DataType, DataTypeGetter, DataTypeSettable, DataTypeSetter,
};
use crate::parser::ast::{BinaryOperator, UnaryOperator};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use std::str::FromStr;

use strum_macros::EnumIter;
use strum_macros::EnumString;

#[derive(Debug, Display, PartialEq, Clone, Serialize, Deserialize, EnumIter)]
pub enum Token {
    #[display(fmt = "{}", "_0")]
    Punctuation(Punctuation),

    #[display(fmt = "{}", "_0")]
    Operator(Operator),

    #[display(fmt = "{}", "_0")]
    Keyword(Keyword),

    #[display(fmt = "{}", "_0")]
    DataLiteral(Literal),

    #[display(fmt = "{}", "_0")]
    Identifier(String),

    #[display(fmt = "{}", "_0")]
    MacroKeyword(String),

    #[display(fmt = "{}", "_0")]
    String(String),

    #[display(fmt = "=")]
    Assignment,

    #[display(fmt = "{}", "_0")]
    ReassignmentOperator(ReassignmentOperator),

    #[display(fmt = "EOF")]
    EOF,
}

impl FromStr for Token {
    type Err = TokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "=" {
            return Ok(Token::Assignment);
        }

        if let Ok(reassignment_operator) = ReassignmentOperator::from_str(s) {
            return Ok(Token::ReassignmentOperator(reassignment_operator));
        }
        if let Ok(punctuation) = Punctuation::from_str(s) {
            return Ok(Token::Punctuation(punctuation));
        }
        if let Ok(binary_operator) = Operator::from_str(s) {
            return Ok(Token::Operator(binary_operator));
        }
        if let Ok(keyword) = Keyword::from_str(s) {
            return Ok(Token::Keyword(keyword));
        }
        if let Ok(data_literal) = Literal::from_str(s) {
            return Ok(Token::DataLiteral(data_literal));
        }

        Err(TokenError::UnknownToken(s.to_owned()))
    }
}

#[derive(
    Debug, Default, Display, PartialEq, Clone, Serialize, Deserialize, EnumString, EnumIter,
)]
pub enum Punctuation {
    /// `.`
    #[default]
    #[display(fmt = ".")]
    #[strum(serialize = ".")]
    Period,

    /// `:`
    #[display(fmt = ":")]
    #[strum(serialize = ":")]
    Colon,

    /// `;`
    #[display(fmt = ";")]
    #[strum(serialize = ";")]
    SemiColon,

    /// `,`
    #[display(fmt = ",")]
    #[strum(serialize = ",")]
    Comma,

    /// `|`
    #[display(fmt = "|")]
    #[strum(serialize = "|")]
    Pipe,

    /// `(`
    #[display(fmt = "(")]
    #[strum(serialize = "(")]
    OpenParen,

    /// `)`
    #[display(fmt = ")")]
    #[strum(serialize = ")")]
    CloseParen,

    /// `{`
    #[display(fmt = "{{")]
    #[strum(serialize = "{")]
    OpenBrace,

    /// `}`
    #[display(fmt = "}}")]
    #[strum(serialize = "}")]
    CloseBrace,

    /// `[`
    #[display(fmt = "[")]
    #[strum(serialize = "[")]
    OpenBracket,

    /// `]`
    #[display(fmt = "]")]
    #[strum(serialize = "]")]
    CloseBracket,

    /// `->`
    #[display(fmt = "->")]
    #[strum(serialize = "->")]
    ThinArrow,

    /// `~`
    #[display(fmt = "~")]
    #[strum(serialize = "~")]
    Tilde,

    /// `&`
    #[display(fmt = "&")]
    #[strum(serialize = "&")]
    Ampersand,

    /// `"`
    #[display(fmt = "\"")]
    #[strum(serialize = "\"")]
    DoubleQuote,

    /// `'`
    #[display(fmt = "'")]
    #[strum(serialize = "'")]
    SingleQuote,
}

#[derive(
    Debug, Display, Default, PartialEq, Clone, Serialize, Deserialize, EnumString, EnumIter,
)]
pub enum Operator {
    /// `+`
    #[default]
    #[display(fmt = "+")]
    #[strum(serialize = "+")]
    Add,

    /// `-`
    #[display(fmt = "-")]
    #[strum(serialize = "-")]
    Subtract,

    /// `*`
    #[display(fmt = "*")]
    #[strum(serialize = "*")]
    Multiply,

    /// `/`
    #[display(fmt = "/")]
    #[strum(serialize = "/")]
    Divide,

    // `%`
    #[display(fmt = "%")]
    #[strum(serialize = "%")]
    Modulo,

    /// `==`
    #[display(fmt = "==")]
    #[strum(serialize = "==")]
    Equal,

    /// `!=`
    #[display(fmt = "!=")]
    #[strum(serialize = "!=")]
    NotEqual,

    ///  `<`
    #[display(fmt = "<")]
    #[strum(serialize = "<")]
    LessThan,

    /// `<=`
    #[display(fmt = "<=")]
    #[strum(serialize = "<=")]
    LessThanOrEqual,

    /// `>`
    #[display(fmt = ">")]
    #[strum(serialize = ">")]
    GreaterThan,

    /// `>=`
    #[display(fmt = ">=")]
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,

    /// `~=`
    #[display(fmt = "~=")]
    #[strum(serialize = "~=")]
    LossyEqual,

    /// `&&`
    #[display(fmt = "&&")]
    #[strum(serialize = "&&")]
    And,

    /// `||`
    #[display(fmt = "||")]
    #[strum(serialize = "||")]
    Or,

    /// `!`
    #[display(fmt = "!")]
    #[strum(serialize = "!")]
    Not,
}

impl Operator {
    pub fn to_binary_op(&self) -> Option<BinaryOperator> {
        match self {
            Operator::Add => Some(BinaryOperator::Add),
            Operator::Subtract => Some(BinaryOperator::Subtract),
            Operator::Multiply => Some(BinaryOperator::Multiply),
            Operator::Divide => Some(BinaryOperator::Divide),
            Operator::Equal => Some(BinaryOperator::Equal),
            Operator::NotEqual => Some(BinaryOperator::NotEqual),
            Operator::LessThan => Some(BinaryOperator::LessThan),
            Operator::LessThanOrEqual => Some(BinaryOperator::LessThanOrEqual),
            Operator::GreaterThan => Some(BinaryOperator::GreaterThan),
            Operator::GreaterThanOrEqual => Some(BinaryOperator::GreaterThanOrEqual),
            Operator::And => Some(BinaryOperator::And),
            Operator::Or => Some(BinaryOperator::Or),
            Operator::Modulo => Some(BinaryOperator::Modulo),

            Operator::Not | Operator::LossyEqual => None,
        }
    }

    pub fn to_unary_op(&self) -> Option<UnaryOperator> {
        match self {
            Operator::Subtract => Some(UnaryOperator::Minus),
            Operator::Not => Some(UnaryOperator::Not),

            Operator::Add
            | Operator::Multiply
            | Operator::Divide
            | Operator::Equal
            | Operator::NotEqual
            | Operator::LessThan
            | Operator::LessThanOrEqual
            | Operator::GreaterThan
            | Operator::GreaterThanOrEqual
            | Operator::And
            | Operator::Or
            | Operator::Modulo
            | Operator::LossyEqual => None,
        }
    }
}

#[derive(
    Debug, Display, Default, PartialEq, Clone, Serialize, Deserialize, EnumString, EnumIter,
)]
pub enum Keyword {
    /// `return`
    #[default]
    #[display(fmt = "return")]
    #[strum(serialize = "return")]
    Return,

    /// `if`
    #[display(fmt = "if")]
    #[strum(serialize = "if")]
    If,

    /// `else`
    #[display(fmt = "else")]
    #[strum(serialize = "else")]
    Else,

    /// `def`
    #[display(fmt = "def")]
    #[strum(serialize = "def")]
    Def,

    /// `let`
    #[display(fmt = "let")]
    #[strum(serialize = "let")]
    Let,

    /// `mut`
    #[display(fmt = "mut")]
    #[strum(serialize = "mut")]
    Mut,

    /// `class`
    #[display(fmt = "class")]
    #[strum(serialize = "class")]
    Class,

    /// `for`
    #[display(fmt = "for")]
    #[strum(serialize = "for")]
    For,

    /// `while`
    #[display(fmt = "while")]
    #[strum(serialize = "while")]
    While,

    /// `extern`
    #[display(fmt = "extern")]
    #[strum(serialize = "extern")]
    Extern,

    /// `as`
    #[display(fmt = "as")]
    #[strum(serialize = "as")]
    As,

    /// `import`
    #[display(fmt = "import")]
    #[strum(serialize = "import")]
    Import,
}

#[derive(Debug, Display, Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum Literal {
    #[default]
    #[display(fmt = "void")]
    Void,
    #[display(fmt = "{:?}", "_0")]
    Integer(isize),
    #[display(fmt = "{:?}", "_0")]
    Float(f64),
    #[display(fmt = "{:?}", "_0")]
    Boolean(bool),
    #[display(fmt = "{:?}", "_0")]
    ArrayLiteral(ArrayLiteral),
    #[display(fmt = "{:?}", "_0")]
    Custom(ClassLiteral),

    // #[display(fmt="{:?}", "_0")]
    DataType {
        value_type: Box<DataType>,
    },
}

impl DataTypeGetter for Literal {
    fn _type(&self) -> DataType {
        match self {
            Literal::Void => DataType::None,
            Literal::Integer(_) => DataType::get_integer_type(),
            Literal::Float(_) => DataType::Float,
            Literal::Boolean(_) => DataType::Boolean,
            Literal::ArrayLiteral(array_literal) => DataType::Array {
                value_type: Box::new(array_literal.value_type.clone()),
                len: array_literal.values.value.len(),
            },
            Literal::Custom(class_literal) => class_literal._type.clone(),
            Literal::DataType { .. } => DataType::DataType,
        }
    }
}

impl DataTypeSetter for Literal {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        match self {
            Literal::Void => vec![],
            Literal::Integer(_) => vec![],
            Literal::Float(_) => vec![],
            Literal::Boolean(_) => vec![],
            Literal::ArrayLiteral(array_literal) => array_literal._type_mut(),
            Literal::Custom(class_literal) => class_literal._type_mut(),
            Literal::DataType { value_type } => value_type._type_mut(),
        }
    }
}

impl FromStr for Literal {
    type Err = TokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(integer) = s.parse::<isize>() {
            return Ok(Literal::Integer(integer));
        }

        if let Ok(float) = s.parse::<f64>() {
            return Ok(Literal::Float(float));
        }

        if s == "true" {
            return Ok(Literal::Boolean(true));
        }

        if s == "false" {
            return Ok(Literal::Boolean(false));
        }

        Err(TokenError::UnknownToken(s.to_owned()))
    }
}

#[derive(
    Debug, Display, Default, PartialEq, Clone, Serialize, Deserialize, EnumString, EnumIter,
)]
pub enum ReassignmentOperator {
    /// `+=`
    #[default]
    #[display(fmt = "+=")]
    #[strum(serialize = "+=")]
    PlusEqual,

    /// `-=`
    #[display(fmt = "-=")]
    #[strum(serialize = "-=")]
    MinusEqual,
}

impl Token {
    pub fn is_reassignment_operator(&self) -> bool {
        matches!(
            self,
            Token::Assignment
                | Token::ReassignmentOperator(ReassignmentOperator::PlusEqual)
                | Token::ReassignmentOperator(ReassignmentOperator::MinusEqual)
        )
    }
}
