use std::str::FromStr;

use lqdc_common::Error;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Type {
    Int,
    Bool,
    Void,
}

impl FromStr for Type {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Self::Int),
            "bool" => Ok(Self::Bool),
            "void" => Ok(Self::Void),
            _ => Err(Error::UnknownType),
        }
    }
}

pub fn map_type(type_: Type) -> codegem::ir::Type {
    match type_ {
        Type::Int => codegem::ir::Type::Integer(true, 64),
        Type::Bool => codegem::ir::Type::Integer(false, 1),
        Type::Void => codegem::ir::Type::Void,
    }
}
