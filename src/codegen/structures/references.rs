use crate::{
    codegen::{
        codegen_main::{CodeGenerator, ComputedExpression},
        llvm_instructions::{IRValue, IRVariable, Instruction, MemoryOperation},
    },
    parser::ast::{DataType, TypedExpr},
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_reference(&mut self, expr: &TypedExpr) -> ComputedExpression {
        let mut instructions = vec![];
        let result_var = self.next_tmp_var(&DataType::Pointer(Box::new(expr._type.clone())));

        let mut computed = self.parse_expression(expr, true);

        instructions.append(&mut computed.instructions);

        instructions.append(&mut vec![
            Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(Instruction::MemoryOperation(MemoryOperation::Alloca {
                    _type: DataType::Pointer(Box::new(expr._type.clone())),
                })),
            },
            Instruction::MemoryOperation(MemoryOperation::Store {
                value: IRValue::Variable(IRVariable {
                    name: computed.result_var.name.clone(),
                    _type: DataType::Pointer(Box::new(expr._type.clone())),
                }),
                pointer: result_var.clone(),
            }),
        ]);

        ComputedExpression {
            instructions,
            result_var,
        }
    }
}
