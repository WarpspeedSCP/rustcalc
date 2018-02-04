extern crate std;

use ast::Node;
use ast::NodeType;

use std::fmt;
use std::mem::discriminant;
use std::ops::*;

#[derive(Clone, Debug, Eq, Hash)]
#[repr(C)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
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
    BlockStart,
    BlockEnd,
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
#[repr(C)]
pub enum Token {
    Number(f64),
    Bool(bool),
    Operator(Op),
    Var(String),
    Other(char),
    Good,
    Bad,
    KeyWord,
    None,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct TokStruct {
    val: Token,
    pos: usize,
}

impl TokStruct {
    pub fn new(t: Token, p: usize) -> TokStruct {
        TokStruct { val: t, pos: p }
    }

    pub fn get_val(&self) -> Token {
        self.val.clone()
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        match (self, other) {
            (&Token::Operator(Op::Any), &Token::Operator(_)) => true,
            (&Token::Operator(ref x), &Token::Operator(ref y)) => x == y,
            _ => discriminant(self) == discriminant(other),
        }
    }
}

impl Add for Token {
    type Output = Token;

    fn add(self, other: Token) -> Token {
        match (self, other) {
            (Token::Number(ref x), Token::Number(ref y)) => Token::Number(x + y),
            (Token::Var(ref x), Token::Var(ref y)) => Token::None,
            _ => Token::None,
        }
    }
}

impl Sub for Token {
    type Output = Token;

    fn sub(self, other: Token) -> Token {
        match (self, other) {
            (Token::Number(ref x), Token::Number(ref y)) => Token::Number(x - y),
            (Token::Var(ref x), Token::Var(ref y)) => Token::None,
            _ => Token::None,
        }
    }
}

impl Mul for Token {
    type Output = Token;

    fn mul(self, other: Token) -> Token {
        match (self, other) {
            (Token::Number(ref x), Token::Number(ref y)) => Token::Number(x * y),
            (Token::Var(ref x), Token::Var(ref y)) => Token::None,
            _ => Token::None,
        }
    }
}

impl Div for Token {
    type Output = Token;

    fn div(self, other: Token) -> Token {
        match (self, other) {
            (Token::Number(ref x), Token::Number(ref y)) => if *y != 0f64 {
                Token::Number(x / y)
            } else {
                Token::Bad
            },
            (Token::Var(ref x), Token::Var(ref y)) => Token::None,
            _ => Token::None,
        }
    }
}

impl Rem for Token {
    type Output = Token;

