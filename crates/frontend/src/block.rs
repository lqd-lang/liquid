use codegem::ir::{ModuleBuilder, Value};
use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    multi::separated_list1,
    sequence::{delimited, preceded},
    Parser,
};

use crate::{expr::Expr, LowerToCodegem, Parse};

#[derive(PartialEq, Debug)]
pub struct Block {
    exprs: Vec<Box<Expr>>,
}

impl Parse for Block {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let input = delimited(multispace0, tag("{"), multispace0)(input)?.0;
        let (input, exprs) = separated_list1(
            tag(";"),
            delimited(multispace0, Expr::parse.map(|x| Box::new(x)), multispace0),
        )(input)?;
        let input = preceded(multispace0, tag("}"))(input)?.0;

        Ok((input, Self { exprs }))
    }
}

impl LowerToCodegem for Block {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> miette::Result<Option<Value>> {
        let block = builder
            .push_block()
            .expect("Failed to create a block, not in a function");
        builder.switch_to_block(block);

        let mut result = Option::<Value>::None;
        for expr in &self.exprs {
            result = expr.lower_to_code_gem(builder)?;
        }

        Ok(result)
    }
}
