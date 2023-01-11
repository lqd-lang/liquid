use codegem::ir::{ModuleBuilder, Type, Value};
use miette::Result;
use nom::{branch::alt, IResult, Parser};

use crate::{
    function::Function, literal::Literal, var_assign::VarAssign, GetType, LowerToCodegem, Parse,
};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    VarAssign(VarAssign),
    Function(Function),
    // Block(Block),
}

impl Parse for Expr {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            // Block::parse.map(Self::Block),
            Literal::parse.map(Self::Literal),
            VarAssign::parse.map(Self::VarAssign),
            Function::parse.map(Self::Function),
        ))(input)
    }
}

impl LowerToCodegem for Expr {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>> {
        match self {
            Expr::Literal(literal) => literal.lower_to_code_gem(builder),
            Expr::VarAssign(var_assign) => var_assign.lower_to_code_gem(builder),
            Expr::Function(function) => function.lower_to_code_gem(builder),
            // Expr::Block(block) => todo!(),
        }
    }
}

impl GetType for Expr {
    fn get_type(&self) -> Result<Type> {
        Ok(match self {
            Expr::Literal(l) => l.get_type()?,
            Expr::VarAssign(v) => v.get_type()?,
            Expr::Function(f) => f.get_type()?,
            // Expr::Block(b) => todo!(),
        })
    }
}
