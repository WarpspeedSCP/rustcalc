extern crate termion;

use parser::Token;
use parser::TokStruct;

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum NodeType {
    Block,
    Assignment,
    AExpression,
    BExpression,
    FnCall,
    FnDef,
    FnArg,
    FnArgs,
    None,
    Cond,
    Program,
    Return
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

    pub fn add_child(mut self, n: Node) -> Node {
        self.children.push(n);
        self.clone()
    }

    pub fn val(mut self, v: TokStruct) -> Node {
        self.val = v;
        self.clone()
    }

    pub fn children(mut self, c: Vec<Node>) -> Node {
        self.children = c;
        self.clone()
    }

    pub fn add_children(mut self, c: &mut Vec<Node>) -> Node {
        self.children.append(c);
        self
    }

    pub fn type_(mut self, t: NodeType) -> Node {
        self.n_type = t.clone();
        self.clone()
    }

    pub fn get_val(&self) -> Token {
        self.val.get_val()
    }

    pub fn get_children(&self) -> &Vec<Node> {
        &self.children
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
