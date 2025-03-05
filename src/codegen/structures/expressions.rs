use itertools::Itertools;

use crate::{
    codegen::{
        codegen_main::{CodeGenerator, ComputedExpression},
        llvm_instructions::{
            BinaryOperation, IRBinaryOperator, IRValue, Instruction, MemoryOperation,
        },
    },
    lexer::{position::Spanned, tokens::Literal},
    parser::ast::{
        BinaryOperator, CallArg, DataType, DataTypeGetter, Expr, Function, Trait, TypedExpr,
        Variable,
    },
};

impl CodeGenerator {
    pub fn parse_expression(&mut self, expr: &TypedExpr, as_ref: bool) -> ComputedExpression {
        let mut result_var = self.next_tmp_var(&expr._type);
        let mut instructions = vec![];
        let mut is_ref = false;

        let mut out = match expr.expression.clone() {
            Expr::Literal(literal) => {
                is_ref = true;
                self.parse_literal(literal)
            } // ref
            Expr::Variable(variable) => {
                is_ref = true;
                self.parse_variable(variable)
            } // ref
            Expr::Binary { lhs, op, rhs } => {
                self.parse_binary_expr(&lhs.value, &op.value, &rhs.value)
            } // value
            Expr::Unary { op: _, expr: _ } => todo!(),
            Expr::Cast { value, to_type } => {
                is_ref = false;
                self.parse_type_cast(&value.value, &to_type.value)
            }
            Expr::Call { function, args } => {
                is_ref = false;
                self.parse_func_call(
                    &function.value,
                    &args.value.iter().map(|a| a.value.clone()).collect_vec(),
                )
            } // value
            Expr::Index { base, idx } => {
                is_ref = true;
                self.parse_indexing(&base.value, &idx.value)
            } // ref

            Expr::FieldAccess {
                base, field_idx, ..
            } => {
                is_ref = true;
                self.parse_field_access(&base.value, field_idx)
            }

            Expr::Deref(base) => {
                is_ref = true;
                self.parse_deref(&base.value)
            }

            Expr::Block { body } => {
                is_ref = false;

                /*
                   def main() -> int {
                       let bar = 10;
                       let foo = {
                           return bar + 1;
                       }
                       return foo;
                   }

                   wird zu:

                   def main() -> int {
                       let bar = 10;
                       let foo = block_1(bar);
                       return foo;
                   }

                   def block_1(bar: int) -> int {
                       return bar + 1;
                   }


                */

                let name = format!("block_{}", self.next_count());
                let block_func = Function::from_block(body.value, name);

                let virtual_args = block_func
                    .params
                    .value
                    .iter()
                    .map(|p| {
                        CallArg(TypedExpr {
                            expression: Expr::Variable(Variable {
                                name: p.value.name.clone(),
                                is_mutable: false,
                                _type: p.value._type.value.clone(),
                            }),
                            _type: p.value._type.value.clone(),
                            raw: None,
                        })
                    })
                    .collect_vec();

                self.program.functions.insert(
                    block_func.name.clone(),
                    Spanned {
                        value: block_func.clone(),
                        span: Default::default(),
                    },
                );

                self.parse_func_call(&block_func, &virtual_args)
            } // value

            Expr::Reference { value } => {
                is_ref = true;
                self.parse_reference(&value.value)
            }
            _ => unreachable!(),
        };

        instructions.append(&mut out.instructions);
        if as_ref && !is_ref {
            // sonderfall für index/feld zugriff auf funktionen
            // die basis wird immer als ref angefordert
            instructions.append(&mut vec![
                Instruction::VRegisterAssignment {
                    variable: result_var.clone(),
                    value: Box::new(Instruction::MemoryOperation(MemoryOperation::Alloca {
                        _type: expr._type.clone(),
                    })),
                },
                Instruction::MemoryOperation(MemoryOperation::Store {
                    value: IRValue::Variable(out.result_var),
                    pointer: result_var.clone(),
                }),
            ])
        } else if !as_ref && is_ref {
            instructions.append(&mut vec![Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(Instruction::MemoryOperation(MemoryOperation::Load {
                    pointer: out.result_var,
                })),
            }])
        } else {
            result_var = out.result_var;
        }

