use std::{
    fs::{self, read_to_string, remove_dir_all, OpenOptions},
    io::Write,
    path::PathBuf,
    process::Command,
};

use chrono::Local;
use clap::Parser;
use lqdc_codegem::{
    codegem::{
        arch::x64::{X64Instruction, X64Selector},
        ir::ModuleBuilder,
        regalloc::RegAlloc,
    },
    codegen::CodegenPass,
    CodegemError,
};
use miette::*;

use lqdc_common::{
    codepass::PassRunner, make_signatures::MakeSignaturesPass, parsepass::ParsePass,
    type_check::TypeCheck,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let tmp_folder = PathBuf::from(format!(
        ".lqdc-tmp-{}",
        Local::now().format("%d-%m-%Y_%H%M%S")
    ));
    fs::create_dir(&tmp_folder)
        .into_diagnostic()
        .map_err(|e| e.wrap_err("Failed to create tmp folder"))?;

    let without_ext = cli.input.with_extension("");
    let name = without_ext.file_name().unwrap().to_str().unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(tmp_folder.join(format!("{name}.s")))
        .into_diagnostic()
        .map_err(|e| e.wrap_err("Failed to obtain handle to tmp file"))?;
    let input = read_to_string(&cli.input).into_diagnostic()?;
    let mut module_builder = ModuleBuilder::default().with_name(name);

    PassRunner::<(), ()>::new(&input)
        .run::<ParsePass>()?
        .run::<MakeSignaturesPass>()?
        .inject::<TypeCheck>()?
        .set_arg(&mut module_builder)
        .run::<CodegenPass>()?;

    let module = module_builder
        .build()
        .map_err(CodegemError::ModuleCreationError)?;

    let mut vcode = module.lower_to_vcode::<X64Instruction, X64Selector>();
    vcode.allocate_regs::<RegAlloc>();

    let mut buf = Vec::new();

    {
        vcode.emit_assembly(&mut buf).into_diagnostic()?;
    }

    // Codegem generates assembly with percentage signs, but clang does not support them
    let str_buf = String::from_utf8(buf).into_diagnostic()?;
    let str_buf = str_buf.replace("%", "");
    file.write_all(str_buf.as_bytes()).into_diagnostic()?;

    #[cfg(any(feature = "clang", feature = "gcc"))]
    {
        #[cfg(feature = "clang")]
        let mut command = Command::new("clang");
        #[cfg(feature = "gcc")]
        let mut command = Command::new("gcc");
        command
            .arg(tmp_folder.join(format!("{name}.s")))
            .args(["-o", &cli.output.to_str().unwrap()])
            .args(&cli.linker_args)
            .status()
            .unwrap();
    }

    remove_dir_all(tmp_folder)
        .into_diagnostic()
        .map_err(|e| e.wrap_err("Failed to delete tmp folder"))?;

    Ok(())
}

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    #[clap(short, long)]
    output: PathBuf,
    /// Only output object files
    #[clap(short = 'c')]
    gen_obj_files: bool,
    #[clap(short = 'L')]
    linker_args: Vec<String>,
}
