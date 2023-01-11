#[macro_use]
extern crate lazy_static;

mod bin_op;
pub mod block;
pub mod expr;
mod function;
mod identifier;
mod literal;
pub mod top;
mod var_assign;

use std::collections::HashMap;

use codegem::ir::{ModuleBuilder, Type, Value};

use miette::Result;
use nom::IResult;

lazy_static! {
    static ref TYPES: HashMap<String, Type> =
        HashMap::from([("int".to_string(), Type::Integer(true, 64))]);
}

pub trait Parse: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}

pub trait LowerToCodegem {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>>;
}

trait GetType {
    fn get_type(&self) -> Result<Type>;
}
