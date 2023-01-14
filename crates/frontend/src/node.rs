use std::rc::Rc;

use lang_pt::{
    production::{
        Concat, EOFProd, List, Node, Nullable, SeparatedList, TokenField, TokenFieldSet, Union,
    },
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
    FnDef,
    FnCall,
    FnDefArgSet,
    FnCallArgSet,
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

    let value = Rc::new(Union::init("value"));

    let product = Rc::new(SeparatedList::new(&value, &mul_ops, true));
    let product_node = Rc::new(Node::new(&product, NodeValue::Product));
    let sum = Rc::new(SeparatedList::new(&product_node, &add_ops, true));
    let sum_node = Rc::new(Node::new(&sum, NodeValue::Sum));
    let fn_call = Rc::new(Concat::init("call"));
    let fn_call_node = Rc::new(Node::new(&fn_call, NodeValue::FnCall));
    let let_ = Rc::new(Concat::init("let"));
    let let_node = Rc::new(Node::new(&let_, NodeValue::VarAssign));
    let fn_def = Rc::new(Concat::init("fn_def"));
    let fn_def_node = Rc::new(Node::new(&fn_def, NodeValue::FnDef));
    let semicolon = Rc::new(TokenField::new(Token::Semicolon, None));
    let expression = Rc::new(Union::new(
        "expression",
        vec![
            fn_call_node.clone(),
            let_node.clone(),
            fn_def_node.clone(),
            sum_node.clone(),
        ],
    ));
    let expr_node = Rc::new(Node::new(&expression, NodeValue::Expr));
    let exprs = Rc::new(SeparatedList::new(&expr_node, &semicolon, true));
    let root = Rc::new(Concat::new(
        "root",
        vec![Rc::new(List::new(&fn_def_node)), end_of_file],
    ));
    let root_node = Rc::new(Node::new(&root, NodeValue::Root));

    let open_paren = Rc::new(TokenField::new(Token::OpenParen, None));
    let close_paren = Rc::new(TokenField::new(Token::CloseParen, None));
    paren_expr
        .set_symbols(vec![open_paren.clone(), sum.clone(), close_paren.clone()])
        .unwrap();

    let typed_identifier = Rc::new(Concat::new(
        "typed_identifier",
        vec![
            identifier.clone(),
            Rc::new(TokenField::new(Token::Colon, None)),
            identifier.clone(),
        ],
    ));

    let comma = Rc::new(TokenField::new(Token::Comma, None));
    let fn_def_arg_set = Rc::new(Concat::new(
        "fn_def_arg_set",
        vec![
            open_paren.clone(),
            Rc::new(SeparatedList::new(&typed_identifier, &comma, false)),
            close_paren.clone(),
        ],
    ));
    let fn_def_arg_set_node = Rc::new(Node::new(&fn_def_arg_set, NodeValue::FnDefArgSet));
    fn_def
        .set_symbols(vec![
            Rc::new(TokenField::new(Token::Fn, None)),
            identifier.clone(),
            Rc::new(Nullable::new(&fn_def_arg_set_node)),
            Rc::new(TokenField::new(Token::TypeArrow, None)),
            identifier.clone(),
            Rc::new(TokenField::new(Token::OpenBrace, None)),
            exprs.clone(),
            Rc::new(TokenField::new(Token::CloseBrace, None)),
        ])
        .unwrap();
    let fn_call_arg_set = Rc::new(SeparatedList::new(&expr_node, &comma, false));
    let fn_call_arg_set_node = Rc::new(Node::new(&fn_call_arg_set, NodeValue::FnCallArgSet));
    fn_call
        .set_symbols(vec![
            identifier.clone(),
            Rc::new(TokenField::new(Token::OpenParen, None)),
            Rc::new(Nullable::new(&fn_call_arg_set_node)),
            Rc::new(TokenField::new(Token::CloseParen, None)),
        ])
        .unwrap();
    let_.set_symbols(vec![
        Rc::new(TokenField::new(Token::Let, None)),
        identifier.clone(),
        Rc::new(TokenField::new(Token::Assign, None)),
        expr_node.clone(),
    ])
    .unwrap();
    value
        .set_symbols(vec![
            fn_call_node.clone(),
            number,
            identifier.clone(),
            paren_expr.clone(),
        ])
        .unwrap();

    let parser = DefaultParser::new(Rc::new(token::tokenizer()), root_node).unwrap();
    parser
}
