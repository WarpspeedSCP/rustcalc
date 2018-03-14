extern crate std;

use ast::Node;
use ast::NodeType;

use std::fmt;

// For distinguishing enum types.
use std::mem::discriminant;

// Operator overloads.
use std::ops::*;

use std::f64;
use std::collections::HashMap;

pub type SymTable = HashMap<String, Token>;

lazy_static! {
    pub static ref KEYWORD_TABLE: SymTable = {
        let mut d: SymTable = SymTable::new();

        let m = ["state",
                 "if",
                 "else",
                 "elif",
                 "return",
                 "write",
                 "read",
                 "for",
                 "in",
                 "array"];

        for i in 0..m.len() {
            d.insert(m[i].to_owned(), Token::KeyWord(m[i].to_owned()));
        }

        d
    };
}

// Enum of operator IDs recognised by the parser.
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

// Enum of token types recognised by the parser.
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
    KeyWord(String),
    None,
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        match (self, other) {
            (&Token::Number(_), &Token::Number(_)) => true,
            (&Token::Bool(x), &Token::Bool(y)) => x == y,
            (&Token::Operator(Op::Any), &Token::Operator(_)) => true,
            (&Token::Operator(ref x), &Token::Operator(ref y)) => x == y,
            (&Token::Var(ref x), &Token::Var(ref y)) => x == y,
            (&Token::Other(_), &Token::Other(_)) => false,
            (&Token::KeyWord(ref x), &Token::KeyWord(ref y)) => x == y,
            (&Token::None, _) => false,
            _ => discriminant(&self) == discriminant(&other),
        }
    }
}

impl Eq for Token {}

// Wrapper for the Token enum, which adds a position variable for easy debugging.
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
            // TODO: Implement code for variables.
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

// Conversions from the Token type to long & double.

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

// Lexer implementation.
pub struct Lexer {
    // The current position for the given input.
    pos: usize,

    // Base for parsing numbers.
    base: u32,

    // THe current token + metadata.
    curr: TokStruct,

    // An easy way to store a string.
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

    // Peek forward by one character.
    fn peek(&self) -> char {
        self.input[self.pos + 1] as char
    }

    // Peek backward to get the previous non-whitespace character.
    fn peek_back(&self) -> char {
        let mut pb = self.pos - 1;
        if pb > 0 {
            while pb > 0 && self.input[pb] as char == ' ' {
                pb -= 1;
            }
            self.input[pb] as char
        } else {
            self.input[pb] as char
        }
    }

    // Parses numbers. Returns a TokStruct with a Token::Number and its position in the string.
    fn get_number(&mut self) -> TokStruct {
        let mut num: String = "".into();

        // Manual bounds checking since iterators aren't as versatile as I'd want them to be.
        while self.pos < self.input.len()
            // Make sure current character is a digit or a '.'
            && ((self.input[self.pos] as char).is_digit(self.base)
                || self.input[self.pos] as char == '.')
        {
            num.push(self.input[self.pos] as char);
            self.pos += 1;
        }

        // If we have a valid number at the end of this, we can return.
        match num.parse() {
            Ok(val) => TokStruct::new(Token::Number(val), self.pos - num.len()),
            Err(_) => panic!(
                "Lexer error at position {}! {} is not a number!",
                self.pos - num.len(),
                num
            ),
        }
    }

    // Parses variables. A variable identifier is composed of digits,
    //   letters and underscores, and can start with either a letter or an underscore.
    //   This parser is case sensitive so 'a' is different from 'A'.
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

