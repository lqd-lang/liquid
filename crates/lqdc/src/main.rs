use std::path::PathBuf;

use clap::Parser;

#[cfg(feature = "backend_codegem")]
use lqdc::codegem::Compiler;
use miette::*;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
enum Error {
    // #[error("Require an output file")]
    // #[help("Try using the '-o' flag")]
    // RequireOutFile,
    // #[error("Unused option '{}'", .0)]
    // #[diagnostic(severity(warning))]
    // UnusedOption(String, #[help] Option<String>),
}

#[derive(Debug, Diagnostic, Error)]
// #[error(transparent)]
#[error("")]
struct ManyError {
    #[related]
    errors: Vec<Error>,
}

#[cfg(feature = "backend_codegem")]
fn run_codegem(cli: Cli) -> miette::Result<()> {
    use codegem::{
        arch::{rv64::RvSelector, urcl::UrclSelector, x64::X64Selector},
        ir::ModuleBuilder,
        regalloc::RegAlloc,
    };
    use std::{
        fs::{self, OpenOptions},
        io::Write,
    };

    let input = fs::read_to_string(&cli.file).expect("File does not exist");

    let errors = vec![];

    let name = cli.file.with_extension("");
    let mut builder = ModuleBuilder::default().with_name(name.to_str().unwrap());
    let mut compiler = Compiler::new(&input);
    compiler.compile(&mut builder)?;

    let module = builder.build();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&cli.output)
        .map_err(|e| miette!("Failed to open output file\n{}", e))?;
    match cli.target.as_str() {
        "rv64" | "riscv64" => {
            let mut vcode = module.lower_to_vcode::<_, RvSelector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly(&mut file).into_diagnostic()
        }
        "urcl" => {
            let mut vcode = module.lower_to_vcode::<_, UrclSelector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly(&mut file).into_diagnostic()
        }
        "x64" => {
            let mut vcode = module.lower_to_vcode::<_, X64Selector>();
            vcode.allocate_regs::<RegAlloc>();
            vcode.emit_assembly(&mut file).into_diagnostic()
        }
        "codegem-ir" => file
            .write_all(format!("{}", module).as_bytes())
            .into_diagnostic(),
        _ => bail!(miette!(
            "Unknown target, choose from ['riscv64', 'urcl', 'x64', 'codegem-ir']"
        )),
    }?;

    match errors.len() {
        0 => Ok(()),
        _ => bail!(ManyError { errors }),
    }
}

#[cfg(feature = "backend_cranelift")]
fn run_cranelift(cli: Cli) -> Result<()> {
    use std::fs;

    let input = fs::read_to_string(&cli.file).unwrap();
    let module_name = cli.file.with_extension("");
    let mut compiler =
        lqdc::cranelift::Compiler::new(&input, module_name.file_name().unwrap().to_str().unwrap());

    compiler.use_object_module(&cli.target)?;
    compiler.compile()?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.backend.as_str() {
        #[cfg(feature = "backend_codegem")]
        "codegem" => run_codegem(cli),
        #[cfg(feature = "backend_cranelift")]
        "cranelift" => run_cranelift(cli),
        _ => panic!("Invalid backend"),
    }
}

#[derive(Debug, Parser)]
struct Cli {
    /// Input file
    file: PathBuf,
    /// What target you are compiling for
    ///
    /// Eg. ['riscv64', 'urcl', 'x64', 'codegem-ir']
    #[clap(long, default_value = "x64")]
    target: String,
    /// Where to put the result?
    #[clap(short, long)]
    output: PathBuf,
    #[clap(short, long, default_value = "codegem")]
    backend: String,
}
