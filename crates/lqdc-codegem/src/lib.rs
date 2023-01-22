pub mod codegen;

use codegem::ir::ModuleCreationError;
use lqdc_common::{linkage::Linkage, type_::Type};
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
        Type::Bool => codegem::ir::Type::Integer(false, 8),
        Type::Void => codegem::ir::Type::Void,
        Type::Uint => codegem::ir::Type::Integer(false, 64),
        Type::Number => unreachable!(),
    }
}
pub(crate) fn map_linkage(linkage: &Linkage) -> codegem::ir::Linkage {
    match linkage {
        Linkage::Private => codegem::ir::Linkage::Private,
        Linkage::Public => codegem::ir::Linkage::Public,
        Linkage::External => codegem::ir::Linkage::External,
    }
}
