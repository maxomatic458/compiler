use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use std::sync::RwLock;

use ahash::AHashMap;
use itertools::peek_nth;
use itertools::Itertools;
use itertools::PeekNth;
use std::vec::IntoIter;

use crate::lexer::position::Span;
use crate::lexer::tokens::Literal;
use crate::lexer::tokens::Operator;
use crate::lexer::tokens::{Keyword, Punctuation};
use crate::lexer::{position::Spanned, tokens::Token};
use crate::parser::ast::DataType;

use super::ast::CommonGeneric;

use super::ast::DataTypeInfo;

use super::ast::Trait;
use super::ast::{Block, Function, Program};
use super::structures::r#if::validate_if_return;
use super::structures::r#if::BranchReturn;
use super::utils::same_variant;
use super::{ast::Statement, error::ParserError};

#[derive(Debug, Clone)]
pub struct Parser {
    pub tokens: PeekNth<IntoIter<Spanned<Token>>>,
    pub program: Program,
    pub count: u32,
    pub relative_path: Option<PathBuf>,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>, path: Option<&Path>) -> Self {
        Parser {
            tokens: peek_nth(tokens),
            program: Program::default(),
            count: 0,
            relative_path: path.map(|p| p.to_path_buf()),
        }
    }

    pub fn with_source_code(mut self, source_code: &str) -> Self {
        self.program.source_code = source_code.chars().collect();
        self
    }

    pub fn with_require_main(mut self, require_main: bool) -> Self {
        self.program.require_main = require_main;
        self
    }

    pub fn with_dependency_cache(
        mut self,
        dependencies: Arc<RwLock<AHashMap<PathBuf, Program>>>,
    ) -> Self {
        self.program.dependency_cache = dependencies;
        self
    }

    pub fn with_import_queue(mut self, import_queue: Vec<PathBuf>) -> Self {
        self.program.import_queue = import_queue;
        self
    }

    pub fn with_relative_path(mut self, path: Option<PathBuf>) -> Self {
        self.relative_path = path;
        self
    }

    pub fn add_dependency_to_queue(&mut self, path: PathBuf) {
        self.program.add_dependency_to_queue(path)
    }

    pub fn pop_dependency_queue(&mut self) -> Option<PathBuf> {
        self.program.pop_dependency_queue()
    }

    pub fn cache_dependency(&mut self, path: PathBuf, program: Program) {
        self.program.cache_dependency(path, program)
    }

    pub fn get_cached_dependency(&self, path: &Path) -> Option<Program> {
        self.program.get_cached_dependency(path)
    }

    // pub fn to_string<S>(&self, spanned: Spanned<S>) -> String {

    // }

    pub fn parse(&mut self) -> Result<Program, Spanned<ParserError>> {
        while let Ok(Spanned { value, span }) = self.peek() {
            match value {
                Token::Keyword(Keyword::Def) | Token::Keyword(Keyword::Extern) => {
                    self.parse_func_def()?;
                }
                Token::Keyword(Keyword::Class) => {
                    self.parse_class_def()?;
                }
                Token::Keyword(Keyword::Import) => {
                    self.parse_import()?;
                }

                _ => {
                    return Err(Spanned {
                        value: ParserError::UnexpectedToken(value),
                        span,
                    })
                }
            };
        }

        if !self.program.functions.contains_key("main") && self.program.require_main {
            return Err(Spanned {
                value: ParserError::NoMainFunction,
                span: Span::default(),
            });
        }

        Ok(self.program.clone())
    }

    pub fn get_count(&mut self) -> u32 {
        self.count += 1;
        self.count
    }

    pub fn find_ahead<F: Fn(&Token) -> bool>(
        &mut self,
        tokens: Vec<Token>,
        stop_condition: F,
    ) -> Result<Option<usize>, Spanned<ParserError>> {
        let mut contains = false;
        let mut i = 0;
        loop {
            let next = self.peek_nth(i)?;
            if stop_condition(&next.value) {
                break;
            }
            if tokens.contains(&next.value) {
                contains = true;
                break;
            }

            i += 1;
        }

        if contains {
            Ok(Some(i))
        } else {
            Ok(None)
        }
    }

    pub fn parse_block(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<Block>, Spanned<ParserError>> {
        let start = self
            .expect_next(&[Token::Punctuation(Punctuation::OpenBrace)])?
            .span;

        let mut block = Spanned {
            value: Block {
                statements: vec![],
                return_type: DataType::None,
                variables: scope.variables.clone(),
                closure_params: scope
                    .variables
                    .values()
                    .map(|v| Spanned {
                        value: v.value.clone().into(),
                        span: v.span,
                    })
                    .collect_vec(),
                generics: scope.generics.clone(),
                function_depth: scope.function_depth,
            },
            span: start,
        };

        let end =
            self.walk_to_terminator(Token::Punctuation(Punctuation::CloseBrace), |parser| {
                let statement = parser.parse_statement(&mut block.value)?;
                block.value.statements.push(statement);
                Ok(())
            })?;

        block.span = block.span.extend(&end);

        block.value.return_type = validate_block_return(&block)?.unwrap_or_default();

        Ok(block)
    }

    pub fn parse_statement(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<Statement>, Spanned<ParserError>> {
        let next = self.peek()?;

        let mut requires_semicolon = true;
        let out = match next.value {
            Token::Keyword(Keyword::Let) => self.parse_variable_decl(scope)?,
            Token::Keyword(Keyword::Return) => self.parse_return(scope)?,
            Token::Keyword(Keyword::If) => self.parse_if(scope)?,
            Token::Keyword(Keyword::While) => self.parse_while(scope)?,
            _ => {
                let is_reassignment = self
                    .find_ahead(vec![Token::Assignment], |t| {
                        matches!(
                            t,
                            Token::Punctuation(Punctuation::SemiColon)
                                | Token::Punctuation(Punctuation::OpenBrace)
                        )
                    })?
                    .is_some();

                if is_reassignment {
                    self.parse_variable_reassignment(scope)?
                } else {
                    let expression = self.parse_expression(scope)?;
                    Spanned {
                        value: Statement::Expr(Spanned {
                            value: expression.value.expression,
                            span: expression.span,
                        }),
                        span: expression.span,
                    }
                }

                // let next_next = self.peek_nth(1)?;

                // if let Token::Punctuation(Punctuation::OpenParen) = next_next.value {
                //     let func_call = self.parse_func_call(scope.unwrap(), None)?;

                //     Spanned {
                //         value: Statement::Expr(Spanned {
                //             value: func_call.value.expression,
                //             span: func_call.span.clone(),
                //         }),
                //         span: func_call.span,
                //     }
                // } else {
                //     let is_reassignment = {
                //         let mut i = 0;
                //         let mut is_reassignment = false;
                //         while let Ok(token) = self.peek_nth(i) {
                //             if token.value.is_reassignment_operator() {
                //                 is_reassignment = true;
                //                 break
                //             }

                //             if Token::Punctuation(Punctuation::SemiColon) == token.value {
                //                 break
                //             }
                //             i += 1;
                //         }
                //         is_reassignment
                //     };

                //     if is_reassignment {
                //         self.parse_variable_reassignment(scope.unwrap())?
                //     } else {
                //         return Err(Spanned {
                //             value: ParserError::UnexpectedToken(next.value),
                //             span: next.span,
                //         })
                //     }
                // }
            } // unexpected => return Err(Spanned {
              //     value: ParserError::UnexpectedToken(unexpected),
              //     span: next.span,
              // })
        };

        if let Token::Keyword(Keyword::If) | Token::Keyword(Keyword::While) = next.value {
            requires_semicolon = false;
        }

        match self.peek()?.value {
            Token::Punctuation(Punctuation::SemiColon) => {
                self.next_token()?;
            }
            Token::Punctuation(Punctuation::CloseBrace) => {}
            _ => {
                if requires_semicolon {
                    self.expect_next(&[Token::Punctuation(Punctuation::SemiColon)])?;
                }
            }
        };

        Ok(out)
    }
    // generics: &Option<Vec<Spanned<DataType>>>
    pub(super) fn parse_data_type(
        &mut self,
        generics: Option<&Vec<Spanned<DataType>>>,
    ) -> Result<Spanned<DataType>, Spanned<ParserError>> {
        let next = self.next_token()?;

        match &next.value {
            // TODO: Datatype::from_str ?
            Token::Operator(Operator::Multiply) => {
                let inner_type = self.parse_data_type(generics)?;
                // dbg!(inner_type.clone());
                Ok(Spanned {
                    value: DataType::Pointer(Box::new(inner_type.value)),
                    span: next.span.extend(&inner_type.span),
                })
            }

            Token::Identifier(data_type_name) => {
                if let Some(ref mut class) = self.program.custom_types.get(data_type_name).cloned()
                {
                    if let DataType::Custom(custom_type) = &class.value {
                        if custom_type.is_generic()
                            && self.peek()?.value == Token::Operator(Operator::LessThan)
                        {
                            // let mut span = self.expect_next(&[Token::Operator(Operator::LessThan)])?.span; // <
                            let types = self.collect_generic_annotations(generics)?;
                            let unspanned_types =
                                types.iter().map(|t| t.value.clone()).collect::<Vec<_>>();

                            let subtype = match custom_type.subtypes.get(&unspanned_types) {
                                Some(subtype) => subtype.clone(),
                                None => {
                                    // println!("<<>> unspanned: {:?}", unspanned_types);
                                    custom_type.subtype(&unspanned_types, &mut self.program, true)
                                }
                            };

                            // self.program.classes.insert(subtype.name.clone(), Spanned {
                            //     value: parser::ast:DataType::Custom(subtype.clone()),
                            //     span: next.span.clone(),
                            // });

                            // let class_ref = self.program.classes.get_mut(data_type_name).unwrap();

                            // if let DataType::Custom(ref mut inner) = class_ref.value {
                            //     inner.subtypes.insert(unspanned_types, subtype.clone());
                            // }
                            // let class_mut = self.program.custom_types.get_mut(data_type_name).unwrap();
                            // panic!("hg");

                            *class = Spanned {
                                value: DataType::Custom(subtype),
                                span: class.span,
                            }
                        }

                        return Ok(Spanned {
                            value: class.value.clone(),
                            span: next.span,
                        });
                    }
                }

                if let Some(generics) = generics {
                    if let Some(generic) = generics.iter().find(|g| {
                        if let DataType::Generic(name) = &g.value {
                            return name == data_type_name;
                        }
                        false
                    }) {
                        return Ok(Spanned {
                            value: generic.value.clone(),
                            span: next.span,
                        });
                    }
                }

                Ok(Spanned {
                    value: DataType::from_str(data_type_name).map_err(|_| Spanned {
                        value: ParserError::UnexpectedToken(next.value),
                        span: next.span,
                    })?,
                    span: next.span,
                })
            }

            Token::Punctuation(Punctuation::OpenBracket) => {
                let value_type = self.parse_data_type(generics)?;
                self.expect_next(&[Token::Punctuation(Punctuation::SemiColon)])?;
                if let Spanned {
                    value: Token::DataLiteral(data_literal),
                    span: data_literal_span,
                } = self.expect_next(&[Token::DataLiteral(Literal::Integer(0))])?
                {
                    let end_span = self
                        .expect_next(&[Token::Punctuation(Punctuation::CloseBracket)])?
                        .span;

                    if let Literal::Integer(len) = data_literal {
                        if len < 1 {
                            return Err(Spanned {
                                value: ParserError::EmptyArray,
                                span: data_literal_span,
                            });
                        }

                        return Ok(Spanned {
                            value: DataType::Array {
                                value_type: Box::new(value_type.value),
                                len: len.try_into().unwrap(),
                            },
                            span: next.span.extend(&end_span),
                        });
                    }
                }

                todo!()
            }

            unexpected => Err(Spanned {
                value: ParserError::UnexpectedToken(unexpected.clone()),
                span: next.span,
            }),
        }
    }

    pub(super) fn collect_generic_annotations(
        &mut self,
        generics: Option<&Vec<Spanned<DataType>>>,
    ) -> Result<Vec<Spanned<DataType>>, Spanned<ParserError>> {
        let mut _span = self
            .expect_next(&[Token::Operator(Operator::LessThan)])?
            .span;
        let mut generic_annotations = vec![];

        let _end = self.walk_separated_values(
            Token::Punctuation(Punctuation::Comma),
            Token::Operator(Operator::GreaterThan),
            |parser| {
                let _type = parser.parse_data_type(generics)?;
                _span.extend(&_type.span);
                generic_annotations.push(_type);
                Ok(())
            },
        )?;

        Ok(generic_annotations)
    }

    pub(super) fn get_type_from_name(
        &mut self,
        name: &Spanned<String>,
    ) -> Result<Spanned<DataType>, Spanned<ParserError>> {
        if let Ok(_type) = DataType::from_str(&name.value) {
            return Ok(Spanned {
                value: _type,
                span: name.span,
            });
        }

        match self.program.custom_types.get(&name.value) {
            Some(class) => Ok(class.clone()),
            None => Err(Spanned {
                value: ParserError::ClassDoesNotExist(name.value.clone()),
                span: name.span,
            }),
        }
    }

    pub(super) fn get_type_info(&mut self, _type: &DataType) -> DataTypeInfo {
        self.program.get_type_info(_type)
    }

    pub(super) fn get_type_info_mut(&mut self, _type: &DataType) -> &mut DataTypeInfo {
        self.program.get_type_info_mut(_type)
    }

    // pub(super) fn get_traits(&mut self, _type: &DataType) -> Vec<Trait> {
    //     self.get_type_info(_type)
    //         .get_traits()
    // }

    pub(super) fn implements_trait(
        &mut self,
        _type: &DataType,
        _trait: &Trait,
        params: &[DataType],
    ) -> bool {
        self.get_type_info(_type).implements_trait(_trait, params)
    }

    pub(super) fn get_function(
        &mut self,
        name: &Spanned<String>,
    ) -> Result<&mut Spanned<Function>, Spanned<ParserError>> {
        match self.program.functions.get_mut(&name.value) {
            Some(function) => Ok(function),
            None => Err(Spanned {
                value: ParserError::FunctionDoesNotExist(name.value.clone()),
                span: name.span,
            }),
        }
    }

    pub(super) fn next_token(&mut self) -> Result<Spanned<Token>, Spanned<ParserError>> {
        match self.tokens.next() {
            Some(token) => Ok(token),
            None => Err(Spanned {
                value: ParserError::UnexpectedEOF,
                span: Span::default(),
            }),
        }
    }

    pub(super) fn peek(&mut self) -> Result<Spanned<Token>, Spanned<ParserError>> {
        self.peek_nth(0)
    }

    pub(super) fn peek_nth(&mut self, nth: usize) -> Result<Spanned<Token>, Spanned<ParserError>> {
        match self.tokens.peek_nth(nth).cloned() {
            Some(token) => Ok(token),
            None => Err(Spanned {
                value: ParserError::UnexpectedEOF,
                span: Span::default(),
            }),
        }
    }

    pub(super) fn expect_next(
        &mut self,
        expected: &[Token],
    ) -> Result<Spanned<Token>, Spanned<ParserError>> {
        let token = self.next_token()?;
        if expected.iter().any(|t| same_variant(t, &token.value)) {
            return Ok(token);
        }

        if expected.len() == 1 {
            return Err(Spanned {
                value: ParserError::UnexpectedTokenExpected(expected[0].clone(), token.value),
                span: token.span,
            });
        }

        Err(Spanned {
            value: ParserError::UnexpectedToken(token.value),
            span: token.span,
        })
    }

    /// konsumiert den terminator
    pub(in crate::parser) fn walk_separated_values<F>(
        &mut self,
        separator: Token,
        terminator: Token,
        mut callback: F,
    ) -> Result<Span, Spanned<ParserError>>
    where
        F: FnMut(&mut Parser) -> Result<(), Spanned<ParserError>>,
    {
        let mut span = self.peek()?.span;

        if self.peek()?.value == terminator {
            self.next_token()?;
            return Ok(span);
        }

        callback(self)?;

        loop {
            let token = self.peek()?;
            if token.value == terminator {
                span = span.extend(&token.span);
                self.next_token()?;
                break;
            }

            self.next_token()?;
            if token.value == separator {
                if self.peek()?.value == terminator {
                    span = span.extend(&token.span);
                    self.next_token()?;
                    break;
                }
                callback(self)?;
            }
        }

        Ok(span)
    }
    /// konsumiert den terminator
    pub(in crate::parser) fn walk_to_terminator<F>(
        &mut self,
        terminator: Token,
        mut callback: F,
    ) -> Result<Span, Spanned<ParserError>>
    where
        F: FnMut(&mut Parser) -> Result<(), Spanned<ParserError>>,
    {
        let mut span = self.peek()?.span;

        if self.peek()?.value == terminator {
            self.next_token()?;
            return Ok(span);
        }

        callback(self)?;

        loop {
            let token = self.peek()?;
            if token.value == terminator {
                span = span.extend(&token.span);
                self.next_token()?;
                break;
            }
            callback(self)?;
        }

        Ok(span)
    }
}
/// return:
/// None -> der Block hat kein return statement
pub(in crate::parser) fn validate_block_return(
    block: &Spanned<Block>,
) -> Result<Option<DataType>, Spanned<ParserError>> {
    let mut return_type: Option<DataType> = None;
    let mut conditional_return = None;

    let mut error = None;

    block.value.walk(&mut |statement| match statement {
        Statement::Return { value } => {
            if let Some(return_type) = &return_type {
                if *return_type != value.value._type {
                    error = Some(Spanned {
                        value: ParserError::WrongType(
                            return_type.clone(),
                            value.value._type.clone(),
                        ),
                        span: value.span,
                    });
                    return;
                }
            } else {
                return_type = Some(value.value._type.clone());
            }
            conditional_return = None;
        }
        Statement::If {
            true_branch,
            else_if_branches,
            false_branch,
            ..
        } => {
            let (if_return_type, conditional) = match validate_if_return(
                true_branch,
                else_if_branches,
                &false_branch.clone().map(|b| *b),
                // block.value.function_depth,
            ) {
                Ok((return_type, BranchReturn::AllReturn)) => (return_type, false),
                Ok((return_type, BranchReturn::SomeReturn)) => {
                    (return_type, block.value.function_depth < 2)
                } // TODO: ? nur äußerste if
                Ok((return_type, BranchReturn::NoneReturn)) => (return_type, false),

                Err(err) => {
                    error = Some(err);
                    return;
                }
            };

            if let Some(if_return_type) = if_return_type {
                if let Some(return_type) = &return_type {
                    if *return_type != if_return_type {
                        error = Some(Spanned {
                            value: ParserError::WrongType(return_type.clone(), if_return_type),
                            span: block.span,
                        });
                        return;
                    }
                } else {
                    return_type = Some(if_return_type);
                }
            }

            if conditional {
                conditional_return = Some(
                    true_branch.span.extend(
                        &false_branch
                            .clone()
                            .map(|b| b.span)
                            .unwrap_or(true_branch.span),
                    ),
                );
            }
        }
        _ => {}
    });

    // if block.value.function_depth < 2 {
    if let Some(span) = conditional_return {
        error = Some(Spanned {
            value: ParserError::ConditionalReturnMismatch,
            span,
        })
    }
    // }

    if let Some(err) = error {
        return Err(err);
    }

    Ok(return_type)
}