    // Parses boolean values. A boolean value is either 'true' or 'false'.
    fn get_bool(&mut self) -> TokStruct {
        // We go through get_var() to grab strings from input.
        let d = self.get_var();

        // We need to use a match here to destructure the underlying Token
        //   even though we know it's going to be a Token::Var because that's
        //   the only way to get at the underlying data of an enum variable.
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

    // The main interface of the lexer. It advances token by token, and outputs a single token for each non-whitespace character it reads from input.
    pub fn get_next(&mut self) -> TokStruct {
        // Bounds check & initialisation of the current token.
        if !(self.pos < self.input.len()) {
            self.curr = TokStruct::new(Token::None, self.pos);
            return self.curr.clone();
        }

        let mut ch = self.input[self.pos] as char;

        // This block skips any whitespace.
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

        // The real parsing gets done at these if-else statements.

        //  Number
        if ch.is_digit(self.base) || ch == '.' {
            self.curr = self.get_number();

        // Identifier
        } else if ch == '_' || ch.is_alphabetic() {
            self.curr = self.get_var();

            match self.curr.get_val() {
                Token::Var(ref x) => if KEYWORD_TABLE.contains_key(x) {
                    self.curr.val = KEYWORD_TABLE.get(x).unwrap().clone();
                },
                _ => panic!("This should never happen."),
            }

        // Symbol
        // TODO: Add support for shorthand operators such as '+='
        } else {
            self.curr = TokStruct::new(
                match ch {
                    // If the previous token is a number or identifier, we know it's a binary operator.
                    // If the previous token is an operator, we can consider this token to be a unary operator.
                    '+' => match self.curr.get_val() {
                        Token::Number(_) | Token::Var(_) => Token::Operator(Op::Add),
                        _ => Token::Operator(Op::Pos),
                    },

                    // Same.
                    '-' => match self.curr.get_val() {
                        Token::Number(_) | Token::Var(_) => Token::Operator(Op::Sub),
                        _ => Token::Operator(Op::Neg),
                    },

                    // The peek forward method is used here to differentiate Op::Mul and Op::Pow.
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

                    // These operators are for symbols such as line and block delimiters.
                    '\n' | ';' => Token::Operator(Op::LineEnd),
                    '{' => Token::Operator(Op::BlockStart),
                    '}' => Token::Operator(Op::BlockEnd),

                    // We don't recognise the symbol.
                    _ => panic!(
                        "get_next: Expected an operator, but got \"{}\" at position {}!",
                        ch, self.pos
                    ),
                },
                self.pos,
            );
            self.pos += 1;
        }

        // Self.curr has been updated by the match block.
        TokStruct::new(self.curr.get_val(), self.pos)
    }

    // Returns a Vector containing all the tokens in the input.
    pub fn get_all(&mut self) -> Vec<Token> {
        let mut toks: Vec<Token> = Vec::new();

        while self.input[self.pos] as char != '\0' {
            toks.push(self.get_next().get_val());
        }
        toks
    }

    // Validates the current token against a provided value and,
    //   on successful validation, updates the current token and returns the previous token.
    pub fn eat(&mut self, token: Token) -> TokStruct {
        let t = self.curr.clone();
        if discriminant(&t.get_val()) == discriminant(&token) {
            self.curr = self.get_next();
        } else {
            panic!(
                "eat: Expected {:?}, but got {:?} at position {}!",
                token,
                t.get_val(),
                self.pos
            );
        }

        t
    }
}

// The Parser implementation.
pub struct Parser {
    lexer: Lexer,

    // Might be better to remove this.
    input: String,
}

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

    // Incomplete temporary eval function to test parse tree generation.
    pub fn eval(&mut self) -> Node {
        // We need to reset the lexer before we parse anything.
        self.lexer.reset();
        /*
        if self.input.find(|c: char| c == '>' || c == '<' || c == '!') == None {
            self.expr()
        } else {
            self.bool_expr()
        }
        */

        self.statement()
    }

