use crate::{
    codegen::{
        codegen_main::{CodeGenerator, ComputedExpression},
        llvm_instructions::{Cast, IRValue, Instruction, MemoryOperation},
    },
    lexer::position::Spanned,
    parser::ast::{DataType, Expr, TypedExpr},
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_type_cast(
        &mut self,
        value: &TypedExpr,
        to: &DataType,
    ) -> ComputedExpression {
        let mut instructions = vec![];
        let result_var = self.next_tmp_var(to);
        let mut computed_value = self.parse_expression(value, false);

        if matches!(value._type, DataType::Pointer(_)) && matches!(to, DataType::Pointer(_)) {
            // ptr -> ptr
            // wird zu ptr -> int -> ptr
            let ptr_to_int = TypedExpr {
                expression: Expr::Cast {
                    value: Box::new(Spanned {
                        value: value.clone(),
                        span: Default::default(),
                    }),
                    to_type: Spanned {
                        value: DataType::get_integer_type(),
                        span: Default::default(),
                    },
                },
                _type: DataType::get_integer_type(),
                raw: None,
            };

            return self.parse_type_cast(&ptr_to_int, to);
        }

        if matches!(value._type, DataType::Pointer(_)) && *to == DataType::get_integer_type() {
            instructions.append(&mut computed_value.instructions);

            instructions.push(Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(Instruction::MemoryOperation(MemoryOperation::PtrToInt {
                    pointer: IRValue::Variable(computed_value.result_var),
                })),
            });

            return ComputedExpression {
                instructions,
                result_var,
            };
        }

        if value._type == DataType::get_integer_type() && matches!(to, DataType::Pointer(_)) {
            instructions.append(&mut computed_value.instructions);

            instructions.push(Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(Instruction::MemoryOperation(MemoryOperation::IntToPtr {
                    value: IRValue::Variable(computed_value.result_var),
                    pointer: to.clone(),
                })),
            });

            return ComputedExpression {
                instructions,
                result_var,
            };
        }

        if (value._type.is_integer() || to.is_boolean()) && (to.is_integer() || to.is_boolean()) {
            let is_upcast = value._type.size() < to.size();
            instructions.append(&mut computed_value.instructions);

            if value._type == *to {
                return ComputedExpression {
                    instructions,
                    result_var: computed_value.result_var,
                };
            }

            let cast = if is_upcast {
                Cast::SignedIntUp(IRValue::Variable(computed_value.result_var), to.clone())
            } else {
                Cast::SignedIntDown(IRValue::Variable(computed_value.result_var), to.clone())
            };

            instructions.push(Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(Instruction::Cast(cast)),
            });

            return ComputedExpression {
                instructions,
                result_var,
            };
        }

        if value._type.is_integer() && to.is_float() {
            instructions.append(&mut computed_value.instructions);

            instructions.push(Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(Instruction::Cast(Cast::SignedIntToFloat(
                    IRValue::Variable(computed_value.result_var),
                    to.clone(),
                ))),
            });

            return ComputedExpression {
                instructions,
                result_var,
            };
        }

        if value._type.is_float() && to.is_integer() {
            instructions.append(&mut computed_value.instructions);

            instructions.push(Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(Instruction::Cast(Cast::FloatToSignedInt(
                    IRValue::Variable(computed_value.result_var),
                    to.clone(),
                ))),
            });

            return ComputedExpression {
                instructions,
                result_var,
            };
        }

        // if value._type.can_be_converted_to(&to) {
        //     return ComputedExpression {
        //         instructions: computed_value.instructions,
        //         result_var: computed_value.result_var,
        //     };
        // }

        println!("from: {:?}, to: {:?}", value._type, to);
        unreachable!()
    }
}
