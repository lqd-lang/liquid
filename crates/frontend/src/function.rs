use codegem::ir::{ModuleBuilder, Type, Value};
use miette::Result;
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    IResult,
};

use crate::{block::Block, identifier::Identifier, GetType, LowerToCodegem, Parse};

#[derive(PartialEq, Debug)]
pub struct Function {
    id: Identifier,
    block: Block,
}

impl Parse for Function {
    fn parse(input: &str) -> IResult<&str, Self> {
        let input = tag("fn")(input)?.0;
        let input = multispace1(input)?.0;
        let (input, id) = Identifier::parse(input)?;
        // let input = tag(":")(input)?.0;
        let input = multispace0(input)?.0;
        let (input, block) = Block::parse(input)?;

        Ok((input, Self { id, block }))
    }
}

impl LowerToCodegem for Function {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>> {
        let func_id = builder.new_function(&self.id.0, &[], &Type::Void);
        builder.switch_to_function(func_id);
        let entry_block = builder.push_block().expect("Failed to create entry block");
        builder.switch_to_block(entry_block);
        self.block.lower_to_code_gem(builder)?;

        Ok(None)
    }
}

impl GetType for Function {
    fn get_type(&self) -> Result<Type> {
        todo!()
    }
}