    // Terminal function to accept a number.
    fn number(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Number(0.)))
    }

    // Terminal function to accept an identifier.
    fn id(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Var(String::default())))
    }

    // Terminal function to accept a boolean value.
    fn boolean(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Bool(true))).type_(NodeType::BExpression)
    }

    // Non terminal function to accept a factor.
    // A factor is  defined by the rule -
    //
    // factor: NUMBER | VARIABLE | fn_call | LPARENS expr RPARENS | POW pow_factor | (POS | NEG) factor
    pub fn factor(&mut self) -> Node {
        let mut t = Node::new();
        let m = self.get_curr();

        match m.get_val() {
            Token::Number(_) => t = self.number().type_(NodeType::AExpression),
            Token::Var(_) => t = self.id().type_(NodeType::AExpression),

            // If we encounter a '(' character, We interpret it as a subexpression.
            Token::Operator(Op::LParens) => {
                self.lexer.eat(m.get_val());
                t = self.expr();
                self.lexer.eat(Token::Operator(Op::RParens));
            }
            Token::Operator(Op::Pow) => {
                // The power operator is right associative.
                //
                // We pass the power operator from pow_factor() without consuming it,
                //   then consume the operator and parse a nested power factor. If
                //   the nested expression is again an expression of the form 'a ** b',
                //   it is evaluated again by passing through pow_factor() and this function
                //   until there are no power factors left. The resulting AST is structured
                //   so that all power functions appear on the right subtree of parent nodes.
                self.lexer.eat(m.get_val());
                t = self.pow_factor();
            }
            Token::Operator(Op::Pos) => {
                // We first "eat" a '+', then parse what comes after as a factor
                //   (since it could be a subexpression as well).
                t = Node::new()
                    .val(self.lexer.eat(m.get_val()))
                    .add_child(self.factor())
            }
            Token::Operator(Op::Neg) => {
                t = Node::new()
                    .val(self.lexer.eat(m.get_val()))
                    .add_child(self.factor())
            }
            // To prevent errors when we get an empty input sequence.
            Token::None => t = t.val(TokStruct::new(Token::None, 0)),
            _ => panic!(
                "factor: Expected Number or Variable at position {}, got {}!",
                m.get_pos(),
                m.get_val()
            ),
        }

        t
    }

    // A non terminal function to parse expressions of the form 'a ** b', which I call a power factor.
    // Both 'a' and 'b' are factors.
    // It can be applied on multilevelled expressions such as 'a ** (2 * b ** 4)' as well, through a recursion loop.
    //
    // A power factor is represented by the rule-
    //
    // pow_factor: factor (POW factor)
    //
    // If this function encounters a single factor alone, it directly passes on the result of factor().
    // If an expression like 'a ** b' is encountered, it passes on the expression without consuming the power operator.
    // The power operator is consumed by the factor() function.
    pub fn pow_factor(&mut self) -> Node {
        // The first factor is guaranteed to exist.
        let t = self.factor().type_(NodeType::AExpression);
        let m = self.get_curr();

        match m.get_val() {
            // If we find Op::Pow, we can continue to get the next power factor.
            Token::Operator(Op::Pow) => Node::new().val(m).add_child(t).add_child(self.factor()),
            _ => t,
        }
    }

    // A non terminal function to parse expressions of the form 'a * b', 'a / b', 'a // b' or 'a % b'
    //   where both 'a' and 'b' are power factors.
    // A term is composed of 1 or more power factors and is defined by the rule-
    //
    // term: pow_factor ((MUL | DIV | MOD | INTDIV) pow_factor)*

    pub fn term(&mut self) -> Node {
        let mut t = self.pow_factor().type_(NodeType::AExpression);
        let mut m = self.get_curr();

        // Keep looping while the current operator is any of these.
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

    // A non terminal function to parse expressions of the form 'a + b' or 'a - b'.
    // It functions similarly to term().
    //
    // An expression is represented by the rule-
    //
    // expr: term ((ADD | SUB) term)*

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

    fn function(&mut self) -> Node {
        Node::new()
    }

    fn statement_list(&mut self) -> Vec<Node> {
        let mut t = vec![self.statement()];

        while match self.get_curr().get_val() {
            Token::Operator(Op::BlockEnd) => return t,
            _ => true,
        } {
            t.push(self.statement());
        }

        t
    }

    fn statement(&mut self) -> Node {
        let mut t = Node::new();

        match self.lexer.get_curr().get_val() {
            Token::Operator(Op::BlockStart) => {
                self.lexer.eat(Token::Operator(Op::BlockStart));
                t.children(self.statement_list()).type_(NodeType::Block);
                self.lexer.eat(Token::Operator(Op::BlockEnd));
            }
            Token::Var(_) => t = self.assign_function_disambiguate(),
            Token::KeyWord(x) => if x == "if".to_owned() {
                t.children(self.conditional_statement())
                    .type_(NodeType::Cond);
            },
            Token::Operator(Op::LParens) => t = self.expr(),
            Token::Operator(Op::LineEnd) => {
                t = Node::new().val(self.lexer.eat(Token::Operator(Op::LineEnd)))
            }
            Token::Operator(Op::BlockEnd) => if self.lexer.peek_back() == '{' {
                t = Node::new().type_(NodeType::Block);
            },
            _ => panic!(
                "Did not expect {} at position {}",
                self.get_curr().get_val(),
                self.get_curr().get_pos()
            ),
        };

        t
    }

    fn assign_function_disambiguate(&mut self) -> Node {
        let inputstr = Vec::from(self.input.clone().as_bytes());
        for i in self.lexer.get_pos()..inputstr.len() {
            if inputstr[i] as char == ';' || inputstr[i] as char == '\n' {
                return self.fn_call();
            } else if inputstr[i] as char == '=' && inputstr[i + 1] as char != '=' {
                return self.assign_statement();
            }
        }
        self.fn_call()
    }

    fn assign_statement(&mut self) -> Node {
        Node::new().val(TokStruct::new(Token::Other('8'), 434))
    }

    fn return_statement(&mut self) {}

    fn conditional_statement(&mut self) -> Vec<Node> {
        let mut t = vec![
            Node::new()
                .val(self.lexer.eat(KEYWORD_TABLE[&"if".to_owned()].clone()))
                .type_(NodeType::Cond),
        ];

        t.push(self.bool_expr());
        t.push(self.statement());

        if match self.get_curr().get_val() {
            Token::KeyWord(x) => x == "else".to_owned(),
            _ => false,
        } {
            t.push(
                Node::new()
                    .val(self.lexer.eat(KEYWORD_TABLE[&"else".to_owned()].clone()))
                    .type_(NodeType::Cond),
            );

            t.push(self.statement());
        }

        t
    }

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
        let t: Node;
        let m = self.lexer.get_curr();

        match m.get_val() {
            Token::Number(_) => t = self.expr().type_(NodeType::BExpression),
            Token::Var(_) => {
                let d = self.input.as_bytes()[self.lexer.get_pos()] as char;
                if d == '(' {
                    t = self.fn_call().type_(NodeType::BExpression);
                } else {
                    t = self.expr().type_(NodeType::BExpression);
                }
            }
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

    fn fn_call(&mut self) -> Node {
        self.lexer.eat(Token::Var("".to_owned()));
        self.lexer.eat(Token::Operator(Op::LParens));
        self.lexer.eat(Token::Operator(Op::RParens));
        Node::new().val(TokStruct::new(Token::Var("t".to_owned()), 0))
    }
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

    bool_factor: (NOT)? BOOL | VARIABLE | bool_expr | fn_call | relational_expr

    relational_expr: relational_factor ((LT | GT | LEQ | GEQ | EQ | NEQ) relational_factor)*

    relational_factor: VARIABLE | NUMBER | BOOL

    expr: term ((ADD | SUB) term)*

    term: pow_factor ((MUL | DIV | MOD) pow_factor)*

    pow_factor: factor (POW factor)

    factor: NUMBER | VARIABLE | fn_call | LPARENS expr RPARENS | POW pow_factor | (POS | NEG) factor
*/
