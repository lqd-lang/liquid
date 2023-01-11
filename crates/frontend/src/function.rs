use std::fmt::Debug;

use codegem::ir::{ModuleBuilder, Type, Value};
use miette::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    sequence::{delimited, tuple},
    IResult, Parser,
};

use crate::{block::Block, identifier::Identifier, GetType, LowerToCodegem, Parse, TYPES};

pub struct Function {
    pub id: Identifier,
    block: Block,
    return_type: Type,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.block == other.block
    }
}
impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("id", &self.id)
            .field("block", &self.block)
            // .field("return_type", &self.return_type)
            .finish()
    }
}
impl Parse for Function {
    fn parse(input: &str) -> IResult<&str, Self> {
        let input = tag("fn")(input)?.0;
        let input = multispace1(input)?.0;
        let (input, id) = Identifier::parse(input)?;

        let (input, return_type) = alt((
            tuple((
                delimited(multispace0, tag("->"), multispace0),
                Identifier::parse.map(|type_name| {
                    TYPES
                        .get(&type_name.0)
                        .expect(&format!("Type {} does not exist", type_name.0))
                }),
            ))
            .map(|(_, return_type)| return_type.clone()),
            tag("").map(|_| Type::Void),
        ))(input)?;

        let input = multispace0(input)?.0;
        let (input, block) = Block::parse(input)?;

        Ok((
            input,
            Self {
                id,
                block,
                return_type,
            },
        ))
    }
}
impl LowerToCodegem for Function {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>> {
        let func_id = builder.new_function(&self.id.0, &[], &self.return_type);
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
