#[macro_use]
extern crate clap;

use std::{
    fs::{read_to_string, OpenOptions},
    path::PathBuf,
};

use clap::Parser;

use codegem::{
    arch::{
        rv64::{RvInstruction, RvSelector},
        urcl::{UrclInstruction, UrclSelector},
        x64::{X64Instruction, X64Selector},
    },
    ir::ModuleBuilder,
    regalloc::RegAlloc,
};

use lqdc_codegem::{
    codegen::CodegenPass, make_signatures::MakeSignaturesPass, parsepass::ParsePass,
};
use lqdc_common::codepass::PassRunner;
use miette::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let without_ext = cli.input.with_extension("");
    let name = without_ext.file_name().unwrap().to_str().unwrap();
    let input = read_to_string(&cli.input).into_diagnostic()?;
    // let mut compiler = Compiler::new(&input);
    let mut builder = ModuleBuilder::default().with_name(name);

    PassRunner::<()>::new(&input)
        .run::<ParsePass>()?
        // .inject::<TypeCheck>()?
        .run::<MakeSignaturesPass>()?
        .run_with_arg::<CodegenPass>(&mut builder)?;

    let module = builder.build();

    let mut out = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cli.output)
        .into_diagnostic()?;
    match cli.target {
        Target::RISCV64 => {
            let mut vcode = module.lower_to_vcode::<RvInstruction, RvSelector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly(&mut out).into_diagnostic()?;
        }
        Target::X64 => {
            let mut vcode = module.lower_to_vcode::<X64Instruction, X64Selector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly(&mut out).into_diagnostic()?;
        }
        Target::URCL => {
            let mut vcode = module.lower_to_vcode::<UrclInstruction, UrclSelector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly(&mut out).into_diagnostic()?;
        }
    }

    Ok(())
}

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    #[clap(short, long)]
    target: Target,
    #[clap(short, long)]
    output: PathBuf,
}

#[derive(ValueEnum, Clone)]
enum Target {
    RISCV64,
    X64,
    URCL,
}
