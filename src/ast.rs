extern crate std;

use parser::Token;
use parser::ValType;
use parser::Op;

pub type SymTable = std::collections::HashMap<String, ValType>;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Node {
    children: Vec<Node>,
    val: Token,
}

impl Node {
    pub fn new() -> Node {
        Node {
            children: Vec::new(),
            val: Token::None,
        }
    }

    pub fn add_child(mut self, n: &Node) -> Node {
        self.children.push(n.clone());
        self
    }

    pub fn val(mut self, v: &Token) -> Node {
        self.val = v.clone();
        self
    }

    pub fn get_val(&self) -> &Token {
        &self.val
    }

    pub fn make_node(v: &Token) -> Node {
        Node {
            children: Vec::new(),
            val: v.clone(),
        }
    }

    pub fn po_(n: &Node) {
        for i in &n.children {
            Node::po_(i);
        }

        println!("{:?}", &n.val);
    }

    pub fn expr_eval(&self, sym_table: &mut SymTable) -> ValType {
        match &self.val {
            &Token::Operator(Op::Add) => match (&self.children[0], &self.children[1]) {
                (ref x, ref y) => x.expr_eval(sym_table) + y.expr_eval(sym_table),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Sub) => match (&self.children[0], &self.children[1]) {
                (ref x, ref y) => x.expr_eval(sym_table) - y.expr_eval(sym_table),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Mul) => match (&self.children[0], &self.children[1]) {
                (ref x, ref y) => x.expr_eval(sym_table) * y.expr_eval(sym_table),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Div) => match (&self.children[0], &self.children[1]) {
                (ref x, ref y) => x.expr_eval(sym_table) / y.expr_eval(sym_table),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Mod) => match (&self.children[0], &self.children[1]) {
                (ref x, ref y) => x.expr_eval(sym_table) % y.expr_eval(sym_table),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Pow) => match (&self.children[0], &self.children[1]) {
                (ref x, ref y) => ValType::Number(
                    f64::from(x.expr_eval(sym_table))
                        .powi(f64::from(y.expr_eval(sym_table)) as i32),
                ),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Pos) => match &self.children[0] {
                ref y => y.expr_eval(sym_table),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Neg) => match &self.children[0] {
                ref y => y.expr_eval(sym_table) * ValType::Number(-1.),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            &Token::Number(x) => ValType::Number(x),
            &Token::Var(ref x) => {
                if sym_table.contains_key(x) {
                    sym_table[x].clone()
                } else {
                    panic!("There is no instance of the variable {} in scope!", x);
                }
            }

            &Token::Operator(Op::Eq_) => match (&self.children[0], &self.children[1]) {
                (ref x, ref y) => ValType::Bool(x.expr_eval(sym_table) == y.expr_eval(sym_table)),
                _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
            },
            _ => panic!("expr_eval error!\n Tree structure- {:?}", self),
        }
    }

    pub fn module_eval(&mut self, sym_table: &mut SymTable) {
        self.children[0].compound_eval(sym_table)
    }

    fn compound_eval(&mut self, sym_table: &mut SymTable) {
        for i in &mut self.children {
            i.statement_eval(sym_table);
        }
    }

    pub fn statement_eval(&mut self, sym_table: &mut SymTable) -> ValType {
        match self.val {
            Token::Operator(Op::Assign) => self.assign_statement_eval(sym_table),
            _ => self.expr_eval(sym_table),
        }
    }

    fn assign_statement_eval(&mut self, sym_table: &mut SymTable) -> ValType {
        let m = self.children[0].val.clone();
        let res = self.children[1].expr_eval(sym_table);
        match &m {
            &Token::Var(ref x) => if sym_table.contains_key(x) {
                sym_table.remove(x);
                sym_table.insert(x.clone(), res.clone());
                self.children[0].val = Token::Var(x.clone());
                return sym_table[x].clone();
            } else {
                sym_table.insert(x.clone(), res.clone());
                self.children[0].val = Token::Var(x.clone());
                return sym_table[x].clone();
            },
            _ => panic!("This shouldn't happen."),
        }
    }
}
