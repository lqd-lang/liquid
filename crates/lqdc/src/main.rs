use std::{fs, path::PathBuf};

use clap::Parser;

use codegem::ir::ModuleBuilder;
use frontend::{Expr, LowerToCodegem, Parse};

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.file).expect("File does not exist");

    let expr = Expr::parse(&input).unwrap().1;
    let mut builder = ModuleBuilder::default();
    expr.lower_to_code_gem(&mut builder)?;
    let module = builder.build();
    println!("{}", module);

    Ok(())
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long)]
    file: PathBuf,
}
