extern crate num;
extern crate std;

use ast::Node;

use std::mem::discriminant;

#[derive(Clone, Debug, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Pos,
    Neg,
    Eq_,
    Gt_,
    Lt_,
    Geq,
    Leq,
    Neq,
    And,
    Or_,
    Not,
    BitAnd,
    BitOr,
    BitNot,
    BitXor,
    LLS,
    LRS,
    ARS,
    Assign,
    Comma,
    LParens,
    RParens,
    Any,
}

impl PartialEq for Op {
    fn eq(&self, other: &Op) -> bool {
        discriminant(self) == discriminant(other)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
enum ValType {
    Bool(bool),
    Number(f64),
}

#[derive(Clone, Debug)]
#[repr(C)]
struct Variable {
    name: String,
    val: ValType,
}

#[derive(Clone, Debug)]
pub enum Token {
    Number(f64),
    Operator(Op),
    UnaryFn(fn(f64) -> f64),
    BinaryFn(fn(f64, f64) -> f64),
    Var(Variable),
    Other(char),
    None,
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        match (self, other) {
            (&Token::Operator(Op::Any), &Token::Operator(ref y)) => true,
            (&Token::Operator(ref x), &Token::Operator(ref y)) => x == y,
            _ => discriminant(self) == discriminant(other),
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    pos: usize,
    base: u32,
    curr: Token,
    AST: Node,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            base: 10,
            curr: Token::None,
            pos: 0,
            AST: Node::new(),
        }
    }

    pub fn set_base(&mut self, base: u32) {
        self.base = base;
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.curr = Token::None;
    }

    fn peek(&mut self, input: &[u8]) -> char {
        input[self.pos + 1] as char
    }

    pub fn eval(&mut self, input: &[u8]) -> Node {
        self.reset();
        self.get_next(input);

        self.expr(input)
    }

    pub fn get_number(&mut self, input: &[u8]) -> f64 {
        let mut num: String = "".into();

        while self.pos < input.len()
            && ((input[self.pos] as char).is_digit(self.base) || input[self.pos] as char == '.')
        {
            num.push(input[self.pos] as char);
            self.pos += 1;
        }

        match num.parse() {
            Ok(val) => val,
            Err(e) => panic!("\"{}\" is not a number!", num),
        }
    }

    fn get_next(&mut self, input: &[u8]) -> Token {
        if !(self.pos < input.len()) {
            self.curr = Token::None;
            return self.curr.clone();
        }

        let mut ch = input[self.pos] as char;

        if ch == ' ' {
            while ch == ' ' {
                if self.pos < (input.len() - 1) {
                    self.pos += 1;
                } else {
                    return Token::Other(ch);
                }
                ch = input[self.pos] as char;
            }
        }

        if ch.is_digit(self.base) || ch == '.' {
            self.curr = Token::Number(self.get_number(input))
        } else {
            //if !ch.is_alphanumeric() {
            self.pos += 1;
            self.curr = match ch {
                '+' => match self.curr.clone() {
                    Token::Number(_) => Token::Operator(Op::Add),
                    Token::Var(_) => Token::Operator(Op::Add),
                    _ => Token::Operator(Op::Pos),
                },
                '-' => match self.curr.clone() {
                    Token::Number(_) => Token::Operator(Op::Sub),
                    Token::Var(_) => Token::Operator(Op::Sub),
                    _ => Token::Operator(Op::Neg),
                },
                '*' => Token::Operator(Op::Mul),
                '/' => Token::Operator(Op::Div),
                '^' => Token::Operator(Op::Pow),
                '%' => Token::Operator(Op::Mod),
                '(' => Token::Operator(Op::LParens),
                ')' => Token::Operator(Op::RParens),
                '=' => if self.peek(input) == ' ' {
                    Token::Operator(Op::Eq_)
                } else {
                    Token::Operator(Op::Assign)
                },
                _ => Token::Other(ch),
            };
        }

        self.curr.clone()
    }

    fn eat(&mut self, token: Token, input: &[u8]) {
        if self.curr == token {
            self.curr = self.get_next(input);
        } else {
            panic!(
                "Parse error! Expected {:?}, got {:?} at position {}!",
                token, self.curr, self.pos
            );
        }
    }

    fn number(&mut self, input: &[u8]) -> Node {
        self.eat(Token::Number(0.0), input);
        Node::make_node(self.curr.clone())
    }

    fn factor(&mut self, input: &[u8]) -> Node {
        let mut t = Node::make_node(self.curr.clone());

        if *t.get_val() == Token::Number(0.00) {
            self.number(input);
        } else {
            match *t.get_val() {
                Token::Operator(Op::LParens) => {
                    self.eat(Token::Operator(Op::LParens), input);
                    t = self.expr(input);
                    self.eat(Token::Operator(Op::RParens), input);
                }
                Token::Operator(Op::Pow) => {
                    self.eat(Token::Operator(Op::Pow), input);
                    t = self.pow_factor(input);
                }
                Token::Operator(Op::Pos) => {
                    self.eat(Token::Operator(Op::Pos), input);
                    t = Node::new().val(t.get_val()).right(&self.factor(input));
                }
                Token::Operator(Op::Neg) => {
                    self.eat(Token::Operator(Op::Neg), input);
                    t = Node::new().val(t.get_val()).right(&self.factor(input));
                }
                _ => t = t.val(&Token::None),
            };
        }
        t
    }

    fn pow_factor(&mut self, input: &[u8]) -> Node {
        let mut res = self.factor(input);

        while match self.curr {
            Token::Operator(Op::Pow) => true,
            _ => false,
        } {
            let ser = res.clone();
            res = res.left(&ser)
                .val(&self.curr.clone())
                .right(&self.factor(input));
        }

        res
    }

    fn term(&mut self, input: &[u8]) -> Node {
        let mut res = self.pow_factor(input);
        let mut tok = Token::None;

        while match self.curr {
            Token::Operator(Op::Mul) => true,
            Token::Operator(Op::Div) => true,
            Token::Operator(Op::Mod) => true,
            _ => false,
        } {
            tok = self.curr.clone();

            match tok {
                Token::Operator(Op::Mul) => {
                    self.eat(Token::Operator(Op::Mul), input);
                }

                Token::Operator(Op::Div) => {
                    self.eat(Token::Operator(Op::Div), input);
                }

                Token::Operator(Op::Mod) => {
                    self.eat(Token::Operator(Op::Mod), input);
                }

                _ => panic!(
                    "Expected {:?} but got {:?} at position {}!",
                    Token::Operator(Op::Any),
                    tok,
                    self.pos
                ),
            }

            let ser = res.clone();
            res = res.left(&ser).val(&tok).right(&self.pow_factor(input));
        }

        res
    }

    fn expr(&mut self, input: &[u8]) -> Node {
        let mut res = self.term(input);
        let mut tok = Token::None;

        while match self.curr.clone() {
            Token::Operator(Op::Add) => true,
            Token::Operator(Op::Sub) => true,
            _ => false,
        } {
            tok = self.curr.clone();

            match tok {
                Token::Operator(Op::Add) => {
                    self.eat(Token::Operator(Op::Add), input);
                }
                Token::Operator(Op::Sub) => {
                    self.eat(Token::Operator(Op::Sub), input);
                }
                _ => panic!(
                    "Expected {:?} but got {:?} at position {}!",
                    Token::Operator(Op::Any),
                    tok,
                    self.pos
                ),
            }

            let ser = res.clone();
            res = res.left(&ser).val(&tok).right(&self.term(input));
        }

        res
    }

    fn id(&mut self, input: &[u8]) -> Node {
        Node::new()
    }
}
