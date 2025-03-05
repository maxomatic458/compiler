// use std::fmt::Display;

use itertools::Itertools;

use crate::{
    lexer::tokens::Literal,
    parser::ast::{CustomDataType, DataType, DataTypeGetter},
};

use super::structs::{IRFunction, IRFunctionParam};

pub type IR = String;

#[derive(Debug, Clone)]
pub enum Instruction {
    BinaryOperation(BinaryOperation),
    // BitwiseBinaryOperation(BitwiseBinaryOperation),
    BlockDecl {
        label: String,
    },
    MemoryOperation(MemoryOperation),
    Call(FunctionCall),
    // Comparison(Comparison),
    VRegisterAssignment {
        variable: IRVariable,
        value: Box<Instruction>,
    },
    Return {
        expr: IRValue,
    },
    Jump(JumpKind),
    DeclareType(DataType),
    DeclareFunction(IRFunction),
    Cast(Cast),

    Comment(String),
    NOOP,
}

pub trait ToIR {
    fn to_ir(&self) -> IR;
}

impl ToIR for Instruction {
    fn to_ir(&self) -> IR {
        match self {
            Instruction::BinaryOperation(binary_operation) => binary_operation.to_ir(),
            Instruction::MemoryOperation(memory_operation) => memory_operation.to_ir(),
            Instruction::Call(function_call) => function_call.to_ir(),
            Instruction::VRegisterAssignment {
                variable: name,
                value,
            } => format!("{} = {}", name.name, (*value).to_ir()), //TODO: nur wenn instruction wert returned
            Instruction::Return { expr } => {
                if expr._type() == DataType::None {
                    return "ret void".to_string();
                }
                format!("ret {} {}", expr._type().to_ir(), expr.to_ir())
            }
            Instruction::BlockDecl { label } => format!("{label}:\n"),
            Instruction::Jump(jump_kind) => jump_kind.to_ir(),
            Instruction::DeclareType(_type) => {
                if let DataType::Custom(CustomDataType { fields, .. }) = _type.clone() {
                    return format!(
                        "{} = type {{\n{}}}",
                        _type.to_ir(),
                        fields
                            .value
                            .into_iter()
                            .map(|f| format!("{} ;{}\n", f._type.value.to_ir(), f.name.value))
                            .join(",")
                    );
                }
                unreachable!()
            }
            Instruction::DeclareFunction(inner) => inner.to_ir(),
            Instruction::Comment(comment) => format!("; {}", comment.replace('\n', "\n; ")),
            Instruction::Cast(cast) => cast.to_ir(),
            Instruction::NOOP => "add i1 0, 0".to_string(),
        }
    }
}