        ComputedExpression {
            instructions,
            result_var,
        }
    }

    fn parse_binary_expr(
        &mut self,
        lhs: &TypedExpr,
        op: &BinaryOperator,
        rhs: &TypedExpr,
    ) -> ComputedExpression {
        let mut instructions = vec![];
        let mut _type = lhs._type.clone();
        let lhs_type_info = self.program.get_type_info(&lhs._type);

        let trait_ = Trait::from_binary_operator(op);

        let mut lhs_computed = self.parse_expression(lhs, false);
        let mut rhs_computed = self.parse_expression(rhs, false);

        instructions.append(&mut lhs_computed.instructions);
        instructions.append(&mut rhs_computed.instructions);

        if let Some(trait_function) = self.program.get_trait_function(
            &lhs_type_info,
            &trait_,
            &[lhs._type.clone(), rhs._type.clone()],
        ) {
            let function_call = trait_function.to_call(&[lhs.clone(), rhs.clone()]);
            let mut function_instructions = self.parse_expression(&function_call, false);

            instructions.append(&mut function_instructions.instructions);

            return ComputedExpression {
                instructions,
                result_var: function_instructions.result_var,
            };
        }

        let numerical_is_float = {
            if _type.is_integer() {
                false
            } else if _type.is_float() {
                true
            } else {
                // TODO: vielleicht hier handling?
                false
            }
        };

        let operator = match op {
            // TODO: binop enum
            BinaryOperator::Add => {
                if numerical_is_float {
                    IRBinaryOperator::FAdd
                } else {
                    IRBinaryOperator::Add
                }
            }
            BinaryOperator::Subtract => {
                if numerical_is_float {
                    IRBinaryOperator::FSub
                } else {
                    IRBinaryOperator::Sub
                }
            }
            BinaryOperator::Multiply => {
                if numerical_is_float {
                    IRBinaryOperator::FMul
                } else {
                    IRBinaryOperator::Mul
                }
            }
            BinaryOperator::Divide => {
                if numerical_is_float {
                    IRBinaryOperator::FDiv
                } else {
                    IRBinaryOperator::Div
                }
            }
            BinaryOperator::Modulo => {
                if numerical_is_float {
                    IRBinaryOperator::FRem
                } else {
                    IRBinaryOperator::SRem
                }
            }

            BinaryOperator::Equal => IRBinaryOperator::Eq,
            BinaryOperator::NotEqual => IRBinaryOperator::Ne,
            BinaryOperator::GreaterThan => IRBinaryOperator::Sgt,
            BinaryOperator::GreaterThanOrEqual => IRBinaryOperator::Sge,
            BinaryOperator::LessThan => IRBinaryOperator::Slt,
            BinaryOperator::LessThanOrEqual => IRBinaryOperator::Sle,
            BinaryOperator::And => IRBinaryOperator::And,
            BinaryOperator::Or => IRBinaryOperator::Or,
        };

        if operator.is_ordering() {
            _type = DataType::Boolean
        }

        let result_var = self.next_tmp_var(&_type);

        // let mut instructions: Vec<Instruction> = lhs_computed
        //     .instructions
        //     .iter()
        //     .chain(&rhs_computed.instructions)
        //     .cloned()
        //     .collect();

        instructions.push(Instruction::VRegisterAssignment {
            variable: result_var.clone(),
            value: Box::new(Instruction::BinaryOperation(BinaryOperation {
                lhs: IRValue::Variable(lhs_computed.result_var),
                operator,
                rhs: IRValue::Variable(rhs_computed.result_var),
            })),
        });

        ComputedExpression {
            instructions,
            result_var,
        }
    }

    fn parse_literal(&mut self, literal: Literal) -> ComputedExpression {
        let mut result_var = self.next_tmp_var(&literal._type());
        let mut instructions = vec![];

        instructions.push(Instruction::VRegisterAssignment {
            variable: result_var.clone(),
            value: Box::new(Instruction::MemoryOperation(MemoryOperation::Alloca {
                _type: literal._type(),
            })),
        });

        match literal {
            Literal::ArrayLiteral(array_literal) => {
                let mut arr = self.parse_array_literal(&array_literal);
                instructions.append(&mut arr.instructions);
                result_var = arr.result_var
            }
            Literal::Custom(custom_literal) => {
                let mut class = self.parse_class_literal(&custom_literal);
                instructions.append(&mut class.instructions);
                result_var = class.result_var
            }
            literal => instructions.push(Instruction::MemoryOperation(MemoryOperation::Store {
                value: IRValue::Literal(literal),
                pointer: result_var.clone(),
            })),
        };

        ComputedExpression {
            instructions,
            result_var,
        }
    }

    fn parse_deref(&mut self, base: &TypedExpr) -> ComputedExpression {
        let mut instructions = vec![];
        let result_var = self.next_tmp_var(&base._type);
        let mut base = self.parse_expression(base, true);

        instructions.append(&mut base.instructions);

        instructions.push(Instruction::VRegisterAssignment {
            variable: result_var.clone(),
            value: Box::new(Instruction::MemoryOperation(MemoryOperation::Load {
                pointer: base.result_var,
            })),
        });

        ComputedExpression {
            instructions,
            result_var,
        }
    }
}
