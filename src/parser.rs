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
pub enum Token {
    Number(f32),
    Operator(Op),
    UnaryFn(fn(f32) -> f32),
    BinaryFn(fn(f32, f32) -> f32),
    Var(String),
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
    ans: Token,
    AST: Node,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            base: 10,
            ans: Token::None,
            pos: 0,
            AST: Node::new(),
        }
    }

    pub fn set_base(&mut self, base: u32) {
        self.base = base;
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.ans = Token::None;
    }

    pub fn eval(&mut self, input: &[u8]) -> f32 {
        self.reset();
        self.get_next(input);

        match self.expr(input) {
            Token::Number(x) => x,
            _ => std::f32::NAN,
        }
    }

    fn get_number(&mut self, input: &[u8]) -> f32 {
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
            self.ans = Token::None;
            return self.ans.clone();
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
            self.ans = Token::Number(self.get_number(input))
        } else {
            //if !ch.is_alphanumeric() {
            self.pos += 1;
            self.ans = match ch {
                '+' => match self.ans.clone() {
                    Token::Number(_) => Token::Operator(Op::Add),
                    Token::Var(_) => Token::Operator(Op::Add),
                    _ => Token::Operator(Op::Pos),
                },
                '-' => match self.ans.clone() {
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
                _ => Token::Other(ch),
            };
        }

        self.ans.clone()
    }

    fn eat(&mut self, token: Token, input: &[u8]) {
        if self.ans == token {
            self.ans = self.get_next(input);
        } else {
            panic!(
                "Parse error! Expected {:?}, got {:?} at position {}!",
                token, self.ans, self.pos
            );
        }
    }

    fn factor(&mut self, input: &[u8]) -> Token {
        let mut t = self.ans.clone();
        if t == Token::Number(0.00) {
            self.eat(Token::Number(0.0), input);
        } else {
            match t {
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
                    t = match self.factor(input) {
                        Token::Number(n) => Token::Number(n),
                        _ => t,
                    }
                }
                Token::Operator(Op::Neg) => {
                    self.eat(Token::Operator(Op::Neg), input);
                    t = match self.factor(input) {
                        Token::Number(n) => Token::Number(-n),
                        _ => t,
                    }
                }
                _ => return Token::None,
            };
        }
        t
    }

    fn pow_factor(&mut self, input: &[u8]) -> Token {
        let mut res = self.factor(input);

        while match self.ans {
            Token::Operator(Op::Pow) => true,
            _ => false,
        } {
            res = match (res, self.factor(input)) {
                (Token::Number(x), Token::Number(y)) => Token::Number(x.powi(y as i32)),
                _ => Token::None,
            }
        }

        res
    }

    fn term(&mut self, input: &[u8]) -> Token {
        let mut res = self.pow_factor(input);
        let mut tok = Token::None;

        while match self.ans {
            Token::Operator(Op::Mul) => true,
            Token::Operator(Op::Div) => true,
            Token::Operator(Op::Mod) => true,
            _ => false,
        } {
            tok = self.ans.clone();
            match tok {
                Token::Operator(Op::Mul) => {
                    self.eat(Token::Operator(Op::Mul), input);
                    res = match (res, self.pow_factor(input)) {
                        (Token::Number(x), Token::Number(y)) => Token::Number(x * y),
                        _ => panic!("Unable to multiply at position {}", self.pos),
                    }
                }

                Token::Operator(Op::Div) => {
                    self.eat(Token::Operator(Op::Div), input);
                    res = match (res, self.pow_factor(input)) {
                        (Token::Number(x), Token::Number(y)) => if y != 0.0 {
                            Token::Number(x / y)
                        } else {
                            panic!("Divide by zero at {}!", self.pos)
                        },
                        _ => panic!("Unable to divide at position {}", self.pos),
                    }
                }

                Token::Operator(Op::Mod) => {
                    self.eat(Token::Operator(Op::Mod), input);
                    res = match (res, self.pow_factor(input)) {
                        (Token::Number(x), Token::Number(y)) => if y != 0.0 {
                            Token::Number(x % y)
                        } else {
                            panic!("Divide by zero at {}!", self.pos)
                        },
                        _ => panic!("Unable to find modulo at position {}", self.pos),
                    }
                }

                _ => panic!(
                    "Expected {:?} but got {:?} at position {}!",
                    Token::Operator(Op::Any),
                    tok,
                    self.pos
                ),
            }
        }

        res
    }

    fn expr(&mut self, input: &[u8]) -> Token {
        let mut tok = Token::None;

        let mut res = self.term(input);

        while match self.ans.clone() {
            Token::Operator(Op::Add) => true,
            Token::Operator(Op::Sub) => true,
            _ => false,
        } {
            tok = self.ans.clone();

            res = match tok {
                Token::Operator(Op::Add) => {
                    self.eat(Token::Operator(Op::Add), input);
                    match (res, self.term(input)) {
                        (Token::Number(x), Token::Number(y)) => Token::Number(x + y),
                        _ => Token::None,
                    }
                }
                Token::Operator(Op::Sub) => {
                    self.eat(Token::Operator(Op::Sub), input);
                    match (res, self.term(input)) {
                        (Token::Number(x), Token::Number(y)) => Token::Number(x - y),
                        _ => Token::None,
                    }
                }
                _ => panic!(
                    "Expected {:?} but got {:?} at position {}!",
                    Token::Operator(Op::Any),
                    tok,
                    self.pos
                ),
            }
        }

        res
    }
}
