use clap::Parser;
use compiler::Compiler;
use semver::VersionReq;
use std::{
    io::{Read, Write},
    path::PathBuf,
};
use thiserror::Error;
use utils::{get_clang_version, ClangError};

pub mod codegen;
pub mod compiler;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod tests;
pub mod utils;

const LLVM_FILE_DEFAULT_NAME: &str = "out.ll";
const EXECUTABLE_FILE_DEFAULT_NAME: &str = "out.exe";
const CLANG_VERSION_REQ: &str = ">=16.0.0";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_name: Option<PathBuf>,
    #[arg(short, long, default_value = "false")]
    emit_llvm: bool,
    #[arg(short, long, default_value = "false")]
    dont_write_output: bool,
    #[arg(short, long)]
    output_path: Option<PathBuf>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ProgramError {
    #[error("clang error: {0}")]
    ClangError(ClangError),
    #[error("compiler error: {0}")]
    CompilerError(u32),
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install().unwrap();

    let args = Args::parse();

    let source_code = match args.file_name {
        Some(ref file_name) => std::fs::read_to_string(file_name)?,
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let start_time = std::time::Instant::now();

    let ir = match Compiler::compile(
        &source_code,
        args.file_name
            .as_deref()
            .map(|p| p.parent().unwrap().into()),
    ) {
        Ok(ir) => ir,
        Err(e) => {
            return Err(ProgramError::CompilerError(e.value.id()).into());
        }
    };

    if args.dont_write_output {
        return Ok(());
    }

    let output_path = match args.output_path {
        Some(output_path) => output_path,
        None => {
            if args.emit_llvm {
                PathBuf::from(LLVM_FILE_DEFAULT_NAME)
            } else {
                PathBuf::from(EXECUTABLE_FILE_DEFAULT_NAME)
            }
        }
    };

    if args.emit_llvm {
        std::fs::write(output_path, ir)?;
        return Ok(());
    }

    let clang_version = match get_clang_version() {
        Some(clang_version) => clang_version,
        None => return Err(ProgramError::ClangError(ClangError::NotInstalled).into()),
    };

    let clang_version_req = VersionReq::parse(CLANG_VERSION_REQ).unwrap();

    if !clang_version_req.matches(&clang_version) {
        println!("WARN: The clang version might not be supported!")
        // return Err(ProgramError::ClangError(ClangError::UnsupportedVersion(clang_version)).into());
    }

    let mut clang = std::process::Command::new("clang")
        .args(["-x", "ir", "-", "-o", output_path.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = clang.stdin.as_ref().unwrap();
    stdin.write_all(ir.as_bytes())?;

    let status = clang.wait()?;

    if !status.success() {
        return Err(ProgramError::ClangError(ClangError::ClangErrorCode(
            status.code().unwrap_or(-1),
        ))
        .into());
    }

    let elapsed_time = start_time.elapsed();
    println!("took: {:?}", elapsed_time);
    Ok(())
}