    fn rem(self, other: Token) -> Token {
        match (self, other) {
            (Token::Number(ref x), Token::Number(ref y)) => if *y != 0f64 {
                Token::Number((*x as i64 % *y as i64) as f64)
            } else {
                Token::Bad
            },
            (Token::Var(ref x), Token::Var(ref y)) => Token::None,
            _ => Token::None,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Token::Number(ref x) => write!(f, "{}", x),
            &Token::Var(ref x) => write!(f, "{}", x),
            &Token::Operator(ref x) => write!(f, "{:?}", x),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl From<Token> for f64 {
    fn from(tok: Token) -> f64 {
        match tok {
            Token::Number(x) => x,
            _ => 0f64,
        }
    }
}

impl From<Token> for i64 {
    fn from(tok: Token) -> i64 {
        match tok {
            Token::Number(x) => x as i64,
            _ => 0i64,
        }
    }
}

pub struct Lexer {
    pos: usize,
    base: u32,
    curr: TokStruct,
    input: Box<[u8]>,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            pos: 0,
            base: 10,
            curr: TokStruct::new(Token::None, 0),
            input: Box::from([0u8]),
        }
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    pub fn get_base(&self) -> u32 {
        self.base
    }

    pub fn get_curr(&self) -> TokStruct {
        self.curr.clone()
    }

    pub fn base(&mut self, base: u32) -> &mut Lexer {
        self.base = base;
        self
    }

    pub fn input(&mut self, input: String) -> &mut Lexer {
        self.input = Box::from(input.into_bytes());
        self
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.curr = self.get_next();
    }

    fn peek(&self) -> char {
        self.input[self.pos + 1] as char
    }

    fn peek_back(&self) -> char {
        let mut pb = self.pos;
        if pb > 0 {
            while pb > 0 && self.input[pb] as char == ' ' {
                pb -= 1;
            }
            self.input[pb] as char
        } else {
            self.input[pb] as char
        }
    }

    fn get_number(&mut self) -> TokStruct {
        let mut num: String = "".into();

        while self.pos < self.input.len()
            && ((self.input[self.pos] as char).is_digit(self.base)
                || self.input[self.pos] as char == '.')
        {
            num.push(self.input[self.pos] as char);
            self.pos += 1;
        }

        match num.parse() {
            Ok(val) => TokStruct::new(Token::Number(val), self.pos - num.len()),
            Err(_) => panic!(
                "Lexer error at position {}! {} is not a number!",
                self.pos - num.len(),
                num
            ),
        }
    }

    fn get_var(&mut self) -> TokStruct {
        let mut n = String::new();
        while self.pos < self.input.len()
            && ((self.input[self.pos] as char).is_alphanumeric()
                || self.input[self.pos] as char == '_')
        {
            n.push(self.input[self.pos] as char);
            self.pos += 1;
        }

        TokStruct::new(Token::Var(n.clone()), self.pos - n.len())
    }

    fn get_bool(&mut self) -> TokStruct {
        let d = self.get_var();
        match d.get_val() {
            Token::Var(x) => match x.as_str() {
                "true" => TokStruct::new(Token::Bool(true), self.pos - 4),
                "false" => TokStruct::new(Token::Bool(false), self.pos - 5),
                _ => panic!(
                    "Expected boolean value at position {}, got {}!\n",
                    d.get_pos(),
                    d.get_val()
                ),
            },
            _ => panic!("Expected boolean value at position {}!\n", d.get_pos()),
        }
    }

    pub fn get_next(&mut self) -> TokStruct {
        if !(self.pos < self.input.len()) {
            self.curr = TokStruct::new(Token::None, self.pos);
            return self.curr.clone();
        }

        let mut ch = self.input[self.pos] as char;

        if ch == ' ' {
            while ch == ' ' {
                if self.pos < (self.input.len() - 1) {
                    self.pos += 1;
                } else {
                    return TokStruct::new(Token::None, 0);
                }
                ch = self.input[self.pos] as char;
            }
        }

        if ch.is_digit(self.base) || ch == '.' {
            self.curr = self.get_number();
        } else if ch == '_' || ch.is_alphabetic() {
            self.curr = self.get_var();
        } else {
            self.curr = TokStruct::new(
                match ch {
                    '+' => match self.curr.get_val() {
                        Token::Number(_) | Token::Var(_) => Token::Operator(Op::Add),
                        _ => Token::Operator(Op::Pos),
                    },
                    '-' => match self.curr.get_val() {
                        Token::Number(_) | Token::Var(_) => Token::Operator(Op::Sub),
                        _ => Token::Operator(Op::Neg),
                    },
                    '*' => if self.peek() == '*' {
                        self.pos += 1;
                        Token::Operator(Op::Pow)
                    } else {
                        Token::Operator(Op::Mul)
                    },
                    '/' => if self.peek() == '/' {
                        self.pos += 1;
                        Token::Operator(Op::IntDiv)
                    } else {
                        self.pos += 1;
                        Token::Operator(Op::Div)
                    },
                    '^' => Token::Operator(Op::BitXor),
                    '&' => if self.peek() == '&' {
                        self.pos += 1;
                        Token::Operator(Op::And)
                    } else {
                        Token::Operator(Op::BitAnd)
                    },

                    '|' => if self.peek() == '|' {
                        self.pos += 1;
                        Token::Operator(Op::Or_)
                    } else {
                        Token::Operator(Op::BitOr)
                    },
                    '!' => if self.peek() == '=' {
                        self.pos += 1;
                        Token::Operator(Op::Neq)
                    } else {
                        Token::Operator(Op::Not)
                    },
                    '%' => Token::Operator(Op::Mod),
                    '(' => Token::Operator(Op::LParens),
                    ')' => Token::Operator(Op::RParens),
                    '=' => if self.peek() == '=' {
                        self.pos += 1;
                        Token::Operator(Op::Eq_)
                    } else {
                        Token::Operator(Op::Assign)
                    },
                    '>' => if self.peek() == '=' {
                        self.pos += 1;
                        Token::Operator(Op::Geq)
                    } else {
                        Token::Operator(Op::Gt_)
                    },
                    '<' => if self.peek() == '=' {
                        self.pos += 1;
                        Token::Operator(Op::Leq)
                    } else {
                        Token::Operator(Op::Lt_)
                    },

                    '\n' | ';' => Token::Operator(Op::LineEnd),
                    '{' => Token::Operator(Op::BlockStart),
                    '}' => Token::Operator(Op::BlockEnd),

                    _ => panic!(
                        "get_next: Expected an operator, but got \"{}\" at position {}!",
                        ch, self.pos
                    ),
                },
                self.pos,
            );
            self.pos += 1;
        }

        TokStruct::new(self.curr.get_val(), self.pos)
    }

    pub fn get_all(&mut self) -> Vec<Token> {
        let mut toks: Vec<Token> = Vec::new();

        while self.input[self.pos] as char != '\0' {
            toks.push(self.get_next().get_val());
        }
        toks
    }

    pub fn eat(&mut self, token: Token) -> TokStruct {
        let t = self.curr.clone();
        if t.get_val() == token {
            self.curr = self.get_next();
        } else {
            panic!(
                "eat: Expeected {:?}, but got {:?} at position {}!",
                token,
                t.get_val(),
                self.pos
            );
        }

        t
    }
}

pub struct Parser {
    lexer: Lexer,
    input: String,
}
//a - 1
impl Parser {
    pub fn new() -> Parser {
        Parser {
            lexer: Lexer::new(),
            input: String::new(),
        }
    }

    pub fn input(&mut self, input: String) {
        self.input = input;
        self.lexer.input(self.input.clone());
    }

    pub fn base(&mut self, base: u32) {
        self.lexer.base(base);
    }

    pub fn eval(&mut self) -> Node {
        self.lexer.reset();

        if self.input.find(|c: char| c == '>' || c == '<' || c == '!') == None {
            self.expr()
        } else {
            self.bool_expr()
        }
    }

    fn number(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Number(0.)))
    }

