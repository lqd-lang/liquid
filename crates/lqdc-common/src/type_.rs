use std::str::FromStr;

use miette::*;

use crate::Error;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Type {
    // Specifics
    Int,
    Bool,
    Void,
    Uint,

    // Inferables
    Number,
}

impl Type {
    pub fn coerce(self, to: Type) -> Result<Type> {
        match self {
            Type::Int | Type::Bool | Type::Void | Type::Uint => {
                if self == to {
                    Ok(to)
                } else {
                    Err(miette!("Cannot coerce {:?} to {:?}", self, to))
                }
            }
            Type::Number => match to {
                Type::Int | Type::Uint | Type::Number => Ok(to),
                Type::Bool | Type::Void => Err(miette!("Cannot coerce {:?} to {:?}", self, to)),
            },
        }
    }
}

impl FromStr for Type {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Self::Int),
            "bool" => Ok(Self::Bool),
            "void" => Ok(Self::Void),
            "uint" => Ok(Self::Uint),
            _ => Err(Error::UnknownType),
        }
    }
}
