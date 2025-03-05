use crate::{
    codegen::{
        codegen_main::{CodeGenerator, ComputedExpression},
        llvm_instructions::{IRValue, IRVariable, Instruction, MemoryOperation},
    },
    lexer::tokens::Literal,
    parser::ast::{ArrayLiteral, DataType, DataTypeGetter, Trait, TypedExpr},
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_array_literal(
        &mut self,
        array_literal: &ArrayLiteral,
    ) -> ComputedExpression {
        let result_var = self.next_tmp_var(&array_literal._type());

        let mut instructions = vec![];

        instructions.append(&mut vec![Instruction::VRegisterAssignment {
            variable: result_var.clone(),
            value: Box::new(Instruction::MemoryOperation(MemoryOperation::Alloca {
                _type: array_literal._type(),
            })),
        }]);

        // array elemente in array nach initialisierung speichern
        for (idx, value) in array_literal.values.value.iter().enumerate() {
            let mut computed_value = self.parse_expression(&value.value, false);
            instructions.append(&mut computed_value.instructions);

            let mut idx_ptr = self.get_index_pointer(
                result_var.clone(),
                IRValue::Literal(Literal::Integer(idx.try_into().unwrap())),
                None,
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
    /// Im Fall des index traits ist das wahrscheinlich ein dangling pointer
    /// aber das sollte für opt level 0 und 1 kein problem sein
    /// hoffentlich, weil dieser nur geholt wird, wenn er direkt überschrieben wird
    pub fn get_index_pointer(
        &mut self,
        array_like: IRVariable,
        idx: IRValue,
        field_type: Option<DataType>,
    ) -> ComputedExpression {
        let value_type = match array_like._type.clone() {
            DataType::Array { value_type, .. } => *value_type,
            DataType::Custom { .. } => field_type.unwrap(),

            unknown => panic!("cannot index type: {:?}", unknown),
        };

        let result_var = self.next_tmp_var(&value_type);

        let instructions = vec![Instruction::VRegisterAssignment {
            variable: result_var.clone(),
            value: Box::new(Instruction::MemoryOperation(
                MemoryOperation::GetElementPointer {
                    array: IRValue::Variable(array_like),
                    idx,
                },
            )),
        }];

        ComputedExpression {
            instructions,
            result_var,
        }
    }

    pub(in crate::codegen) fn parse_indexing(
        &mut self,
        base: &TypedExpr,
        idx: &TypedExpr,
    ) -> ComputedExpression {
        let mut instructions = vec![];

        let mut base_computed = self.parse_expression(base, true);
        let mut idx_computed = self.parse_expression(idx, false);

        instructions.append(&mut base_computed.instructions);
        instructions.append(&mut idx_computed.instructions);

        let base_type_info = self.program.get_type_info(&base._type);
        if let Some(trait_function) = self.program.get_trait_function(
            &base_type_info,
            &Trait::Index,
            &[base._type.clone(), idx._type.clone()],
        ) {
            let function_call = trait_function.to_call(&[base.clone(), idx.clone()]);
            let mut function_computed = self.parse_expression(&function_call, false);

            instructions.append(&mut function_computed.instructions);

            function_computed.result_var._type =
                if let DataType::Pointer(ref p) = function_computed.result_var._type {
                    *(p.to_owned())
                } else {
                    // Index trait muss einen Pointer zurückgeben siehe ast.rs
                    unreachable!()
                };

            return ComputedExpression {
                instructions,
                result_var: function_computed.result_var,
            };
        }

        let mut out = self.get_index_pointer(
            base_computed.result_var,
            IRValue::Variable(idx_computed.result_var),
            None,
        );

        instructions.append(&mut out.instructions);

        ComputedExpression {
            instructions,
            result_var: out.result_var,
        }
    }
}
