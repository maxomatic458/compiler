use crate::{
    codegen::{
        codegen_main::{CodeGenerator, ComputedExpression},
        llvm_instructions::{IRValue, Instruction, MemoryOperation},
    },
    lexer::tokens::Literal,
    parser::ast::{ClassLiteral, CommonGeneric, CustomDataType, DataType, TypedExpr},
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_class_def(&self, class: DataType) -> Vec<Instruction> {
        if let DataType::Custom(custom) = &class {
            if custom.is_generic() {
                return vec![];
            }

            return vec![Instruction::DeclareType(class)];
        }

        unreachable!()
    }

    pub(in crate::codegen) fn parse_class_literal(
        &mut self,
        class_literal: &ClassLiteral,
    ) -> ComputedExpression {
        let result_var = self.next_tmp_var(&class_literal._type);
        let mut instructions = vec![];

        instructions.append(&mut vec![Instruction::VRegisterAssignment {
            variable: result_var.clone(),
            value: Box::new(Instruction::MemoryOperation(MemoryOperation::Alloca {
                _type: class_literal._type.clone(),
            })),
        }]);

        for (idx, value) in class_literal.fields.value.iter().enumerate() {
            let mut computed_value = self.parse_expression(&value.1.value, false);
            instructions.append(&mut computed_value.instructions);

            let mut idx_ptr = self.get_index_pointer(
                result_var.clone(),
                IRValue::Literal(Literal::Integer(idx.try_into().unwrap())),
                Some(value.1.value._type.clone()),
            );

            instructions.append(&mut idx_ptr.instructions);

            instructions.push(Instruction::MemoryOperation(MemoryOperation::Store {
                value: IRValue::Variable(computed_value.result_var),
                pointer: idx_ptr.result_var,
            }));
        }

        ComputedExpression {
            instructions,
            result_var,
        }
    }

    pub(in crate::codegen) fn parse_field_access(
        &mut self,
        base: &TypedExpr,
        idx: usize,
    ) -> ComputedExpression {
        if let DataType::Custom(CustomDataType { fields, .. }) = &base._type {
            let mut instructions = vec![];

            let mut base = self.parse_expression(base, true);
            instructions.append(&mut base.instructions);

            let field_type = fields.value[idx]._type.value.clone();
            let idx_literal = IRValue::Literal(Literal::Integer(idx.try_into().unwrap()));
            let mut out = self.get_index_pointer(base.result_var, idx_literal, Some(field_type));

            instructions.append(&mut out.instructions);

            return ComputedExpression {
                instructions,
                result_var: out.result_var,
            };
        }

        unreachable!()
    }
}
