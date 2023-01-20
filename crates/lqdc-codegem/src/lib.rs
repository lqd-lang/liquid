pub mod codegen;
pub mod make_signatures;
pub mod parsepass;
pub mod type_check;

use codegem::ir::ModuleCreationError;
use lqdc_common::type_::Type;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum CodegemError {
    #[error("{:?}", .0)]
    ModuleCreationError(ModuleCreationError),
}

pub fn map_type(type_: Type) -> codegem::ir::Type {
    match type_ {
        Type::Int => codegem::ir::Type::Integer(true, 64),
        Type::Bool => codegem::ir::Type::Integer(false, 1),
        Type::Void => codegem::ir::Type::Void,
    }
}