impl<T: ToIR> ToIR for Vec<T> {
    fn to_ir(&self) -> IR {
        self.iter()
            .map(|i| i.to_ir())
            .collect::<Vec<IR>>()
            .join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub return_type: DataType,
    pub args: Vec<IRValue>,
}

impl ToIR for FunctionCall {
    fn to_ir(&self) -> IR {
        format!(
            "call {} @{}({})",
            self.return_type.to_ir(),
            self.name,
            self.args
                .iter()
                .map(|a| format!("{} {}", a._type().to_ir(), a.clone().to_ir()))
                .join(","),
        )
    }
}

#[derive(Debug, Clone)]
pub enum IRValue {
    Literal(Literal),
    Variable(IRVariable),
}

impl ToIR for IRValue {
    fn to_ir(&self) -> IR {
        match self {
            IRValue::Literal(literal) => literal.to_ir(),
            IRValue::Variable(variable) => variable.to_ir(),
        }
    }
}

impl DataTypeGetter for IRValue {
    fn _type(&self) -> DataType {
        match self {
            IRValue::Literal(literal) => literal._type(),
            IRValue::Variable(variable) => variable._type(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum JumpKind {
    Jump {
        label: String,
    },
    ConditionJump {
        condition: IRValue,
        true_label: String,
        false_label: String,
    },
}

impl ToIR for JumpKind {
    fn to_ir(&self) -> IR {
        match self {
            JumpKind::Jump { label } => format!("br label %{label}"),
            JumpKind::ConditionJump {
                condition,
                true_label,
                false_label,
            } => format!(
                "br {} {}, label %{}, label %{} ",
                condition._type().to_ir(),
                condition.to_ir(),
                true_label,
                false_label,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IRVariable {
    pub name: String,
    pub _type: DataType,
}

impl From<IRFunctionParam> for IRVariable {
    fn from(value: IRFunctionParam) -> Self {
        IRVariable {
            name: value.name,
            _type: value._type,
        }
    }
}

impl ToIR for IRVariable {
    fn to_ir(&self) -> IR {
        let mut out = self.name.to_owned();
        if !out.starts_with("%_") {
            out = format!("%_{}", out)
        }
        out
    }
}

impl DataTypeGetter for IRVariable {
    fn _type(&self) -> DataType {
        self._type.clone()
    }
}

#[derive(Debug, Clone)]
pub struct BinaryOperation {
    pub lhs: IRValue,
    pub operator: IRBinaryOperator,
    pub rhs: IRValue,
}

impl ToIR for BinaryOperation {
    fn to_ir(&self) -> IR {
        if self.operator.is_ordering() {
            let comp = if self.lhs._type().is_integer() || self.lhs._type().is_boolean() {
                "icmp"
            } else if self.lhs._type().is_float() {
                "fcmp"
            } else {
                unreachable!()
            };

            return format!(
                "{} {} {} {}, {}",
                comp,
                self.operator.to_ir(),
                self.lhs._type().to_ir(),
                self.lhs.to_ir(),
                self.rhs.to_ir(),
            );
        }

        format!(
            "{} {} {}, {}",
            self.operator.to_ir(),
            self.lhs._type().to_ir(),
            self.lhs.to_ir(),
            self.rhs.to_ir()
        )
    }
}

#[derive(Debug, Clone)]
pub enum IRBinaryOperator {
    // int
    Add,
    Sub,
    Mul,
    Div,
    SRem, // modulo

    // float
    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem, // modulo

    // int
    And,
    Or,
    Xor,

    // float & int
    Eq,
    Ne,

    Sgt,
    Sge,

    Slt,
    Sle,
}

impl IRBinaryOperator {
    pub fn is_ordering(&self) -> bool {
        matches!(
            self,
            IRBinaryOperator::Eq
                | IRBinaryOperator::Ne
                | IRBinaryOperator::Sgt
                | IRBinaryOperator::Sge
                | IRBinaryOperator::Slt
                | IRBinaryOperator::Sle
        )
    }
}

impl ToIR for IRBinaryOperator {
    fn to_ir(&self) -> IR {
        match self {
            IRBinaryOperator::Add => "add",
            IRBinaryOperator::Sub => "sub",
            IRBinaryOperator::Mul => "mul",
            IRBinaryOperator::Div => "sdiv",
            IRBinaryOperator::FAdd => "fadd",
            IRBinaryOperator::FSub => "fsub",
            IRBinaryOperator::FMul => "fmul",
            IRBinaryOperator::FDiv => "fdiv",
            IRBinaryOperator::And => "and",
            IRBinaryOperator::Or => "or",
            IRBinaryOperator::Xor => "xor",
            IRBinaryOperator::Eq => "eq",
            IRBinaryOperator::Ne => "ne",
            IRBinaryOperator::Sgt => "sgt",
            IRBinaryOperator::Sge => "sge",
            IRBinaryOperator::Slt => "slt",
            IRBinaryOperator::Sle => "sle",
            IRBinaryOperator::SRem => "srem",
            IRBinaryOperator::FRem => "frem",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum MemoryOperation {
    Alloca { _type: DataType },
    Load { pointer: IRVariable },
    Store { value: IRValue, pointer: IRVariable },
    GetElementPointer { array: IRValue, idx: IRValue },
    GetSizeOf { _type: DataType },
    PtrToInt { pointer: IRValue },
    IntToPtr { value: IRValue, pointer: DataType },
}

impl ToIR for MemoryOperation {
    fn to_ir(&self) -> IR {
        match self {
            MemoryOperation::Alloca { _type } => format!("alloca {}", _type.to_ir()),

            MemoryOperation::Load { pointer } => format!(
                "load {pointer_type}, {pointer_type}* {pointer_value}",
                pointer_type = pointer._type().to_ir(),
                pointer_value = pointer.to_ir(),
            ),

            MemoryOperation::Store { value, pointer } => format!(
                "store {value_type} {value}, {value_type}* {pointer}",
                value_type = value._type().to_ir(),
                value = value.to_ir(),
                pointer = pointer.to_ir(),
            ),

            MemoryOperation::GetElementPointer { array, idx } => {
                let is_array = matches!(array._type(), DataType::Array { .. });

                let mut idx_type = DataType::get_integer_type();
                if !is_array {
                    idx_type = DataType::Integer32;
                }

                format!("getelementptr {array_type}, {array_type}* {array}, {idx_type} 0, {idx_type} {idx}",
                    array_type = array._type().to_ir(),
                    array = array.to_ir(),
                    idx_type = idx_type.to_ir(),
                    idx = idx.to_ir(),
                )
            }
            MemoryOperation::GetSizeOf { _type } => format!(
                "getelementptr {array_type}, {array_type}* null, {idx_type} 1",
                array_type = _type.to_ir(),
                idx_type = DataType::get_integer_type().to_ir(),
            ),
            MemoryOperation::PtrToInt { pointer } => format!(
                "ptrtoint {pointer_type} {pointer} to {int_type}",
                pointer_type = pointer._type().to_ir(),
                pointer = pointer.to_ir(),
                int_type = DataType::get_integer_type().to_ir(),
            ),
            MemoryOperation::IntToPtr { value, pointer } => format!(
                "inttoptr {int_type} {int_value} to {pointer_type}",
                int_type = value._type().to_ir(),
                int_value = value.to_ir(),
                pointer_type = pointer._type().to_ir(),
            ),
        }
    }
}

impl ToIR for DataType {
    fn to_ir(&self) -> IR {
        match self {
            DataType::Integer8 => "i8".to_string(),
            DataType::Integer16 => "i16".to_string(),
            DataType::Integer32 => "i32".to_string(),
            DataType::Integer64 => "i64".to_string(),
            DataType::Float => "float".to_string(),
            DataType::Boolean => "i1".to_string(),
            DataType::Pointer(data_type) => format!("{}*", (*data_type).to_ir()),
            DataType::Custom(CustomDataType { name, .. }) => format!("%{name}"),
            DataType::None => "void".to_string(),
            DataType::Array {
                value_type: _type,
                len,
            } => format!("[{} x {}]", len, _type.to_ir()),
            DataType::Generic(inner) => {
                unreachable!("{inner}")
            }
            DataType::DataType => unreachable!(),
        }
    }
}

impl DataTypeGetter for DataType {
    fn _type(&self) -> DataType {
        self.clone()
    }
}

impl ToIR for Literal {
    fn to_ir(&self) -> IR {
        match self {
            Literal::Void => "void".to_string(),
            Literal::Integer(integer) => integer.to_string(),
            Literal::Float(float) => float_to_llvm(*float as f32),
            Literal::Boolean(bool) => (*bool as u8).to_string(),
            Literal::ArrayLiteral(_array) => panic!(),
            Literal::Custom(_) => panic!(),
            Literal::DataType { .. } => unreachable!(),
        }
    }
}

// https://wiki.alcidesfonseca.com/blog/generate-float-and-double-literals-llvm-java/
fn float_to_llvm(f: f32) -> String {
    let d = f as f64;
    let bits: u64 = d.to_bits();
    format!("0x{}", to_hex_string(bits))
}

fn to_hex_string(mut l: u64) -> String {
    let mut count = if l == 0 {
        1
    } else {
        ((64 - l.leading_zeros() + 3) / 4) as usize
    };
    let mut buffer = String::with_capacity(count);

    while count > 0 {
        let t = l & 0xF;
        let c = if t > 9 {
            (t - 10 + b'A' as u64) as u8
        } else {
            (t + b'0' as u64) as u8
        };
        buffer.insert(0, c as char);
        l >>= 4;
        count -= 1;
    }

    buffer
}

#[derive(Debug, Clone)]
pub enum Cast {
    SignedIntUp(IRValue, DataType),
    SignedIntDown(IRValue, DataType),
    SignedIntToFloat(IRValue, DataType),
    FloatToSignedInt(IRValue, DataType),
}

impl ToIR for Cast {
    fn to_ir(&self) -> IR {
        match self {
            Cast::SignedIntUp(value, to) => format!(
                "sext {} {} to {}",
                value._type().to_ir(),
                value.to_ir(),
                to.to_ir()
            ),
            Cast::SignedIntDown(value, to) => format!(
                "trunc {} {} to {}",
                value._type().to_ir(),
                value.to_ir(),
                to.to_ir()
            ),
            Cast::SignedIntToFloat(value, to) => format!(
                "sitofp {} {} to {}",
                value._type().to_ir(),
                value.to_ir(),
                to.to_ir()
            ),
            Cast::FloatToSignedInt(value, to) => format!(
                "call {} @llvm.fptosi.sat.{}.{}({} {})",
                to.to_ir(),
                to.to_ir(),
                value._type().to_ir(),
                value._type().to_ir(),
                value.to_ir()
            ),
        }
    }
}
