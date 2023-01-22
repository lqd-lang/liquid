use frontend::{node::NodeValue, parser};
use lang_pt::ASTNode;
use lqdc_common::{
    codepass::{CodePass, Is},
    Error, IntoLabelled,
};

pub struct ParsePass {
    pub(crate) nodes: Vec<ASTNode<NodeValue>>,
}
impl<'input> CodePass<'input> for ParsePass {
    type Prev = ();

    type Arg = ();

    fn pass(_: Self::Prev, input: &'input str, _: &mut impl Is<Self::Arg>) -> miette::Result<Self> {
        Ok(Self {
            nodes: parser()
                .parse(input.as_bytes())
                .map_err(|e| Error::ParseError(e.message).labelled(e.pointer.into()))?,
        })
    }
}

impl Into<Vec<ASTNode<NodeValue>>> for ParsePass {
    fn into(self) -> Vec<ASTNode<NodeValue>> {
        self.nodes
    }
}
