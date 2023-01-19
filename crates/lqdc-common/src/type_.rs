use std::str::FromStr;

use crate::Error;

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
