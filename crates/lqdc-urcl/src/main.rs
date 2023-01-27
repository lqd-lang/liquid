use std::{
    fs::{read_to_string, OpenOptions},
    path::PathBuf,
};

use clap::Parser;
use miette::*;
use url_open::Url;

use lqdc_codegem::{
    codegem::{
        arch::urcl::{UrclInstruction, UrclSelector},
        ir::ModuleBuilder,
        regalloc::RegAlloc,
    },
    codegen::CodegenPass,
    CodegemError,
};
use lqdc_common::{
    codepass::PassRunner, make_signatures::MakeSignaturesPass, parsepass::ParsePass,
    type_check::TypeCheck,
};

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    #[clap(short, long)]
    output: PathBuf,
    #[clap(long)]
    stdlib: Url,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let without_ext = cli.input.with_extension("");
    let name = without_ext.file_name().unwrap().to_str().unwrap();
    let input = read_to_string(&cli.input).into_diagnostic()?;

    let mut builder = ModuleBuilder::default().with_name(name);

    PassRunner::<(), ()>::new(&input)
        .run::<ParsePass>()?
        .run::<MakeSignaturesPass>()?
        .inject::<TypeCheck>()?
        .set_arg(&mut builder)
        .run::<CodegenPass>()?;

    let module = builder.build().map_err(CodegemError::ModuleCreationError)?;

    let mut vcode = module.lower_to_vcode::<UrclInstruction, UrclSelector>();
    vcode.allocate_regs::<RegAlloc>();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cli.output)
        .into_diagnostic()?;

    vcode.emit_assembly(&mut file).into_diagnostic()?;

    Ok(())
}
