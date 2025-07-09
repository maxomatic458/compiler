pub mod codegen;
pub mod compiler;
pub mod error;
pub mod lexer;
pub mod parser;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    error::error_as_string,
    lexer::{position::Spanned, tokens::Token},
    parser::ast::{DataType, DataTypeInfo, Program},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompileResult {
    tokens: Vec<Spanned<Token>>,
    ast: ProgramSer,
    ir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgramSer {
    data_types: Vec<(String, DataTypeInfo)>,
    custom_types: Vec<(String, Spanned<DataType>)>,
    functions: Vec<Spanned<crate::parser::ast::Function>>,
    require_main: bool,
}

impl ProgramSer {
    pub fn from_program(program: &Program) -> Self {
        ProgramSer {
            data_types: program
                .data_types
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            custom_types: program
                .custom_types
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            functions: program.functions.values().cloned().collect(),
            require_main: program.require_main,
        }
    }
}

#[wasm_bindgen]
pub fn compile(input: &str) -> Result<JsValue, JsValue> {
    let tokens = lexer::lexer_main::lex(input).map_err(|e| {
        let boxed: Spanned<Box<dyn error::CompilerError>> = Spanned {
            span: e.span,
            value: Box::new(e.value),
        };
        JsValue::from_str(&error_as_string("input-file", input, &boxed))
    })?;

    let ast = parser::parser_main::Parser::new(tokens.clone(), None)
        .with_require_main(true)
        .parse()
        .map_err(|e| {
            let boxed: Spanned<Box<dyn error::CompilerError>> = Spanned {
                span: e.span,
                value: Box::new(e.value),
            };
            JsValue::from_str(&error_as_string("input-file", input, &boxed))
        })?;

    let ir = codegen::codegen_main::CodeGenerator::new(ast.clone()).parse();

    Ok(serde_wasm_bindgen::to_value(&CompileResult {
        tokens,
        ast: ProgramSer::from_program(&ast),
        ir,
    })
    .unwrap())
}
