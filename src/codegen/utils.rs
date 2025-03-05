use std::cmp::Ordering;

use crate::{
    lexer::tokens::Literal,
    parser::ast::{DataType, Expr},
};

use super::{
    codegen_main::{CodeGenerator, ComputedExpression},
    llvm_instructions::{IRValue, Instruction, MemoryOperation},
};

pub trait InstructionVecExt {
    fn move_allocations_to_top(&mut self);
    /// alle instructions nach dem letzten return entfernen
    fn kill_last_unused(&mut self);
}

impl InstructionVecExt for Vec<Instruction> {
    fn move_allocations_to_top(&mut self) {
        self.sort_by(|a, _| {
            if let Instruction::VRegisterAssignment { value, .. } = a {
                if let Instruction::MemoryOperation(MemoryOperation::Alloca { .. }) = **value {
                    return Ordering::Less;
                }
            }
            Ordering::Greater
        })
    }

    fn kill_last_unused(&mut self) {
        if !self.iter().any(|i| matches!(i, Instruction::Return { .. })) {
            return;
        }

        while let Some(last) = self.last() {
            if let Instruction::Return { .. } = last {
                break;
            }
            self.pop();
        }
    }
}

impl CodeGenerator {
    pub(in crate::codegen) fn size_of(&mut self, type_literal: Expr) -> ComputedExpression {
        if let Expr::Literal(Literal::DataType { value_type }) = type_literal {
            let result_var = self.next_tmp_var(&DataType::get_integer_type());
            let tmp_var = self.next_tmp_var(&DataType::Pointer(value_type.clone()));

            let instructions = vec![
                Instruction::VRegisterAssignment {
                    variable: tmp_var.clone(),
                    value: Box::new(Instruction::MemoryOperation(MemoryOperation::GetSizeOf {
                        _type: *value_type,
                    })),
                },
                Instruction::VRegisterAssignment {
                    variable: result_var.clone(),
                    value: Box::new(Instruction::MemoryOperation(MemoryOperation::PtrToInt {
                        pointer: IRValue::Variable(tmp_var),
                    })),
                },
            ];

            return ComputedExpression {
                instructions,
                result_var,
            };
        }

        unreachable!()
    }
}
