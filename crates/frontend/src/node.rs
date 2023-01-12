use std::rc::Rc;

use lang_pt::{
    production::{Concat, EOFProd, Node, SeparatedList, TokenField, TokenFieldSet, Union},
    DefaultParser, NodeImpl,
};

use crate::token::{self, Token};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeValue {
    NULL,
    Id,
    Number,
    Add,
    Sub,
    Mul,
    Div,
    Product,
    Sum,
    Expr,
    Root,
    VarAssign,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

pub fn parser() -> DefaultParser<NodeValue, Token> {
    let identifier = Rc::new(TokenField::new(Token::Id, Some(NodeValue::Id)));
    let number = Rc::new(TokenField::new(Token::Number, Some(NodeValue::Number)));
    let end_of_file = Rc::new(EOFProd::new(None));

    let add_ops = Rc::new(TokenFieldSet::new(vec![
        (Token::Add, Some(NodeValue::Add)),
        (Token::Sub, Some(NodeValue::Sub)),
    ]));
    let mul_ops = Rc::new(TokenFieldSet::new(vec![
        (Token::Mul, Some(NodeValue::Mul)),
        (Token::Div, Some(NodeValue::Div)),
    ]));

    let paren_expr = Rc::new(Concat::init("paren_expr"));

    let value = Rc::new(Union::new(
        "value",
        vec![number, identifier.clone(), paren_expr.clone()],
    ));

    let product = Rc::new(SeparatedList::new(&value, &mul_ops, true));
    let product_node = Rc::new(Node::new(&product, NodeValue::Product));
    let sum = Rc::new(SeparatedList::new(&product_node, &add_ops, true));
    let sum_node = Rc::new(Node::new(&sum, NodeValue::Sum));
    let let_ = Rc::new(Concat::new(
        "let",
        vec![
            Rc::new(TokenField::new(Token::Let, None)),
            identifier.clone(),
            Rc::new(TokenField::new(Token::Assign, None)),
            sum_node.clone(),
        ],
    ));
    let let_node = Rc::new(Node::new(&let_, NodeValue::VarAssign));
    let semicolon = Rc::new(TokenField::new(Token::Semicolon, None));
    let expression = Rc::new(Union::new(
        "expression",
        vec![let_node.clone(), sum_node.clone()],
    ));
    let expr_node = Rc::new(Node::new(&expression, NodeValue::Expr));
    let exprs = Rc::new(SeparatedList::new(&expr_node, &semicolon, false));
    let root = Rc::new(Concat::new("root", vec![exprs.clone(), end_of_file]));
    let root_node = Rc::new(Node::new(&root, NodeValue::Root));

    let open_paren = Rc::new(TokenField::new(Token::OpenParen, None));
    let close_paren = Rc::new(TokenField::new(Token::CloseParen, None));
    paren_expr
        .set_symbols(vec![open_paren, sum.clone(), close_paren])
        .unwrap();

    let parser = DefaultParser::new(Rc::new(token::tokenizer()), root_node).unwrap();
    parser
}
