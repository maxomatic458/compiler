use crate::{
    error::CompilerError,
    lexer::tokens::{Operator, Token},
};
use thiserror::Error;

use super::ast::{DataType, Trait};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParserError {
    #[error("parser unexpectedly ran out of tokens")]
    UnexpectedEOF,
    /// ``expected`` ``got``
    #[error("expected: \"{0}\", got: \"{1}\"")]
    UnexpectedTokenExpected(Token, Token),
    /// ``token``
    #[error("unexpected token: \"{0}\"")]
    UnexpectedToken(Token),
    /// ``expected`` ``got``
    #[error("expected: \"{0}\", got: \"{1}\"")]
    WrongType(DataType, DataType),
    #[error("parameter with name \"{0}\" already exists")]
    ParamNameAlreadyExists(String),
    #[error("function with name \"{0}\" already exists")]
    FunctionAlreadyExists(String),
    /// ``method name`` ``class name``
    #[error("method with name \"{0}\" already exists for \"{1}\"")]
    MethodAlreadyExists(String, String),
    #[error("class with name \"{0}\" already exists")]
    ClassAlreadyExists(String),
    #[error("variable not found: \"{0}\"")]
    VariableNotFound(String),
    // #[error("function \"{0}\" does not return")]
    // MissingReturnStatement(String),
    #[error("expected: \"{0}\", got: \"{1}\"")]
    WrongReturnType(DataType, DataType),
    #[error("no main function found")]
    NoMainFunction,
    #[error("function \"{0}\" does not exist")]
    FunctionDoesNotExist(String),
    /// ``method name`` ``class name``
    #[error("method \"{0}\" does not exist for \"{1}\"")]
    MethodDoesNotExist(String, String),
    #[error("class \"{0}\" does not exist")]
    ClassDoesNotExist(String),
    /// ``expected`` ``got``
    #[error("expected: \"{0:?}\", got: \"{1:?}\"")]
    WrongArguments(Vec<String>, Vec<String>),
    /// ``expected`` ``got``
    #[error("expected: \"{0:?}\", got: \"{1:?}")]
    WrongClassFields(Vec<String>, Vec<String>),
    /// ``field name`` ``class name``
    #[error("field {0} does not exist on class {1}")]
    ClassFieldDoesNotExist(String, String),
    #[error("field with name \"{0}\" already exists on this class")]
    FieldNameAlreadyExists(String),
    #[error("an array is not allowed to be empty")]
    EmptyArray,
    #[error("cannot index type \"{0}\"")]
    IndexError(DataType),
    #[error("variable \"{0}\" was not defined as mutable")]
    VariableNotMutable(String),
    #[error("invalid reassign")]
    InvalidReassign,
    /// ``from`` ``to``
    #[error("type \"{0}\" can not be casted to \"{1}\"")]
    InvalidCast(DataType, DataType),
    /// ``method name`` ``class name``
    #[error("method \"{0}\" of \"{1}\" is not static")]
    MethodIsNotStatic(String, String),
    /// ``method name`` ``class name``
    #[error("static method \"{0}\" of \"{1}\" was not called statically")]
    MethodIsStatic(String, String),
    #[error("cannot access fields of type \"{0}\"")]
    CannotAccessFields(DataType),
    #[error("cannot dereference type \"{0}\"")]
    CannotDerefType(DataType),
    #[error("conditional return mismatch")]
    ConditionalReturnMismatch,
    #[error("file \"{0}\" not found")]
    FileNotFound(String),
    #[error("macro error: {0}")]
    MacroError(String),
    #[error("invalid operator: {0}")]
    InvalidOperator(Operator),
    #[error("trait implementation must not be a static method")]
    TraitIsStaticMethod,
    #[error(
        "trait \"{0}\" requires {1} parameters, but the implementation has {2} (excluding self)"
    )]
    TraitParamCountMismatch(Trait, usize, usize),
    #[error("trait \"{0}\" is already implemented for type \"{1}\"")]
    TraitAlreadyImplemented(Trait, DataType),
    #[error("trait does not fullfull these requirements: {0}")]
    TraitRequirementsNotFulfilled(String),
    #[error("circular dependency: \"{0}\"")]
    CircularDependency(String),
    #[error("expected: \"{0}\", got: \"{1}\"")]
    WrongGenericParamCount(usize, usize),
    #[error("variable can not have type \"void\"")]
    VoidVariable,
}

