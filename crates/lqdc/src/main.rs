use std::{fs, path::PathBuf};

use clap::Parser;

use codegem::{
    arch::{rv64::RvSelector, urcl::UrclSelector, x64::X64Selector},
    ir::ModuleBuilder,
    regalloc::RegAlloc,
};

use lqdc::Compiler;
use miette::{bail, miette, Diagnostic};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
enum Error {
    #[error("Require an output file")]
    #[help("Try using the '-o' flag")]
    RequireOutFile,
    #[error("Unused option '{}'", .0)]
    #[diagnostic(severity(warning))]
    UnusedOption(String, #[help] Option<String>),
}

#[derive(Debug, Diagnostic, Error)]
// #[error(transparent)]
#[error("")]
struct ManyError {
    #[related]
    errors: Vec<Error>,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.file).expect("File does not exist");

    let mut errors = vec![];

    let name = cli.file.with_extension("");
    let mut builder = ModuleBuilder::default().with_name(name.to_str().unwrap());
    let mut compiler = Compiler::new(&input);
    compiler.compile(&mut builder)?;

    let module = builder.build();

    match cli.target.as_str() {
        "rv64" | "riscv64" => {
            if cli.output.is_some() {
                errors.push(Error::UnusedOption("-o,--output".to_string(), None))
            }

            let mut vcode = module.lower_to_vcode::<_, RvSelector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly();
        }
        "urcl" => {
            if cli.output.is_some() {
                errors.push(Error::UnusedOption("-o,--output".to_string(), None))
            }

            let mut vcode = module.lower_to_vcode::<_, UrclSelector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly();
        }
        "x64" => {
            if cli.output.is_some() {
                errors.push(Error::UnusedOption("-o,--output".to_string(), None))
            }

            let mut vcode = module.lower_to_vcode::<_, X64Selector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly();
        }
        "codegem-ir" => {
            let output = cli.output.ok_or_else(|| Error::RequireOutFile)?;
            match output.to_str().unwrap() {
                "<stdio>" => println!("{}", module),
                _ => fs::write(&output, format!("{}", module)).unwrap(),
            }
        }
        _ => bail!(miette!(
            "Unknown target, choose from ['riscv64', 'urcl', 'x64', 'codegem-ir']"
        )),
    }

    match errors.len() {
        0 => Ok(()),
        _ => bail!(ManyError { errors }),
    }
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
    output: Option<PathBuf>,
}
