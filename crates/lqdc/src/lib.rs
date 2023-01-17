use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "backend_codegem")]
pub mod codegem;
#[cfg(feature = "backend_cranelift")]
pub mod cranelift;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("Variable {} does not exist", .0)]
    VarDoesntExist(String),
    #[error("Function {} does not exist", .0)]
    FuncDoesntExist(String),
    #[error("{}", .0)]
    ParseError(String),
    #[error("Unknown return type")]
    UnknownReturnType,
    #[error("Unknown type")]
    UnknownType,
    #[error("Internal compiler error: {}", .0)]
    InternalCompilerError(String),
    #[error("{} not allowed in {}", .0, .1)]
    NotAllowedHere(String, String),
    #[error("Expected {} args, found {}", .0, .1)]
    ExpectedNumArgs(usize, usize),
    #[error("Malformed integer")]
    InvalidInteger,
}

#[derive(Error, Diagnostic, Debug)]
#[error("")]
struct Labelled<E: Diagnostic + 'static> {
    #[source]
    #[diagnostic_source]
    source: E,
    #[label]
    label: SourceSpan,
}
// impl<E: Diagnostic + 'static> Labelled<E> {
//     fn new(source: E, label: SourceSpan) -> Self {
//         Self { source, label }
//     }
// }
trait IntoLabelled {
    fn labelled(self, label: SourceSpan) -> Labelled<Self>
    where
        Self: Diagnostic + 'static + Sized;
}
impl<E: Diagnostic + 'static + Sized> IntoLabelled for E {
    fn labelled(self, label: SourceSpan) -> Labelled<Self>
    where
        Self: Diagnostic + 'static + Sized,
    {
        Labelled {
            source: self,
            label,
        }
    }
}
