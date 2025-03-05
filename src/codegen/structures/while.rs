use crate::{
    codegen::{
        codegen_main::CodeGenerator,
        llvm_instructions::{IRValue, Instruction, JumpKind},
        utils::InstructionVecExt,
    },
    parser::ast::{Block, TypedExpr},
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_while(
        &mut self,
        condition: &TypedExpr,
        body: &Block,
    ) -> Vec<Instruction> {
        let mut instructions = vec![];
        let mut cond = self.parse_expression(condition, false);
        let count = self.next_count();

        let start_label = format!("while_head_{}", count);
        let body_label = format!("while_body_{}", count);
        let end_label = format!("end_while_{}", count);

        let start_block = Instruction::BlockDecl {
            label: start_label.clone(),
        };
        let body_block = Instruction::BlockDecl {
            label: body_label.clone(),
        };
        let end_block = Instruction::BlockDecl {
            label: end_label.clone(),
        };

        let start_jump = Instruction::Jump(JumpKind::Jump { label: start_label });

        let cond_jump = Instruction::Jump(JumpKind::ConditionJump {
            condition: IRValue::Variable(cond.result_var),
            true_label: body_label,
            false_label: end_label,
        });

        instructions.push(start_jump.clone());

        instructions.push(start_block);
        instructions.append(&mut cond.instructions);
        instructions.push(cond_jump);
        instructions.push(body_block);

        instructions.append(&mut self.parse_block(body));

        instructions.push(start_jump);
        instructions.push(end_block);

        instructions.move_allocations_to_top(); // um stackoverflows zu vermeiden

        instructions
    }
}
