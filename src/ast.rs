extern crate std;
extern crate termion;

use lazy_static::LazyStatic;

use std::fmt;

use parser::Token;
use parser::TokStruct;
use parser::Op;

pub type SymTable = std::collections::HashMap<String, Token>;

lazy_static! {
    pub static ref KEYWORD_TABLE: SymTable = {
        let mut m = SymTable::new();
        m.insert("state".to_owned(), Token::KeyWord);
        m.insert("if".to_owned(), Token::KeyWord);
        m.insert("else".to_owned(), Token::KeyWord);
        m.insert("elif".to_owned(), Token::KeyWord);
        m.insert("endif".to_owned(), Token::KeyWord);
        m.insert("return".to_owned(), Token::KeyWord);
        m.insert("write".to_owned(), Token::KeyWord);
        m.insert("read".to_owned(), Token::KeyWord);
        m.insert("loop".to_owned(), Token::KeyWord);
        m.insert("for".to_owned(), Token::KeyWord);
        m.insert("in".to_owned(), Token::KeyWord);
        m.insert("array".to_owned(), Token::KeyWord);
        m
    };
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum NodeType {
    Function = 1,
    Block = 2,
    Assignment = 3,
    AExpression = 4,
    BExpression = 5,
    FnCall = 6,
    None = 7,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Node {
    children: Vec<Node>,
    val: TokStruct,
    n_type: NodeType,
}

impl Node {
    pub fn new() -> Node {
        Node {
            children: Vec::new(),
            val: TokStruct::new(Token::None, 0),
            n_type: NodeType::None,
        }
    }

    pub fn add_child(&mut self, n: Node) -> Node {
        self.children.push(n);
        self.clone()
    }

    pub fn val(&mut self, v: TokStruct) -> Node {
        self.val = v;
        self.clone()
    }

    pub fn type_(&mut self, t: NodeType) -> Node {
        self.n_type = t.clone();
        self.clone()
    }

    pub fn get_val(&self) -> Token {
        self.val.get_val()
    }

    pub fn get_pos(&self) -> usize {
        self.val.get_pos()
    }

    pub fn make_node(v: TokStruct) -> Node {
        Node {
            children: Vec::new(),
            val: v,
            n_type: NodeType::None,
        }
    }

    pub fn po_(n: &Node) {
        for i in &n.children {
            Node::po_(i);
        }

        println!("{:?} : {:?}", n.val, n.n_type);
    }
}
