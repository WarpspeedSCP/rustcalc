use parser::Op;
use parser::TokStruct;

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