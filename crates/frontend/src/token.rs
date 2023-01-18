use std::rc::Rc;

use lang_pt::{
    lexeme::{Mapper, Pattern, Punctuations},
    TokenImpl, Tokenizer,
};

#[allow(dead_code)]
#[derive(Debug, Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Token {
    Id,
    Number,
    Add,
    Sub,
    Mul,
    Div,
    Space,
    Semicolon,
    Colon,
    Comma,
    LineBreak,
    Assign,
    EOF,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    If,
    Else,
    While,
    For,
    True,
    False,
    Let,
    Null,
    Fn,
    Undefined,
    TypeArrow,
    Extern,
    GT,
    GTE,
    EQ,
    LT,
    LTE,
}

impl TokenImpl for Token {
    fn eof() -> Self {
        Self::EOF
    }

    fn is_structural(&self) -> bool {
        match self {
            Token::Space | Token::LineBreak => false,
            _ => true,
        }
    }
}

pub fn tokenizer() -> Tokenizer<Token> {
    let identifier: Pattern<Token> = Pattern::new(Token::Id, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
    let mapping_identifier = Mapper::new(
        identifier,
        vec![
            ("if", Token::If),
            ("else", Token::Else),
            ("while", Token::While),
            ("for", Token::For),
            ("true", Token::True),
            ("false", Token::False),
            ("null", Token::Null),
            ("undefined", Token::Undefined),
            ("let", Token::Let),
            ("fn", Token::Fn),
            ("extern", Token::Extern),
        ],
    )
    .unwrap();
    let number_literal =
        Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap();
    let non_break_space: Pattern<Token> = Pattern::new(Token::Space, r"^[^\S\r\n]+").unwrap();
    let line_break: Pattern<Token> = Pattern::new(Token::LineBreak, r"^[\r\n]+").unwrap();
    let expression_punctuations: Punctuations<Token> = Punctuations::new(vec![
        ("+", Token::Add),
        ("-", Token::Sub),
        ("*", Token::Mul),
        ("/", Token::Div),
        ("<", Token::LT),
        ("<=", Token::LTE),
        (">", Token::GT),
        (">=", Token::GTE),
        ("==", Token::EQ),
        ("=", Token::Assign),
        ("{", Token::OpenBrace),
        ("}", Token::CloseBrace),
        ("(", Token::OpenParen),
        (")", Token::CloseParen),
        ("[", Token::OpenBracket),
        ("]", Token::CloseBracket),
        (";", Token::Semicolon),
        ("->", Token::TypeArrow),
        (":", Token::Colon),
        (",", Token::Comma),
    ])
    .unwrap();

    let tokenizer = Tokenizer::new(vec![
        Rc::new(non_break_space),
        Rc::new(mapping_identifier),
        Rc::new(number_literal),
        Rc::new(expression_punctuations),
        Rc::new(line_break),
    ]);

    tokenizer
}