    fn id(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Var(String::default())))
    }

    fn boolean(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Bool(true))).type_(NodeType::BExpression)
    }

    pub fn factor(&mut self) -> Node {
        let mut t = Node::new();
        let m = self.get_curr();

        match m.get_val() {
            Token::Number(_) => t = self.number().type_(NodeType::AExpression),
            Token::Var(_) => t = self.id().type_(NodeType::AExpression),
            Token::Operator(Op::LParens) => {
                self.lexer.eat(m.get_val());
                t = self.expr();
                self.lexer.eat(Token::Operator(Op::RParens));
            }
            Token::Operator(Op::Pow) => {
                self.lexer.eat(m.get_val());
                t = self.pow_factor();
            }
            Token::Operator(Op::Pos) => {
                t = Node::new()
                    .val(self.lexer.eat(m.get_val()))
                    .add_child(self.factor())
            }
            Token::Operator(Op::Neg) => {
                t = Node::new()
                    .val(self.lexer.eat(m.get_val()))
                    .add_child(self.factor())
            }
            Token::None => t = t.val(TokStruct::new(Token::None, 0)),
            _ => panic!(
                "factor: Expected Number or Variable at position {}, got {}!",
                m.get_pos(),
                m.get_val()
            ),
        }

        t
    }

    pub fn pow_factor(&mut self) -> Node {
        let mut t = self.factor().type_(NodeType::AExpression);
        let m = self.get_curr();

        match m.get_val() {
            Token::Operator(Op::Pow) => Node::new().val(m).add_child(t).add_child(self.factor()),
            _ => t,
        }
    }
    //pagalpanti2196
    pub fn term(&mut self) -> Node {
        let mut t = self.pow_factor().type_(NodeType::AExpression);
        let mut m = self.get_curr();

        while match m.get_val() {
            Token::Operator(Op::Mul)
            | Token::Operator(Op::Div)
            | Token::Operator(Op::Mod)
            | Token::Operator(Op::IntDiv) => true,
            _ => false,
        } {
            t = Node::make_node(self.lexer.eat(m.get_val()))
                .add_child(t)
                .add_child(self.pow_factor());
            m = self.get_curr();
        }

        t
    }

    pub fn expr(&mut self) -> Node {
        let mut t = self.term().type_(NodeType::AExpression);
        let mut m = self.get_curr();

        while match m.get_val() {
            Token::Operator(Op::Add) | Token::Operator(Op::Sub) => true,
            _ => false,
        } {
            t = Node::make_node(self.lexer.eat(m.get_val()))
                .add_child(t)
                .add_child(self.term())
                .type_(NodeType::AExpression);
            m = self.get_curr();
        }

        t
    }

    fn function(&mut self) {}

    fn scope(&mut self) {}

    fn statement_list(&mut self) {}

    fn statement(&mut self) -> Node {
        let mut t = Node::new();

        match self.lexer.get_curr().get_val() {
            Token::Operator(Op::BlockStart) => {
                self.scope();
                Node::new()
            }
            Token::Var(_) => self.assign_statement(),
            Token::Operator(Op::LParens) => self.expr(),
            _ => panic!(
                "Did not expect {} at position {}",
                self.get_curr().get_val(),
                self.get_curr().get_pos()
            ),
        }
    }

    fn assign_statement(&mut self) -> Node {
        Node::new()
    }

    fn return_statement(&mut self) {}

    fn conditional_statement(&mut self) {}

    fn get_curr(&self) -> TokStruct {
        self.lexer.get_curr()
    }

    //a == b

    pub fn bool_expr(&mut self) -> Node {
        let mut t = self.bool_term().type_(NodeType::BExpression);
        let mut m = self.get_curr();

        while match self.get_curr().get_val() {
            Token::Operator(Op::Or_) => true,
            _ => false,
        } {
            t = Node::new()
                .add_child(t)
                .val(self.lexer.eat(m.get_val()))
                .add_child(self.bool_term())
                .type_(NodeType::BExpression);
            m = self.get_curr();
        }

        t
    }

    pub fn bool_term(&mut self) -> Node {
        let mut t = self.bool_factor().type_(NodeType::BExpression);
        let mut m = self.get_curr();

        while match self.get_curr().get_val() {
            Token::Operator(Op::And) => true,
            _ => false,
        } {
            t = Node::new()
                .add_child(t)
                .val(self.lexer.eat(m.get_val()))
                .add_child(self.bool_factor())
                .type_(NodeType::BExpression);
            m = self.get_curr();
        }

        t
    }

    pub fn bool_factor(&mut self) -> Node {
        let mut t = Node::new();
        let m = self.get_curr();
        match self.get_curr().get_val() {
            Token::Bool(_) | Token::Var(_) | Token::Number(_) | Token::Operator(Op::LParens) => {
                t = self.relational_expr()
            }
            Token::Operator(Op::Not) => {
                t = Node::new()
                    .val(self.lexer.eat(m.get_val()))
                    .add_child(self.bool_factor())
            }
            _ => panic!(
                "bool_factor: Expected Number or Variable at position {}, got {}!",
                self.get_curr().get_pos(),
                self.get_curr().get_val()
            ),
        }

        t
    }

    fn relational_expr(&mut self) -> Node {
        let mut t = self.relational_factor();

        while match self.get_curr().get_val() {
            Token::Operator(Op::Eq_)
            | Token::Operator(Op::Neq)
            | Token::Operator(Op::Lt_)
            | Token::Operator(Op::Leq)
            | Token::Operator(Op::Gt_)
            | Token::Operator(Op::Geq) => true,
            _ => false,
        } {
            match self.get_curr().get_val() {
                Token::Operator(Op::Eq_) | Token::Operator(Op::Neq) => {
                    let m = self.get_curr();
                    t = Node::new()
                        .add_child(t)
                        .val(self.lexer.eat(m.get_val()))
                        .add_child(self.relational_factor())
                        .type_(NodeType::BExpression)
                }
                Token::Operator(Op::Lt_)
                | Token::Operator(Op::Leq)
                | Token::Operator(Op::Gt_)
                | Token::Operator(Op::Geq) => match t.get_val() {
                    Token::Number(_) | Token::Var(_) | Token::Operator(_) => {
                        let m = self.get_curr();
                        t = Node::new()
                            .add_child(t)
                            .val(self.lexer.eat(m.get_val()))
                            .add_child(self.relational_factor())
                            .type_(NodeType::BExpression)
                    }
                    _ => panic!(
                        "relational_expr: Expected Number or Variable at position {}, got {}!",
                        t.get_pos(),
                        t.get_val()
                    ),
                },
                Token::Operator(Op::And) | Token::Operator(Op::Or_) | Token::Operator(Op::Not) => {}
                Token::None => {}
                _ => panic!(
                    "relational_expr: Expected Relational operator at position {}, got {}!",
                    self.get_curr().get_pos(),
                    self.get_curr().get_val() /* !(a > (b - c)) */
                ),
            }
        }

        t
    }

    fn relational_factor(&mut self) -> Node {
        let mut t = Node::new();
        let m = self.lexer.get_curr();

        match m.get_val() {
            Token::Number(_) | Token::Var(_) => t = self.expr().type_(NodeType::BExpression),
            Token::Bool(_) => t = self.boolean(),
            Token::Operator(Op::Not) => {
                t = Node::new()
                    .val(self.lexer.eat(m.get_val()))
                    .add_child(self.relational_expr())
                    .type_(NodeType::BExpression)
            }
            Token::Operator(Op::LParens) => {
                self.lexer.eat(Token::Operator(Op::LParens));
                t = self.relational_expr();
                self.lexer.eat(Token::Operator(Op::RParens));
            }
            _ => panic!(
                "relational_factor: Expected Value type at position {}, got {}!",
                m.get_pos(),
                m.get_val()
            ),
        }
        t
    }

    fn fn_call(&mut self) {}
}

