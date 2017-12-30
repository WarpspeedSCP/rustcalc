extern crate std;

use std::mem::discriminant;
use std::str::Chars;

use itertools::PeekingNext;
use itertools;
use itertools::Itertools;

#[derive(Clone, Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq_,
    Gt_,
    Lt_,
    Geq,
    Leq,
    Neq,
    Any,
}

#[derive(Clone, Debug)]
enum Token {
    Number(u32),
    Operator(Op),
    UnaryFn(fn(u32) -> u32),
    BinaryFn(fn(u32, u32) -> u32, u64),
    Var(String),
    None,
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        if discriminant(self) == discriminant(other) {
            true
        } else {
            false
        }
    }
}

pub struct Parser {
    pos: usize,
    base: u32,
    ans: Token,
}

impl<'a> Parser {
    pub fn new() -> Parser {
        Parser {
            base: 10,
            ans: Token::None,
            pos: 0,
        }
    }

    pub fn set_base(&mut self, base: u32) {
        self.base = base;
    }

    fn get_number(&mut self, input: &[u8]) -> u32 {
        let mut num: String = "".into();

        while self.pos < input.len() && (input[self.pos] as char).is_digit(self.base) {
            num.push(input[self.pos] as char);
            self.pos += 1;
        }

        num.parse().unwrap()
    }

    fn get_next(&mut self, input: &[u8]) {
        if !(self.pos < input.len()) {
            self.ans = Token::None;
            return;
        }

        let mut ch = input[self.pos] as char;

        if ch == ' ' {
            while ch == ' ' {
                self.pos += 1;
                ch = input[self.pos] as char;
            }
        }

        self.ans = if ch.is_digit(self.base) {
            Token::Number(self.get_number(input))
        } else {
            //if !ch.is_alphanumeric() {
            self.pos += 1;
            match ch {
                '+' => Token::Operator(Op::Add),
                '-' => Token::Operator(Op::Sub),
                '*' => Token::Operator(Op::Mul),
                '/' => Token::Operator(Op::Div),
                _ => return,
            }
        }
    }

    fn eat(&mut self, token: Token, input: &[u8]) {
        if self.ans == token {
            self.get_next(input);
        } else {
            panic!(
                "Parse error! Expected {:?}, got {:?} at position {}!",
                token, self.ans, self.pos
            );
        }
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.ans = Token::None;
    }

    pub fn expr(&mut self, input: &'a str) -> u32 {
        self.reset();

        self.eat(Token::None, input.as_bytes());

        let left = self.ans.clone();
        self.eat(Token::Number(0), input.as_bytes());

        let op = self.ans.clone();
        self.eat(Token::Operator(Op::Any), input.as_bytes());

        let right = self.ans.clone();
        self.eat(Token::Number(0), input.as_bytes());

        match (left, op, right) {
            (Token::Number(x), Token::Operator(Op::Add), Token::Number(y)) => x + y,
            (Token::Number(x), Token::Operator(Op::Sub), Token::Number(y)) => x - y,
            (Token::Number(x), Token::Operator(Op::Mul), Token::Number(y)) => x * y,
            (Token::Number(x), Token::Operator(Op::Div), Token::Number(y)) => x / y,
            _ => 0,
        }
    }
}
