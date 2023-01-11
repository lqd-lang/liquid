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
use frontend::{top::Top, Context, LowerToCodegem, Parse};
use miette::{bail, miette};

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.file).expect("File does not exist");

    let top = Top::parse(&input).unwrap().1;
    let mut builder = ModuleBuilder::default().with_name(
        cli.file
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
    );
    let mut context = Context::default();
    top.lower_to_code_gem(&mut builder, &mut context)?;
    // println!("{:#?}", top);
    let module = builder.build();
    println!("{}", module);

    if !PathBuf::from("result").exists() {
        fs::create_dir_all("result").unwrap();
    }

    match cli.target.as_str() {
        "rv64" => {
            let vcode = module.lower_to_vcode::<RvInstruction, RvSelector>();
            fs::write("result/output.asm", format!("{}", vcode)).unwrap();
        }
        "urcl" => {
            let vcode = module.lower_to_vcode::<UrclInstruction, UrclSelector>();
            fs::write("result/output.urcl", format!("{}", vcode)).unwrap();
        }
        "x64" => {
            let vcode = module.lower_to_vcode::<X64Instruction, X64Selector>();
            fs::write("result/output.asm", format!("{}", vcode)).unwrap();
        }
        _ => bail!(miette!(
            "Unknown target, choose from ['rv64', 'urcl', 'x64']"
        )),
    }

    Ok(())
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long)]
    file: PathBuf,
    #[clap(long, default_value = "rv64")]
    target: String,
}