/* 

    function: function_name LPARENS args RPARENS scope

    function_name: VARIABLE

    scope: BLOCK_START statement_list BLOCK_END

    statement_list: statement LINE_END statement_list

    statement: expr | fn_call | scope | assign_statement | return_statement | conditional_statement

    conditional_statement: IF bool_expr scope (ELSE IF bool_expr scope)* (ELSE scope)?

    return_statement: RETURN VARIABLE

    assign_statement: VARIABLE ASSIGN (expr | bool_expr)

    fn_call: function_name LPARENS args RPARENS

    args: (arg SEPARATOR)*

    arg: VARIABLE

    bool_expr: bool_term (OR bool_term)*

    bool_term: bool_factor (AND bool_factor)*

    bool_factor: (NOT) BOOL | VARIABLE | bool_expr | fn_call | relational_expr

    relational_expr: relational_factor (LT | GT | LEQ | GEQ | EQ | NEQ) relational_factor

    relational_factor: VARIABLE | NUMBER | BOOL

    expr: term ((ADD | SUB) term)*

    term: pow_factor ((MUL | DIV | MOD) pow_factor)*

    pow_factor: factor (POW factor)

    factor: NUMBER | VARIABLE | fn_call | LPARENS expr RPARENS | POW pow_factor | (POS | NEG) factor
*/
