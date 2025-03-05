use crate::{
    codegen::{
        codegen_main::{CodeGenerator, ComputedExpression},
        llvm_instructions::{IRValue, IRVariable, Instruction, MemoryOperation},
    },
    parser::ast::{TypedExpr, Variable},
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_variable_decl(
        &mut self,
        name: String,
        value: &TypedExpr,
    ) -> Vec<Instruction> {
        let variable = IRVariable {
            name: format!("%_{}_{}", name, self.next_count()),
            _type: value._type.clone(),
        };
        let value = self.parse_expression(value, false);
        let mut instructions = value.instructions;

        instructions.append(&mut vec![
            Instruction::VRegisterAssignment {
                variable: variable.clone(),
                value: Box::new(Instruction::MemoryOperation(MemoryOperation::Alloca {
                    _type: variable._type.clone(),
                })),
            },
            Instruction::MemoryOperation(MemoryOperation::Store {
                value: IRValue::Variable(value.result_var),
                pointer: variable.clone(),
            }),
        ]);

        self.variable_map.insert(name, variable);

        instructions
    }

    pub(in crate::codegen) fn parse_variable(&mut self, variable: Variable) -> ComputedExpression {
        // let result_var = self.next_tmp_var(&variable._type);
        let result_var = self.variable_map.get(&variable.name.value).unwrap().clone();

        // let instructions = vec![
        //     Instruction::VRegisterAssignment {
        //         variable: result_var.clone(),
        //         value: Box::new(Instruction::MemoryOperation(
        //             MemoryOperation::Load {
        //                 pointer: variable,
        //             }
        //         ))
        //     }
        // ];

        ComputedExpression {
            instructions: vec![],
            result_var,
        }
    }

    pub(in crate::codegen) fn parse_variable_mutation(
        &mut self,
        variable: &TypedExpr,
        new_value: &TypedExpr,
    ) -> Vec<Instruction> {
        let mut instructions = vec![];

        let mut var = self.parse_expression(variable, true); // zum überschreiben als referenz
        instructions.append(&mut var.instructions);

        let mut new_value = self.parse_expression(new_value, false);
        instructions.append(&mut new_value.instructions);

        instructions.push(Instruction::MemoryOperation(MemoryOperation::Store {
            value: IRValue::Variable(new_value.result_var),
            pointer: var.result_var,
        }));

        instructions
    }
}
