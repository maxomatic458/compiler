use itertools::Itertools;

use crate::{
    codegen::{
        codegen_main::{CodeGenerator, ComputedExpression},
        llvm_instructions::{FunctionCall, IRValue, IRVariable, Instruction, MemoryOperation},
        structs::{IRFunction, IRFunctionBody, IRFunctionParam},
        utils::InstructionVecExt,
    },
    lexer::tokens::Literal,
    parser::ast::{
        Block, CallArg, CommonGeneric, DataType, Function, FunctionParam, CLASS_SELF_ARG_NAME,
    },
};

impl CodeGenerator {
    pub fn parse_func_def(&mut self, function: Function) -> Vec<Instruction> {
        if function.is_builtin {
            return vec![];
        }

        if function.is_generic() {
            let mut instructions = vec![];

            for subtype in function.generic_subtypes.values().cloned() {
                instructions.append(&mut self.parse_func_def(subtype));
            }

            return instructions;
        }

        let param_to_var_instructions = match function.is_extern {
            true => vec![],
            false => self.parse_func_params(
                &function
                    .params
                    .value
                    .iter()
                    .map(|p| p.value.clone())
                    .collect_vec(),
            ),
        };

        let params: Vec<IRFunctionParam> = function
            .params
            .value
            .iter()
            .enumerate()
            .map(|(i, p)| IRFunctionParam {
                name: p.value.name.value.clone(),
                _type: if function.is_method() && !function.is_static_method() && i == 0 {
                    DataType::Pointer(Box::new(p.value._type.value.clone()))
                } else {
                    p.value._type.value.clone()
                },
            })
            .collect();

        let mut body = IRFunctionBody {
            instructions: {
                let mut instructions = Vec::new();
                instructions.extend(param_to_var_instructions);
                instructions.extend(self.parse_block(&function.body.value));
                instructions.kill_last_unused();
                instructions
            },
        };

        if function.return_type.value == DataType::None {
            body.instructions.push(Instruction::Return {
                expr: IRValue::Literal(Literal::Void),
            });
        }

        vec![Instruction::DeclareFunction(IRFunction {
            name: function.name,
            params,
            body,
            return_type: function.return_type.value,
            is_extern: function.is_extern,
        })]
    }

    fn parse_func_params(&mut self, params: &[FunctionParam]) -> Vec<Instruction> {
        // parameter müssen zu variablen umgewandelt werden
        let mut instructions = vec![];

        for (idx, param) in params.iter().enumerate() {
            let is_self_param = idx == 0 && param.name.value == *CLASS_SELF_ARG_NAME;

            let variable = IRVariable {
                name: match is_self_param {
                    true => format!("%_{}", param.name.value),
                    false => format!("%_{}_0", param.name.value),
                },
                _type: param._type.value.clone(),
            };

            let param_var = IRVariable {
                name: param.name.value.clone(),
                _type: match is_self_param {
                    true => DataType::Pointer(Box::new(param._type.value.clone())),
                    false => param._type.value.clone(),
                },
            };

            self.variable_map
                .insert(param.name.value.clone(), variable.clone());

            if !is_self_param {
                instructions.append(&mut vec![
                    Instruction::VRegisterAssignment {
                        variable: variable.clone(),
                        value: Box::new(Instruction::MemoryOperation(MemoryOperation::Alloca {
                            _type: variable._type.clone(),
                        })),
                    },
                    Instruction::MemoryOperation(MemoryOperation::Store {
                        value: IRValue::Variable(param_var),
                        pointer: variable.clone(),
                    }),
                ]);
            }
        }

        instructions
    }

    pub(in crate::codegen) fn parse_func_call(
        &mut self,
        function: &Function,
        args: &[CallArg],
    ) -> ComputedExpression {
        if function.is_builtin {
            return self.parse_builtin_func_call(function, args);
        }

        let result_var = self.next_tmp_var(&function.return_type.value);
        let mut instructions = vec![];
        let mut argument_values = vec![];

        for (idx, arg) in args.iter().enumerate() {
            let is_self_arg = idx == 0 && function.is_method() && !function.is_static_method();

            let mut computed = self.parse_expression(&arg.0, is_self_arg);

            if is_self_arg {
                // TODO ohne self arg, und to_class_specific
                computed.result_var._type = DataType::Pointer(Box::new(computed.result_var._type));
            }

            argument_values.push(IRValue::Variable(computed.result_var));
            instructions.append(&mut computed.instructions);
        }

        let call = Instruction::Call(FunctionCall {
            name: function.name.clone(),
            return_type: function.return_type.value.clone(),
            args: argument_values,
        });

        if DataType::None != function.return_type.value {
            instructions.push(Instruction::VRegisterAssignment {
                variable: result_var.clone(),
                value: Box::new(call),
            });
        } else {
            instructions.push(call)
        }

        ComputedExpression {
            instructions,
            result_var,
        }
    }

    fn parse_builtin_func_call(
        &mut self,
        function: &Function,
        args: &[CallArg],
    ) -> ComputedExpression {
        match function.display_name.value.as_str() {
            "size_of" => self.size_of(args[0].0.expression.clone()),
            _ => unreachable!(),
        }
    }

    pub(in crate::codegen) fn parse_block(&mut self, block: &Block) -> Vec<Instruction> {
        block
            .statements
            .iter()
            .flat_map(|s| self.parse_statement(s))
            .collect_vec()
    }
}
