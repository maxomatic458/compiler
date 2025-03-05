#![cfg(test)]
use crate::{
    codegen::llvm_instructions::{FunctionCall, IRValue, IRVariable, MemoryOperation, ToIR},
    lexer::tokens::Literal,
    parser::ast::DataType,
};
use pretty_assertions::assert_eq;

#[test]
fn basic_instructions() {
    let instruction = MemoryOperation::Alloca {
        _type: DataType::Integer32,
    };
    let expected = "alloca i32".to_string();

    assert_eq!(instruction.to_ir(), expected);

    let variable = IRVariable {
        name: "foo".to_string(),
        _type: DataType::Integer32,
    };

    let instruction = MemoryOperation::Store {
        value: IRValue::Literal(Literal::Integer(5)),
        pointer: variable.clone(),
    };
    let expected = "store i64 5, i64* %_foo";
    assert_eq!(instruction.to_ir(), expected);

    let instruction = MemoryOperation::Load { pointer: variable };

    let expected = "load i32, i32* %_foo";
    assert_eq!(instruction.to_ir(), expected);

    let instruction = FunctionCall {
        name: "bar".to_string(),
        return_type: DataType::Boolean,
        args: vec![IRValue::Variable(IRVariable {
            name: "%_foo".to_string(),
            _type: DataType::Integer64,
        })],
    };

    let expected = "call i1 @bar(i64 %_foo)";

    assert_eq!(instruction.to_ir(), expected);
}

#[test]
fn literals() {
    assert_eq!("10".to_string(), Literal::Integer(10).to_ir());
    assert_eq!("1".to_string(), Literal::Boolean(true).to_ir());
    // assert_eq!(
    //     "0x3F93333300000000".to_string(),
    //     Literal::Float(1.15).to_ir()
    // );
}
