extern crate std;

use std::fmt;
use std::hash::Hasher;
use ast::Node;

use std::mem::discriminant;
use std::ops::*;

#[derive(Clone, Debug, Eq, Hash)]
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
    As_Add,
    As_Sub,
    As_Mul,
    As_Div,
    As_Mod,
    As_Pow,
    As_Pos,
    As_Neg,
    As_BitAnd,
    As_BitOr,
    As_BitNot,
    As_BitXor,
    Comma,
    LParens,
    RParens,
    BlockStart,
    BlockEnd,
    IfStart,
    ElseStart,
    ElseIfStart,
    EndIf,
    VarDecl,
    LineEnd,
    ModEnd,
    Any,
}

impl PartialEq for Op {
    fn eq(&self, other: &Op) -> bool {
        discriminant(self) == discriminant(other)
    }
}

#[derive(Clone, Debug)]
pub enum ValType {
    Bool(bool),
    Number(f64),
    KeyWord,
    Void,
}

impl PartialEq for ValType {
    fn eq(&self, other: &ValType) -> bool {
        match (self, other) {
            (&ValType::Number(ref x), &ValType::Number(ref y)) => *x as f32 == *y as f32,
            (&ValType::Bool(ref x), &ValType::Bool(ref y)) => *x == *y,
            (&ValType::Void, _) => false,
            _ => panic!("Cannot compare keywords."),
        }
    }
}

impl fmt::Display for ValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ValType::Number(x) => write!(f, "{}", x),
            &ValType::Bool(x) => write!(f, "{}", x),
            &ValType::Void => write!(f, "{}", "Void"),
            &ValType::KeyWord => write!(f, "{}", "Keyword"),
        }
    }
}

impl Add for ValType {
    type Output = ValType;

    fn add(self, other: ValType) -> Self::Output {
        match (&self, &other) {
            (&ValType::Number(ref x), &ValType::Number(ref y)) => ValType::Number(x + y),
            _ => panic!("Invalid operation!\n"),
        }
    }
}

impl Sub for ValType {
    type Output = ValType;

    fn sub(self, other: ValType) -> Self::Output {
        match (&self, &other) {
            (&ValType::Number(ref x), &ValType::Number(ref y)) => ValType::Number(x - y),
            _ => panic!("Invalid operation!\n"),
        }
    }
}

impl Mul for ValType {
    type Output = ValType;

    fn mul(self, other: ValType) -> Self::Output {
        match (&self, &other) {
            (&ValType::Number(ref x), &ValType::Number(ref y)) => ValType::Number(x * y),
            _ => panic!("Invalid operation!\n"),
        }
    }
}

impl Div for ValType {
    type Output = ValType;

    fn div(self, other: ValType) -> Self::Output {
        match (&self, &other) {
            (&ValType::Number(ref x), &ValType::Number(ref y)) => if *y != 0. {
                ValType::Number(x / y)
            } else {
                panic!("Divide by zero!")
            },
            _ => panic!("Invalid operation!\n"),
        }
    }
}

impl Rem for ValType {
    type Output = ValType;

    fn rem(self, other: ValType) -> Self::Output {
        match (&self, &other) {
            (&ValType::Number(ref x), &ValType::Number(ref y)) => if *y != 0. {
                ValType::Number(x % y)
            } else {
                panic!("Divide by zero!")
            },
            _ => panic!("Invalid operation!\n"),
        }
    }
}

impl Eq for ValType {}

