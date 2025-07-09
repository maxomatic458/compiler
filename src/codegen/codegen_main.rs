use std::{collections::HashMap, sync::atomic::AtomicU64};

use itertools::Itertools;

use super::llvm_instructions::{ToIR, IR};
use crate::{
    codegen::llvm_instructions::{IRVariable, Instruction},
    lexer::position::Spanned,
    parser::ast::{DataType, Expr, Program, Statement},
};

#[derive(Debug, Clone)]
pub struct ComputedExpression {
    pub instructions: Vec<Instruction>,
    pub result_var: IRVariable,
}

impl ToIR for ComputedExpression {
    fn to_ir(&self) -> IR {
        self.instructions.to_ir()
    }
}

pub struct CodeGenerator {
    pub program: Program,
    pub variable_counter: AtomicU64,
    pub variable_map: HashMap<String, IRVariable>,
    pub source_code: Option<String>,
}

impl CodeGenerator {
    pub fn new(program: Program) -> Self {
        CodeGenerator {
            program,
            variable_counter: AtomicU64::new(0),
            variable_map: HashMap::new(),
            source_code: None,
        }
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source_code = Some(source);
        self
    }

    pub fn parse(&mut self) -> IR {
        let mut instructions = vec![];

        // klassen und funktionen können während dem durchlauf hinzugefügt werden (anonyme funktionen)
        // TODO: vielleicht multithreading?
        let mut i = 0;
        while i < self.program.custom_types.len() {
            let (_key, class) = self.program.custom_types.get_index(i).unwrap();
            instructions.append(&mut self.parse_class_def(class.value.clone()));
            i += 1;
        }

        let mut i = 0;
        while i < self.program.functions.len() {
            let (_key, function) = self.program.functions.get_index(i).unwrap();
            instructions.append(&mut self.parse_func_def(function.value.clone()));
            i += 1;
        }

        instructions.to_ir()
    }

    pub(super) fn next_count(&self) -> u64 {
        self.variable_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub(super) fn next_var_name(&self) -> String {
        format!("%_{}", self.next_count())
    }

    pub(super) fn next_tmp_var(&self, _type: &DataType) -> IRVariable {
        IRVariable {
            name: self.next_var_name(),
            _type: _type.clone(),
        }
    }

    pub fn parse_statement(&mut self, statement: &Spanned<Statement>) -> Vec<Instruction> {
        let mut comment = self.source_code.as_ref().map(|s| {
            Instruction::Comment(
                s.get(statement.span.start.abs..statement.span.end.abs)
                    .unwrap_or_default()
                    .to_string(),
            )
        }); //TODO: macro kommentare

        let mut instructions = match &statement.value {
            Statement::If {
                condition,
                true_branch,
                else_if_branches,
                false_branch,
            } => {
                comment = None;

                self.parse_if(
                    &condition.value,
                    &true_branch.value,
                    else_if_branches,
                    false_branch.as_ref().map(|x| &x.value),
                )
            }
            Statement::VariableDecl { name, value, .. } => {
                self.parse_variable_decl(name.value.clone(), &value.value)
            }
            Statement::Return { value } => self.parse_return_statement(&value.value),
            Statement::Expr(expr) => {
                if let Expr::Call { function, args } = &expr.value {
                    return self
                        .parse_func_call(
                            &function.value,
                            &args.value.iter().map(|x| x.clone().value).collect_vec(),
                        )
                        .instructions;
                }
                unreachable!()
            }
            Statement::VariableMutation {
                variable,
                new_value,
            } => self.parse_variable_mutation(&variable.value, &new_value.value),
            Statement::WhileLoop { condition, body } => {
                self.parse_while(&condition.value, &body.value)
            }
        };

        if let Some(comment) = comment {
            instructions.insert(0, comment)
        }

        instructions
    }
}
