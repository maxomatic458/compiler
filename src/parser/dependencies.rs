use std::path::Path;

use crate::{
    error::{emit_error, CompilerError},
    lexer::{
        position::Spanned,
        tokens::{Keyword, Token},
    },
    parser::ast::Program,
};

use super::{error::ParserError, parser_main::Parser};

impl Parser {
    pub(in crate::parser) fn parse_import(&mut self) -> Result<(), Spanned<ParserError>> {
        let _span = self.expect_next(&[Token::Keyword(Keyword::Import)])?.span;
        let name = self.expect_next(&[Token::String("file name".to_string())])?;

        if let Spanned {
            value: Token::String(name),
            span,
        } = name
        {
            let path = self
                .relative_path
                .as_ref()
                .unwrap_or(&std::env::current_dir().unwrap())
                .join(&name);

            let program = self.parse_dependency(&path, Spanned { value: name, span })?;
            self.add_dependency(program)?;

            return Ok(());
        }

        unreachable!()
    }

    fn parse_dependency(
        &mut self,
        path: &Path,
        name: Spanned<String>,
    ) -> Result<Program, Spanned<ParserError>> {
        let path = std::fs::canonicalize(path).unwrap();
        if self.program.import_queue.contains(&path.to_path_buf()) {
            return Err(Spanned {
                value: ParserError::CircularDependency(name.value),
                span: name.span,
            });
        }

        self.add_dependency_to_queue(path.to_path_buf());

        // TODO: muss vielleicht garnicht mehr geparsed werden
        // println!("path: {path:?}");
        if let Some(cached) = self.program.get_cached_dependency(&path) {
            // println!("HIT: {path:?}");
            return Ok(cached.clone());
        }

        let code = std::fs::read_to_string(&path).map_err(|_| Spanned {
            value: ParserError::FileNotFound(name.value),
            span: name.span,
        })?;

        let tokens = crate::lexer::lexer_main::lex(&code).map_err(|err| Spanned {
            value: ParserError::UnexpectedEOF,
            span: err.span,
        })?;

        let mut parser = Parser::new(tokens, Some(&path))
            .with_dependency_cache(self.program.dependency_cache.clone())
            .with_import_queue(self.program.import_queue.clone())
            .with_relative_path(Some(path.parent().unwrap().to_path_buf()));

        let program = match parser.parse() {
            Ok(program) => program,
            Err(err) => {
                let exit_code = err.value.id().try_into().unwrap_or(-1);
                emit_error(
                    path.to_str().unwrap(),
                    &code,
                    &Spanned {
                        value: Box::new(err.value),
                        span: err.span,
                    },
                );
                std::process::exit(exit_code);
            }
        };

        self.pop_dependency_queue();

        self.program
            .cache_dependency(path.to_path_buf(), program.clone());
        Ok(program)
    }

    fn add_dependency(&mut self, dependency: Program) -> Result<(), Spanned<ParserError>> {
        for (name, function) in dependency.functions {
            if let Some(function_here) = self.program.functions.get_mut(&name) {
                // dependency reexportiert bereits vorhandene Funktion
                // könnte man vielleicht besser lösen
                if function_here.value.import_compare(&function.value) || function.value.is_builtin
                {
                    for (specifics, subtype) in function.value.generic_subtypes {
                        if let Some(subtype_here) =
                            function_here.value.generic_subtypes.get_mut(&specifics)
                        {
                            if subtype_here.import_compare(&subtype) {
                                continue;
                            }

                            // return Err(Spanned {
                            //     value: ParserError::SubtypeAlreadyExists(specifics),
                            //     span: subtype.span,
                            // });
                        }

                        function_here
                            .value
                            .generic_subtypes
                            .insert(specifics, subtype);
                    }
                    continue;
                }

                // println!("{} {}", function_here.value, function.value);

                return Err(Spanned {
                    value: ParserError::FunctionAlreadyExists(name),
                    span: function.span,
                });
            }

            self.program.functions.insert(name, function);
        }

        for (name, class) in dependency.custom_types {
            if let Some(class_here) = self.program.custom_types.get(&name) {
                if class_here.value == class.value {
                    continue;
                }

                return Err(Spanned {
                    value: ParserError::ClassAlreadyExists(name),
                    span: class.span,
                });
            }

            self.program.custom_types.insert(name, class);
        }

        for (data_type, data_type_info) in dependency.data_types {
            self.program.data_types.insert(data_type, data_type_info);
        }

        Ok(())
    }
}
