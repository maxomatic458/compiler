use itertools::Itertools;

use crate::{
    lexer::{
        position::Spanned,
        tokens::{Keyword, Token},
    },
    parser::{
        ast::{Block, DataType, ElseIfBranch, Statement},
        error::ParserError,
        parser_main::{validate_block_return, Parser},
    },
};

impl Parser {
    pub(in crate::parser) fn parse_if(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<Statement>, Spanned<ParserError>> {
        scope.function_depth += 1;
        let mut span = self.expect_next(&[Token::Keyword(Keyword::If)])?.span;
        let condition = self.parse_expression(scope)?;

        if condition.value._type != DataType::Boolean {
            return Err(Spanned {
                value: ParserError::WrongType(DataType::Boolean, condition.value._type),
                span: span.extend(&condition.span),
            });
        }

        let true_block = self.parse_block(scope)?;
        let mut false_block = None;

        let mut elifs = vec![];

        loop {
            match (self.peek()?, self.peek_nth(1)?) {
                (
                    Spanned {
                        value: Token::Keyword(Keyword::Else),
                        span: _else_span,
                    },
                    Spanned {
                        value: Token::Keyword(Keyword::If),
                        span: if_span,
                    },
                ) => {
                    span = span.extend(&if_span);
                    self.next_token()?;
                    self.next_token()?;

                    let condition = self.parse_expression(scope)?;
                    let body = self.parse_block(scope)?;

                    elifs.push(Spanned {
                        value: ElseIfBranch {
                            condition,
                            body: body.clone(),
                        },
                        span: span.extend(&body.span),
                    });
                }

                (
                    Spanned {
                        value: Token::Keyword(Keyword::Else),
                        span: else_span,
                    },
                    _,
                ) => {
                    span = span.extend(&else_span);
                    self.next_token()?;
                    false_block = Some(self.parse_block(scope)?);
                    break;
                }

                _ => {
                    break;
                }
            }
        }

        Ok(Spanned {
            value: Statement::If {
                condition,
                true_branch: Box::new(true_block),
                else_if_branches: elifs,
                false_branch: false_block.map(Box::new),
            },
            span,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchReturn {
    AllReturn,
    SomeReturn,
    NoneReturn,
}

pub fn validate_if_return(
    true_branch: &Spanned<Block>,
    else_if_branches: &[Spanned<ElseIfBranch>],
    false_branch: &Option<Spanned<Block>>,
    // depth: usize,
) -> Result<(Option<DataType>, BranchReturn), Spanned<ParserError>> {
    let mut return_type = None;
    let mut all_return = true;
    let mut no_return = true;

    // println!("### depth: {}", depth);#

    let true_return = validate_block_return(true_branch)?;
    let false_return = false_branch
        .as_ref()
        .map(validate_block_return)
        .unwrap_or(Ok(None))?;
    let elif_returns = else_if_branches
        .iter()
        .map(|branch| validate_block_return(&branch.value.body))
        .collect::<Result<Vec<_>, _>>()?;

    let returns = vec![true_return]
        .into_iter()
        .chain(elif_returns)
        .chain(vec![false_return])
        .collect_vec();

    for return_ in returns {
        if let Some(return_) = &return_ {
            no_return = false;
            if let Some(return_type) = &return_type {
                if return_ != return_type {
                    return Err(Spanned {
                        value: ParserError::WrongType(return_type.clone(), return_.clone()),
                        span: true_branch.span,
                    });
                }
            } else {
                return_type = Some(return_.clone());
            }
        } else {
            all_return = false;
        }
    }

    Ok(match (all_return, no_return) {
        (true, _) => (return_type, BranchReturn::AllReturn),
        (false, true) => (None, BranchReturn::NoneReturn),
        (false, false) => (None, BranchReturn::SomeReturn),
    })
}
