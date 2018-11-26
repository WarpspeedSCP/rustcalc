use parser::Op;
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

pub trait NodeT {}

pub trait EncapsulatingNode: NodeT {}

pub trait TerminalNode: NodeT {}

#[derive(Debug, Clone, Default)]
#[allow(unused)]
#[repr(C)]
pub struct Program {
    pub content: Vec<Function>
}

#[derive(Debug, Clone, EncapsulatingNode, Default)]
#[allow(unused)]
#[repr(C)]
pub struct ArgList {    
    pub argv: Vec<Expr>
}

#[derive(Debug, Clone, EncapsulatingNode, Default)]
#[allow(unused)]
#[repr(C)]
pub struct ArgDeclList {
    pub argv: Vec<TokStruct>
}

#[derive(Debug, Clone, EncapsulatingNode, Default)]
#[allow(unused)]
#[repr(C)]
pub struct FnCall {
    pub name: TokStruct,
    pub args: ArgList
}

#[derive(Debug, Clone, NodeT)]
#[allow(unused)]
#[repr(C)]
pub enum Factor {
    Int(TokStruct),
    Float(TokStruct),
    String(TokStruct),
    Symbol(TokStruct),
    Bool(TokStruct),
    FnCall(FnCall),
    Expr(Expr),
    None
}

impl Factor {
    pub fn as_fn_call(&mut self) -> Option<&mut FnCall> {
        match self {
            Factor::FnCall(f) => Some(f),
            _ => None
        }
    }
}

#[derive(Debug, Clone, NodeT)]
#[allow(unused)]
#[repr(C)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        op: Op
    },
    Unary {
        right: Box<Expr>,
        op: Op
    },
    Factor(Box<Factor>)
}

#[derive(Debug, Clone, EncapsulatingNode)]
#[allow(unused)]
#[repr(C)]
pub struct CondBlock {
    pub cond: Expr, 
    pub body: Box<Statement>
}

#[derive(Debug, Clone, EncapsulatingNode)]
#[allow(unused)]
#[repr(C)]
pub enum Statement {
    Expr(Expr),
    Assign {
        left: Expr,
        right: Expr
    },
    Return {
        val: Expr
    },
    Branch {
        if_block: CondBlock,
        alt_blocks: Vec<CondBlock>,
        else_block: Option<Box<Statement>>
    },
    FnDecl(Function),
    Scope(Scope)
}

#[derive(Debug, Clone, EncapsulatingNode, Default)]
#[allow(unused)]
#[repr(C)]
pub struct Scope {
    pub contents: Vec<Statement>
}

#[derive(Debug, Clone, EncapsulatingNode)]
#[allow(unused)]
#[repr(C)]
pub struct Function {
    pub name: TokStruct,
    pub args: ArgDeclList,
    pub body: Scope
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(unused)]
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

