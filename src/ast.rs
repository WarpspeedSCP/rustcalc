use parser::Op;
use serde_json::{Value, Error};
use parser::Token;
use parser::TokStruct;



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Return,
    Var,
    Number,
    Bool
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Sym {
    name: String
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum Num {
    Int(i64),
    Float(f64)
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum Terminal {
    Number(Num),
    String(String),
    Symbol(Sym)
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum Sign {
    Pos,
    Neg
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct ArgList {
    argc: u8,
    argv: Vec<Factor>
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub struct FnCall {
    name: String,
    args: ArgList
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub enum Factor {
    Terminal(Terminal),
    FnCall(Box<FnCall>),
    Expr(Box<Expr>),
    Power {
        base: Box<Factor>, 
        exponent: Box<Factor>
    },
    Signed {
        val: Box<Factor>,
        sign: Sign
    },
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub enum Expr {
    Binary {
        left: Box<Factor>,
        right: Box<Factor>,
        op: Op
    },
    Unary {
        right: Box<Factor>,
        op: Op
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub enum Statement {
    Expr(Box<Expr>),
    Assignment(Box<AssignStmt>),
    Return(Box<ReturnStmt>),
    Condition(Box<ConditionalStmt>),
    Scope(Box<Scope>)
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub struct Scope {
    contents: Vec<Statement>
}


#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub struct AssignStmt {
    left: Terminal,
    right: Box<Factor>
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub struct ReturnStmt {
    val: Box<Factor>
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
#[allow(unused)]
pub struct ConditionalStmt {
    condition: Box<Factor>,
    body: Box<Scope>,
    alternates: Vec<ConditionalStmt>,
    else_: Option<Scope>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Node {
    val: TokStruct,
    children: Vec<Node>,
    n_type: NodeType,
}

impl Node {
    pub fn new() -> Node {
        Node {
            val: TokStruct::default(),
            children: Vec::new(),
            n_type: NodeType::None,
        }
    }

    pub fn add_child(mut self, n: Node) -> Node {
        self.children.push(n);
        self
    }

    pub fn val(mut self, v: TokStruct) -> Node {
        self.val = v;
        self
    }

    pub fn children(mut self, c: Vec<Node>) -> Node {
        self.children = c;
        self
    }

    pub fn add_children(mut self, c: &mut Vec<Node>) -> Node {
        self.children.append(c);
        self
    }

    pub fn type_(mut self, t: NodeType) -> Node {
        self.n_type = t.clone();
        self
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


    pub fn po_(n: &Node) {
        for i in &n.children {
            Node::po_(i);
        }

        println!("{:?} : {:?}", n.val, n.n_type);
    }

}

