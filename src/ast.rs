use parser::Token;
use parser::Op;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Node {
    left: Option<Box<Node>>,
    val: Token,
    right: Option<Box<Node>>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            left: None,
            right: None,
            val: Token::None,
        }
    }

    pub fn left(mut self, l: &Node) -> Node {
        self.left = Some(Box::new(l.clone()));
        self
    }

    pub fn right(mut self, r: &Node) -> Node {
        self.right = Some(Box::new(r.clone()));
        self
    }

    pub fn val(mut self, v: &Token) -> Node {
        self.val = v.clone();
        self
    }

    pub fn get_val(&self) -> &Token {
        &self.val
    }

    pub fn make_node(v: Token) -> Node {
        Node {
            left: None,
            val: v,
            right: None,
        }
    }

    pub fn po_(n: &Node) {
        match &n.left {
            &Some(ref t) => Node::po_(&t),
            &None => {}
        }

        match &n.right {
            &Some(ref t) => Node::po_(&t),
            &None => {}
        }

        println!("{:?}", &n.val);
    }

    pub fn eval(&self) -> f64 {
        match &self.val {
            &Token::Operator(Op::Add) => match (&self.left, &self.right) {
                (&Some(ref x), &Some(ref y)) => x.eval() + y.eval(),
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Sub) => match (&self.left, &self.right) {
                (&Some(ref x), &Some(ref y)) => x.eval() - y.eval(),
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Mul) => match (&self.left, &self.right) {
                (&Some(ref x), &Some(ref y)) => x.eval() * y.eval(),
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Div) => match (&self.left, &self.right) {
                (&Some(ref x), &Some(ref y)) => {
                    let d = y.eval();
                    if d != 0.0 {
                        x.eval() / d
                    } else {
                        panic!("Divide by zero at {:?}\n", self);
                    }
                }
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Mod) => match (&self.left, &self.right) {
                (&Some(ref x), &Some(ref y)) => {
                    let d = y.eval() as i64;
                    if d != 0 {
                        ((x.eval() as i64) % d) as f64
                    } else {
                        panic!("Divide by zero at {:?}\n", self);
                    }
                }
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Pow) => match (&self.left, &self.right) {
                (&Some(ref x), &Some(ref y)) => x.eval().powi(y.eval() as i32),
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Pos) => match &self.right {
                &Some(ref y) => y.eval(),
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Operator(Op::Neg) => match &self.right {
                &Some(ref y) => y.eval() * -1.,
                _ => panic!("Eval error!\n Tree structure- {:?}", self),
            },
            &Token::Number(x) => x,
            _ => panic!("Eval error!\n Tree structure- {:?}", self),
        }
    }
}