impl From<ValType> for f64 {
    fn from(val: ValType) -> f64 {
        match val {
            ValType::Number(x) => x,
            _ => panic!("INVALID!"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Token {
    Number(f64),
    Operator(Op),
    UnaryFn(fn(f64) -> f64),
    BinaryFn(fn(f64, f64) -> f64),
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
    curr: Token,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            base: 10,
            curr: Token::None,
            pos: 0,
        }
    }

    pub fn set_base(&mut self, base: u32) {
        self.base = base;
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.curr = Token::None;
    }

    fn peek(&self, input: &[u8]) -> char {
        input[self.pos + 1] as char
    }

    fn peek_back(&self, input: &[u8]) -> char {
        let mut pb = self.pos;
        if pb > 0 {
            while pb > 0 && input[pb] as char == ' ' {
                pb -= 1;
            }
            input[pb] as char
        } else {
            input[pb] as char
        }
    }

    pub fn eval(&mut self, input: &[u8]) -> Node {
        self.reset();
        self.get_next(input);
        self.statement(input)
    }

    fn get_number(&mut self, input: &[u8]) -> f64 {
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

    fn get_var(&mut self, input: &[u8]) -> String {
        let mut n = String::new();
        while self.pos < input.len()
            && ((input[self.pos] as char).is_alphanumeric() || input[self.pos] as char == '_')
        {
            n.push(input[self.pos] as char);
            self.pos += 1;
        }

        n
    }

    pub fn get_next(&mut self, input: &[u8]) -> Token {
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
                    panic!("Reached end of input!");
                }
                ch = input[self.pos] as char;
            }
        }

        if ch.is_digit(self.base) || ch == '.' {
            self.curr = Token::Number(self.get_number(input));
        } else if ch == '_' || ch.is_alphabetic() {
            self.curr = Token::Var(self.get_var(input));
        } else {
            self.curr = match ch {
                '+' => match self.curr.clone() {
                    Token::Number(_) | Token::Var(_) => {
                        if self.peek(input) == '=' {
                            self.pos += 1;
                            Token::Operator(Op::As_Add)
                        } else {
                            Token::Operator(Op::Add)
                        }
                    }
                    _ => Token::Operator(Op::Pos),
                },
                '-' => match self.curr.clone() {
                    Token::Number(_) | Token::Var(_) => {
                        if self.peek(input) == '=' {
                            self.pos += 1;
                            Token::Operator(Op::As_Sub)
                        } else {
                            Token::Operator(Op::Sub)
                        }
                    }
                    _ => Token::Operator(Op::Neg),
                },
                '*' => if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::As_Mul)
                } else if self.peek(input) == '*' {
                    self.pos += 1;
                    Token::Operator(Op::Pow)
                } else {
                    Token::Operator(Op::Mul)
                },
                '/' => if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::As_Div)
                } else {
                    self.pos += 1;
                    Token::Operator(Op::Div)
                },
                '^' => if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::As_BitXor)
                } else {
                    Token::Operator(Op::BitXor)
                },
                '&' => if self.peek(input) == '&' {
                    self.pos += 1;
                    Token::Operator(Op::And)
                } else if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::As_BitAnd)
                } else {
                    Token::Operator(Op::BitAnd)
                },

                '|' => if self.peek(input) == '|' {
                    self.pos += 1;
                    Token::Operator(Op::Or_)
                } else if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::As_BitOr)
                } else {
                    Token::Operator(Op::BitOr)
                },

                '%' => if self.peek(input) == '=' {
                    Token::Operator(Op::As_Mod)
                } else {
                    Token::Operator(Op::Mod)
                },
                '(' => Token::Operator(Op::LParens),
                ')' => Token::Operator(Op::RParens),
                '=' => if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::Eq_)
                } else {
                    Token::Operator(Op::Assign)
                },
                '>' => if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::Geq)
                } else {
                    Token::Operator(Op::Gt_)
                },
                '<' => if self.peek(input) == '=' {
                    self.pos += 1;
                    Token::Operator(Op::Leq)
                } else {
                    Token::Operator(Op::Lt_)
                },

                '\n' | ';' => Token::Operator(Op::LineEnd),
                '{' => Token::Operator(Op::BlockStart),
                '}' => Token::Operator(Op::BlockEnd),

                _ => Token::None,
            };
            self.pos += 1;
        }

        self.curr.clone()
    }

    fn eat(&mut self, token: Token, input: &[u8]) {
        if self.curr == token {
            //println!("Eat: {:?}", self.curr);
            self.curr = self.get_next(input);
        } else {
            panic!(
                "Parse error! Expected {:?}, got {:?} at position {}!",
                token, self.curr, self.pos
            );
        }
    }

    fn number(&mut self, input: &[u8]) -> Node {
        let m = Node::make_node(&self.curr.clone());
        self.eat(Token::Number(0.), input);
        m
    }

    fn id(&mut self, input: &[u8]) -> Node {
        let m = Node::new().val(&self.curr);
        self.eat(m.get_val().clone(), input);
        m
    }

    fn factor(&mut self, input: &[u8]) -> Node {
        let mut t = Node::make_node(&self.curr.clone());

        match *t.get_val() {
            Token::Number(_) => t = self.number(input),
            Token::Var(_) => t = self.id(input),
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
                t = Node::new().val(t.get_val()).add_child(&self.factor(input));
            }
            Token::Operator(Op::Neg) => {
                self.eat(Token::Operator(Op::Neg), input);
                t = Node::new().val(t.get_val()).add_child(&self.factor(input));
            }
            _ => t = t.val(&Token::None),
        };
        t
    }

    fn pow_factor(&mut self, input: &[u8]) -> Node {
        let mut res = self.factor(input);

        while match self.curr {
            Token::Operator(Op::Pow) => true,
            _ => false,
        } {
            let ser = res.clone();
            res = res.add_child(&ser)
                .val(&self.curr.clone())
                .add_child(&self.factor(input));
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
            res = res.add_child(&ser)
                .val(&tok)
                .add_child(&self.pow_factor(input));
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
            res = res.add_child(&ser).val(&tok).add_child(&self.term(input));
        }

        res
    }

    fn module(&mut self, input: &[u8]) -> Node {
        let mut n = Node::make_node(&self.curr.clone());
        self.eat(Token::Var(String::default()), input);
        //self.eat(Token::Operator(Op::LParens), input);
        n = n.add_child(&self.compound_statement(input));
        n
    }

    fn args(&mut self, input: &[u8]) -> Node {
        Node::new()
    }

    fn compound_statement(&mut self, input: &[u8]) -> Node {
        self.eat(Token::Operator(Op::BlockStart), input);
        let mut n = self.statement_list(input);
        self.eat(Token::Operator(Op::BlockEnd), input);
        n
    }

    fn statement_list(&mut self, input: &[u8]) -> Node {
        let mut n = self.statement(input);
        let mut r = Node::new().add_child(&n);

        while match self.curr {
            Token::Operator(Op::LineEnd) => true,
            _ => false,
        } {
            self.eat(Token::Operator(Op::LineEnd), input);
            r = r.add_child(&self.statement(input));
        }

        if match self.curr {
            Token::Operator(Op::BlockEnd) => true,
            _ => false,
        } {
            ;
        } else {
            panic!(
                "Expected {:?}, got {:?} at position {}!",
                Token::Operator(Op::BlockEnd),
                self.curr,
                self.pos
            );
        }

        r
    }

    fn statement(&mut self, input: &[u8]) -> Node {
        let mut res = Node::new();

        res = match self.curr {
            Token::Operator(Op::BlockStart) => self.compound_statement(input),
            Token::Var(_) => self.assign_statement(input),
            Token::Operator(Op::LParens) => self.expr(input),
            Token::Operator(Op::LineEnd) => match self.peek_back(input) {
                '{' => return res,
                _ => self.empty(input),
            },
            _ => return res,
        };

        res
    }

    fn assign_statement(&mut self, input: &[u8]) -> Node {
        let mut res = self.id(input);

        match self.curr {
            Token::Operator(Op::Assign) => self.eat(Token::Operator(Op::Assign), input),
            _ => panic!(
                "Expected {:?}, got {:?} at position {}!",
                Token::Operator(Op::Assign),
                self.curr,
                self.pos
            ),
        };

        let val = self.expr(input);

        Node::new()
            .val(&Token::Operator(Op::Assign))
            .add_child(&res)
            .add_child(&val)
    }

    fn empty(&mut self, input: &[u8]) -> Node {
        Node::make_node(&Token::None)
    }
}

/* 

    module: module_name LPARENS args RPARENS compound_statement

    module_name: VARIABLE

    compound_statement: BLOCK_START statement_list [return VARIABLE] BLOCK_END

    statement_list: statement | statement LINE_END statement_list

    statement: expr | mod_call | compound_statement | assign_statement | empty

    assign_statement: VARIABLE ASSIGN expr

    mod_call: module_name LPARENS args RPARENS

    args: empty | arg | arg SEPARATOR args

    arg: VARIABLE

    empty: 

    expr: term ((ADD | SUB) term)*

    term: pow_factor ((MUL | DIV | MOD) pow_factor)*

    pow_factor: factor ((POW) factor)*

    factor: NUMBER | VARIABLE | mod_call | LPARENS expr RPARENS | POW pow_factor | (POS | NEG) factor
*/