impl CompilerError for ParserError {
    fn id(&self) -> u32 {
        match self {
            ParserError::UnexpectedEOF => 2,
            ParserError::UnexpectedTokenExpected(..) => 3,
            ParserError::UnexpectedToken(..) => 4,
            ParserError::WrongType(..) => 5,
            ParserError::ParamNameAlreadyExists(..) => 6,
            ParserError::FunctionAlreadyExists(..) => 7,
            ParserError::MethodAlreadyExists(_, _) => 8,
            ParserError::VariableNotFound(..) => 9,
            // ParserError::MissingReturnStatement(..) => 10,
            ParserError::WrongReturnType(..) => 11,
            ParserError::NoMainFunction => 12,
            ParserError::FunctionDoesNotExist(..) => 13,
            ParserError::MethodDoesNotExist(..) => 14,
            ParserError::WrongArguments(..) => 15,
            ParserError::FieldNameAlreadyExists(..) => 16,
            ParserError::ClassAlreadyExists(..) => 17,
            ParserError::ClassDoesNotExist(..) => 18,
            ParserError::ClassFieldDoesNotExist(..) => 19,
            ParserError::WrongClassFields(..) => 20,
            ParserError::EmptyArray => 21,
            ParserError::IndexError(..) => 22,
            ParserError::VariableNotMutable(..) => 23,
            ParserError::InvalidReassign => 24,
            ParserError::InvalidCast(..) => 25,
            ParserError::MethodIsNotStatic(..) => 26,
            ParserError::MethodIsStatic(..) => 27,
            ParserError::CannotAccessFields(..) => 28,
            ParserError::CannotDerefType(..) => 29,
            ParserError::ConditionalReturnMismatch => 30,
            ParserError::FileNotFound(_) => 31,
            ParserError::MacroError(_) => 32,
            ParserError::InvalidOperator(_) => 33,
            ParserError::TraitIsStaticMethod => 34,
            ParserError::TraitParamCountMismatch(_, _, _) => 35,
            ParserError::TraitAlreadyImplemented(_, _) => 36,
            ParserError::TraitRequirementsNotFulfilled(_) => 37,
            ParserError::CircularDependency(_) => 38,
            ParserError::WrongGenericParamCount(_, _) => 39,
            ParserError::VoidVariable => 40,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ParserError::UnexpectedEOF => "unexpected EOF (end of file)",
            ParserError::UnexpectedTokenExpected(..) => "unexpected token",
            ParserError::UnexpectedToken(..) => "unexpected token",
            ParserError::WrongType(..) => "wrong type",
            ParserError::ParamNameAlreadyExists(..) => "parameter name already used",
            ParserError::FunctionAlreadyExists(..) => "function name already used",
            ParserError::MethodAlreadyExists(..) => "method already defined for class",
            ParserError::VariableNotFound(..) => "unknown variable",
            // ParserError::MissingReturnStatement(..) => "missing return statement",
            ParserError::WrongReturnType(..) => "wrong return type",
            ParserError::NoMainFunction => "missing main function",
            ParserError::FunctionDoesNotExist(..) => "function does not exist",
            ParserError::MethodDoesNotExist(..) => "method does not exist",
            ParserError::WrongArguments(..) => "wrong function arguments",
            ParserError::FieldNameAlreadyExists(..) => "field name already used",
            ParserError::ClassAlreadyExists(..) => "class name already used",
            ParserError::ClassDoesNotExist(..) => "class does not exist",
            ParserError::ClassFieldDoesNotExist(..) => "field does not exist",
            ParserError::WrongClassFields(..) => "wrong class fields",
            ParserError::EmptyArray => "array can not be empty",
            ParserError::IndexError(..) => "unable to index this type",
            ParserError::VariableNotMutable(..) => "variable is immutable",
            ParserError::InvalidReassign => "reassignment is not valid",
            ParserError::InvalidCast(..) => "invalid type cast",
            ParserError::MethodIsNotStatic(..) => "method was called statically",
            ParserError::MethodIsStatic(..) => "static method was not called statically",
            ParserError::CannotAccessFields(..) => "cannot access fields of this type",
            ParserError::CannotDerefType(..) => "cannot dereference this type",
            ParserError::ConditionalReturnMismatch => {
                "the branches of this conditional return different types"
            }
            ParserError::FileNotFound(_) => "file not found",
            ParserError::MacroError(_) => "macro error",
            ParserError::InvalidOperator(_) => "invalid operator",
            ParserError::TraitIsStaticMethod => "trait implementation must not be a static method",
            ParserError::TraitParamCountMismatch(_, _, _) => "trait parameter count mismatch",
            ParserError::TraitAlreadyImplemented(_, _) => "trait already implemented for type",
            ParserError::TraitRequirementsNotFulfilled(_) => "trait requirements not fulfilled",
            ParserError::CircularDependency(_) => "circular dependency",
            ParserError::WrongGenericParamCount(_, _) => "wrong generic parameter count",
            ParserError::VoidVariable => "variable can not have type void",
        }
    }

    fn err_msg(&self) -> String {
        self.to_string()
    }
}
