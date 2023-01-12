use std::{fs, path::PathBuf};

use clap::Parser;

use codegem::{
    arch::{
        rv64::{RvInstruction, RvSelector},
        urcl::{UrclInstruction, UrclSelector},
        x64::{X64Instruction, X64Selector},
    },
    ir::ModuleBuilder,
};
use lqdc::Compiler;
use miette::{bail, miette};

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.file).expect("File does not exist");

    let name = cli.file.with_extension("");
    let mut builder = ModuleBuilder::default().with_name(name.to_str().unwrap());
    let mut compiler = Compiler::new(&input);
    compiler.compile(&mut builder)?;

    let module = builder.build();

    match cli.target.as_str() {
        "rv64" | "riscv64" => {
            let vcode = module.lower_to_vcode::<RvInstruction, RvSelector>();
            fs::write(&cli.output, format!("{}", vcode)).unwrap();
        }
        "urcl" => {
            let vcode = module.lower_to_vcode::<UrclInstruction, UrclSelector>();
            fs::write(&cli.output, format!("{}", vcode)).unwrap();
        }
        "x64" => {
            let vcode = module.lower_to_vcode::<X64Instruction, X64Selector>();
            fs::write(&cli.output, format!("{}", vcode)).unwrap();
        }
        "codegem-ir" => {
            fs::write(&cli.output, format!("{}", module)).unwrap();
        }
        _ => bail!(miette!(
            "Unknown target, choose from ['riscv64', 'urcl', 'x64', 'codegem-ir']"
        )),
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct Cli {
    /// Input file
    file: PathBuf,
    /// What target you are compiling for
    ///
    /// Eg. ['risv64', 'urcl', 'x64', 'codegem-ir']
    #[clap(long, default_value = "x64")]
    target: String,
    /// Where to put the result?
    #[clap(short, long)]
    output: PathBuf,
}
