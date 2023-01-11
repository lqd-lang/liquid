pub mod block;
pub mod expr;
mod function;
mod identifier;
mod literal;
mod var_assign;

use codegem::ir::{ModuleBuilder, Type, Value};

use miette::Result;
use nom::IResult;

pub trait Parse: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}

pub trait LowerToCodegem {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>>;
}

trait GetType {
    fn get_type(&self) -> Result<Type>;
}
