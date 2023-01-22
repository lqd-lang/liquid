pub mod codepass;
pub mod linkage;
pub mod make_signatures;
pub mod parsepass;
pub mod type_;
pub mod type_check;

use std::collections::VecDeque;

use miette::*;
use thiserror::Error;

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
    #[error("Expected {}, found {}", .0, .1)]
    TypeMismatch(
        String,
        String,
        #[label("This should be a {}", .0)] SourceSpan,
    ),
}

#[derive(Error, Diagnostic, Debug)]
#[error("")]
pub struct Labelled<E: Diagnostic + 'static> {
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
pub trait IntoLabelled {
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

#[derive(PartialEq)]
pub enum ScopeType {
    Extern,
}

pub struct Stack<T>(VecDeque<T>);

impl<T> Stack<T> {
    pub fn push(&mut self, t: T) {
        self.0.push_back(t);
    }
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop_back()
    }
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn iter(&mut self) -> std::collections::vec_deque::Iter<T> {
        self.0.iter()
    }
}
