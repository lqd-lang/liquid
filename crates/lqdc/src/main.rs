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

    let tmp_folder = if cli.temp_dir.is_none() {
        PathBuf::from(format!(
            ".lqdc-tmp-{}",
            Local::now().format("%d-%m-%Y_%H%M%S")
        ))
    } else {
        cli.temp_dir.unwrap()
    };
    if tmp_folder.exists() {
        fs::remove_dir_all(&tmp_folder)
            .into_diagnostic()
            .map_err(|e| e.wrap_err("Failed to delete temp folder"))?;
    }

    fs::create_dir(&tmp_folder)
        .into_diagnostic()
        .map_err(|e| e.wrap_err("Failed to create temp folder"))?;

    let mut outputs = vec![];
    for input in cli.input {
        let without_ext = input.with_extension("");
        let name = without_ext.file_name().unwrap().to_str().unwrap();
        outputs.push(tmp_folder.join(format!("{name}.s")));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(tmp_folder.join(format!("{name}.s")))
            .into_diagnostic()
            .map_err(|e| e.wrap_err("Failed to obtain handle to tmp file"))?;
        let input = read_to_string(&input).into_diagnostic()?;
        let mut module_builder = ModuleBuilder::default().with_name(name);

        let runner = PassRunner::<(), ()>::new(&input)
            .run::<ParsePass>()?
            .run::<MakeSignaturesPass>()?
            .inject::<TypeCheck>()?;

        if !cli.check {
            runner.set_arg(&mut module_builder).run::<CodegenPass>()?;

            let module = module_builder
                .build()
                .map_err(CodegemError::ModuleCreationError)?;

            if cli.emit_codegem {
                fs::write(
                    tmp_folder.join(format!("{name}.codegem")),
                    format!("{module}"),
                )
                .into_diagnostic()
                .map_err(|e| e.wrap_err("Failed to emit codegem IR"))?;
            }

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
                for output in &outputs {
                    command.arg(output.to_str().unwrap());
                }
                command
                    .args(["-o", &cli.output.to_str().unwrap()])
                    .args(&cli.linker_args)
                    .status()
                    .unwrap();
            }
        }
    }

    if !cli.keep_temp {
        remove_dir_all(tmp_folder)
            .into_diagnostic()
            .map_err(|e| e.wrap_err("Failed to delete tmp folder"))?;
    }

    Ok(())
}

#[derive(Parser)]
struct Cli {
    input: Vec<PathBuf>,
    #[clap(short, long)]
    output: PathBuf,
    /// Only output object files
    #[clap(short = 'c')]
    gen_obj_files: bool,
    #[clap(short = 'L')]
    linker_args: Vec<String>,
    #[clap(long)]
    check: bool,
    /// Don't delete the temp folder when completed
    #[clap(long)]
    keep_temp: bool,
    /// Set the temp directory manually
    #[clap(long)]
    temp_dir: Option<PathBuf>,
    /// Emit codegem IR into the temp directory.
    /// You probably want --keep-dir as well
    #[clap(long)]
    emit_codegem: bool,
}
