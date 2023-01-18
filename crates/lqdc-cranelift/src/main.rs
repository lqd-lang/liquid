use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use cranelift::prelude::{isa::Builder, *};
use cranelift_object::{ObjectBuilder, ObjectModule};
use lqdc_cranelift::Compiler;
use miette::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let without_ext = cli.input.with_extension("");
    let name = without_ext.file_name().unwrap().to_str().unwrap();
    let input = read_to_string(&cli.input).into_diagnostic()?;
    let builder = isa::lookup_by_name(&cli.target).into_diagnostic()?;
    let obj_builder = ObjectBuilder::new(
        builder
            .finish(settings::Flags))
            .into_diagnostic()?,
        name,
        cranelift_module::default_libcall_names(),
    );
    let mut compiler = Compiler::new(&input);

    compiler.compile()?;

    Ok(())
}

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    #[clap(short, long)]
    output: PathBuf,
    #[clap(short, long)]
    target: String,
}
