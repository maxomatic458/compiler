use std::path::{Path, PathBuf};

use itertools::Itertools;

use crate::{
    codegen::{codegen_main::CodeGenerator, llvm_instructions::IR},
    error::{emit_error, CompilerError},
    lexer::{lexer_main::lex, position::Spanned},
    parser::parser_main::Parser,
};

pub struct Compiler;

impl Compiler {
    pub fn compile(
        input: &str,
        path: Option<PathBuf>,
    ) -> Result<IR, Spanned<Box<dyn CompilerError>>> {
        match Compiler::catch_errors(input, path.as_deref()) {
            Ok(ir) => Ok(ir),
            Err(e) => {
                let lines = input.split('\n').collect_vec();
                let _err_line = lines
                    .get(e.span.start.row.saturating_sub(3)..e.span.start.row + 2)
                    .unwrap_or_default()
                    .join("\n");

                emit_error(path.unwrap_or_default().to_str().unwrap(), input, &e);

                Err(e)
            }
        }
    }

    fn catch_errors(
        input: &str,
        path: Option<&Path>,
    ) -> Result<IR, Spanned<Box<dyn CompilerError>>> {
        let tokens = match lex(input) {
            Ok(tokens) => tokens,
            Err(e) => {
                return Err(Spanned {
                    value: Box::new(e.value),
                    span: e.span,
                })
            }
        };

        let program = match Parser::new(tokens, path)
            .with_source_code(input)
            .with_require_main(true)
            .parse()
        {
            Ok(program) => program,
            Err(e) => {
                return Err(Spanned {
                    value: Box::new(e.value),
                    span: e.span,
                })
            }
        };

        let mut codegen = CodeGenerator::new(program).with_source(input.to_string());
        let code = codegen.parse();
        Ok(code)
    }
}
