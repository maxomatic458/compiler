use crate::{
    codegen::{
        codegen_main::CodeGenerator,
        llvm_instructions::{IRValue, Instruction, JumpKind},
    },
    lexer::position::Spanned,
    parser::{
        ast::{Block, ElseIfBranch, TypedExpr},
        structures::r#if::{validate_if_return, BranchReturn},
    },
};

impl CodeGenerator {
    pub(in crate::codegen) fn parse_if(
        &mut self,
        condition: &TypedExpr,
        true_branch: &Block,
        else_if_branches: &[Spanned<ElseIfBranch>],
        false_branch: Option<&Block>,
    ) -> Vec<Instruction> {
        let mut instructions = vec![];
        let mut cond = self.parse_expression(condition, false);
        let count = self.next_count();

        instructions.append(&mut cond.instructions);

        let branch_return_type = validate_if_return(
            &Spanned {
                value: true_branch.clone(),
                ..Default::default()
            },
            else_if_branches,
            &false_branch.map(|x| Spanned {
                value: x.clone(),
                ..Default::default()
            }),
        )
        .unwrap()
        .1;

        let true_label = format!("if_{}", count);
        let false_label = format!("else_{}", count);
        let end_label = format!("end_if{}", count);

        let true_block = Instruction::BlockDecl {
            label: true_label.clone(),
        };

        let _false_block = Instruction::BlockDecl {
            label: false_label.clone(),
        };

        let end_block = Instruction::BlockDecl {
            label: end_label.clone(),
        };

        let cond_jump = Instruction::Jump(JumpKind::ConditionJump {
            condition: IRValue::Variable(cond.result_var),
            true_label: true_label.clone(),
            false_label: if false_branch.is_some() || !else_if_branches.is_empty() {
                false_label.clone()
            } else {
                end_label.clone()
            },
        });

        let end_jump = Instruction::Jump(JumpKind::Jump {
            label: end_label.clone(),
        });

        let requires_end = !matches!(branch_return_type, BranchReturn::AllReturn)
            || matches!(branch_return_type, BranchReturn::NoneReturn);

        instructions.push(cond_jump);
        instructions.push(true_block);

        instructions.append(&mut self.parse_block(true_branch));
        if requires_end {
            instructions.push(end_jump.clone());
        }

        let mut else_if_label = false_label.clone();
        for (idx, elif) in else_if_branches.iter().enumerate() {
            let mut cond = self.parse_expression(&elif.value.condition.value, false);

            let else_if_label_local = else_if_label.clone();
            let block = Instruction::BlockDecl {
                label: else_if_label_local.clone(),
            };

            instructions.push(block);
            instructions.append(&mut cond.instructions);

            else_if_label = format!("else_{}_{}", count, idx);

            let true_label = format!("if_{}_{}", count, idx);
            let false_label = if idx == else_if_branches.len() - 1 && false_branch.is_none() {
                end_label.clone()
            } else {
                else_if_label.clone()
            };

            let cond_jump = Instruction::Jump(JumpKind::ConditionJump {
                condition: IRValue::Variable(cond.result_var),
                true_label: true_label.clone(),
                false_label: false_label.clone(),
            });

            instructions.push(cond_jump);

            let block = Instruction::BlockDecl {
                label: true_label.clone(),
            };

            instructions.push(block);
            instructions.append(&mut self.parse_block(&elif.value.body.value));

            if requires_end {
                instructions.push(end_jump.clone());
            }
        }

        if let Some(false_branch) = false_branch {
            let block = Instruction::BlockDecl {
                label: else_if_label.clone(),
            };

            instructions.push(block);
            instructions.append(&mut self.parse_block(false_branch));

            if requires_end {
                instructions.push(end_jump.clone());
            }
        }

        if requires_end {
            instructions.push(end_block);
        }

        instructions
    }
}
