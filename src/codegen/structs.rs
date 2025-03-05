use super::llvm_instructions::{Instruction, ToIR, IR};
use crate::parser::ast::DataType;

#[derive(Debug, Clone)]
pub struct IRFunctionBody {
    pub instructions: Vec<Instruction>,
}

impl ToIR for IRFunctionBody {
    fn to_ir(&self) -> IR {
        format!(
            " {{\n{}{}\n}}",
            Instruction::BlockDecl {
                label: "entry".to_string()
            }
            .to_ir(),
            self.instructions.to_ir()
        )
    }
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<IRFunctionParam>,
    pub body: IRFunctionBody,
    pub return_type: DataType,
    pub is_extern: bool,
}

impl ToIR for IRFunction {
    fn to_ir(&self) -> IR {
        match self.is_extern {
            true => format!(
                "declare {} @{}{}",
                self.return_type.to_ir(),
                self.name,
                self.params.to_ir(),
            ),
            false => format!(
                "define {} @{}{} {}",
                self.return_type.to_ir(),
                self.name,
                self.params.to_ir(),
                self.body.to_ir(),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IRFunctionParam {
    pub name: String,
    pub _type: DataType,
}

// impl ToIR for IRFunctionParam {
//     fn to_ir(self) -> IR {
//         format!("{} %_{}", self._type.to_ir(), self.name)
//     }
// }

impl ToIR for Vec<IRFunctionParam> {
    fn to_ir(&self) -> IR {
        format!(
            "({})",
            self.iter()
                .map(|p| format!("{} %_{}", p._type.clone().to_ir(), p.name))
                .collect::<Vec<IR>>()
                .join(",")
        )
    }
}
