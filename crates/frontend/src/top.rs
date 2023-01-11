use codegem::ir::{ModuleBuilder, Value};
use miette::{bail, miette, Result};
use nom::{character::complete::multispace0, multi::many1, sequence::delimited};

use crate::{function::Function, LowerToCodegem, Parse};

#[derive(Debug)]
pub struct Top {
    functions: Vec<Function>,
}

impl Parse for Top {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, functions) =
            many1(delimited(multispace0, Function::parse, multispace0))(input)?;

        Ok((input, Self { functions }))
    }
}

impl LowerToCodegem for Top {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>> {
        if !self
            .functions
            .iter()
            .any(|function| function.id.0 == "main".to_string())
        {
            bail!(miette!("Missing a main function"))
        }

        for function in &self.functions {
            function.lower_to_code_gem(builder)?;
        }

        Ok(None)
    }
}
