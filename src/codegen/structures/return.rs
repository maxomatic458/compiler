use crate::{
    codegen::{
        codegen_main::CodeGenerator,
        llvm_instructions::{IRValue, Instruction},
    },
    parser::ast::TypedExpr,
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_return_statement(
        &mut self,
        value: &TypedExpr,
    ) -> Vec<Instruction> {
        let expr = self.parse_expression(value, false);

        let mut instructions = expr.instructions;

        instructions.push(Instruction::Return {
            expr: IRValue::Variable(expr.result_var),
        });

        instructions
    }
}
