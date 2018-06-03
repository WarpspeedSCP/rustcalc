
use ast::Node;
use ast::NodeType;

use std::fmt;

// For distinguishing enum types.
use std::mem::discriminant;

// Operator overloads.
use std::ops::*;

use std::collections::HashMap;
use std::f64;

pub type SymTable = HashMap<String, Token>;

lazy_static! {
    pub static ref KEYWORD_TABLE: SymTable = {
        let mut d: SymTable = SymTable::new();

        let m = [
            "state", "if", "else", "elif", "return", "write", "read", "for", "in", "array", "fn"
        ];

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
    LineEnd,
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

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos
    }

    pub fn get_base(&self) -> u32 {
        self.base
    }

    pub fn get_curr(&self) -> TokStruct {
        self.curr.clone()
    }

    pub fn set_curr(&mut self, curr: TokStruct) {
        self.curr = curr
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

    fn peek_token(&mut self) -> TokStruct {
        let cc = self.get_curr();
        let cp = self.get_pos();

        // Get next token. This will advance the lexer forward by one token.
        let m = self.get_next();

        // We need to reset thr lexer's position so it doesn't miss a token.
        self.set_pos(cp);
        self.set_curr(cc);

        m
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
                        Token::Number(_) | Token::Var(_) | Token::Operator(Op::RParens) => {
                            Token::Operator(Op::Add)
                        }
                        _ => Token::Operator(Op::Pos),
                    },

                    // Same.
                    '-' => match self.curr.get_val() {
                        Token::Number(_) | Token::Var(_) | Token::Operator(Op::RParens) => {
                            Token::Operator(Op::Sub)
                        }
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
                    ',' => Token::Operator(Op::Comma),

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
    //   on successful validation, updates the current token and
    //   returns the previous token.
    pub fn eat(&mut self, token: Token) -> TokStruct {
        let t = self.curr.clone();
        if discriminant(&t.get_val()) == discriminant(&token) {
            self.curr = self.get_next();
        } else {
            panic!(
                "eat: Expected {}, but got {} at position {}!",
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

        self.program()
    }

    // Terminal function to accept a number.
    fn number(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Number(0.))).type_(NodeType::AExpression)
    }

    // Terminal function to accept an identifier.
    fn id(&mut self) -> Node {
        Node::make_node(self.lexer.eat(Token::Var(String::new())))
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
            Token::Number(_) => t = self.number(),
            Token::Var(_) => t = self.var_disambiguate(),

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
        let t = self.factor();
        let m = self.get_curr();

        match m.get_val() {
            // If we find Op::Pow, we can continue to get the next power factor.
            Token::Operator(Op::Pow) => Node::new()
                .val(m)
                .add_child(t)
                .add_child(self.factor())
                .type_(NodeType::AExpression),
            _ => t,
        }
    }

    // A non terminal function to parse expressions of the form 'a * b', 'a / b', 'a // b' or 'a % b'
    //   where both 'a' and 'b' are power factors.
    // A term is composed of 1 or more power factors and is defined by the rule-
    //
    // term: pow_factor ((MUL | DIV | MOD | INTDIV) pow_factor)*
    pub fn term(&mut self) -> Node {
        let mut t = self.pow_factor();
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
                .add_child(self.pow_factor())
                .type_(NodeType::AExpression);
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
        let mut t = self.term();
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

    // A non terminal function representing a comma separated arguement list.
    //   An arguement list must begin with a left parenthesis, and end with
    //   a right parenthesis. An arguement list can only appear within a
    //   function defenition or a function call.
    fn arg_list(&mut self) -> Node {
        self.lexer.eat(Token::Operator(Op::LParens));

        let mut m = self.get_curr();
        let mut t = Node::new();

        while match m.get_val() {
            Token::Operator(Op::RParens) => false,
            Token::Var(_) => {
                m = self.lexer.eat(Token::Var(String::new()));
                true
            }
            Token::Operator(Op::Comma) => {
                self.lexer.eat(Token::Operator(Op::Comma));
                let temp = self.lexer.peek_token();
                match temp.get_val() {
                    Token::Operator(Op::RParens) => {
                        m = self.lexer.eat(Token::Var(String::new()));
                        true
                    }
                    Token::Operator(Op::Comma) => {
                        m = self.lexer.eat(Token::Var(String::new()));
                        true
                    }
                    _ => panic!(
                        "Expected Comma or R-parens, got {} at position {}!",
                        temp.get_val(),
                        temp.get_pos()
                    ),
                }
            }
            _ => panic!(
                "Expected Var, got {} at position {}!",
                m.get_val(),
                m.get_pos()
            ),
        } {
            t = t.add_child(Node::make_node(m.clone()).type_(NodeType::FnArg))
                .type_(NodeType::FnArgs);
            m = self.get_curr();
        }

        self.lexer.eat(Token::Operator(Op::RParens));

        t
    }

// fn a (x, y, z) { if x == y return z * 2; else return z / 2; } fn b (l, m) { if a(l, m, 2) > 2 { x = 3; y = 16;  m = (l * x) / y; } else m = 2; return m; }


    // A non terminal function representing a function definition.
    //   A function defenition consists of the name ofthe function, 
    //   followed by a parenthesis enclosed set of comma separated 
    //   arguements, and then the body of the function enclosed in 
    //   block start and end tokens.
    fn function(&mut self) -> Node {
        self.lexer.eat(KEYWORD_TABLE[&"fn".to_owned()].clone());
        let t = Node::new()
            .val(self.lexer.eat(Token::Var(String::new())))
            .type_(NodeType::FnDef)
            .add_child(self.arg_list())
            .add_child(
            Node::new()
                .children(self.statement_list())
                .type_(NodeType::Block),
        );

        t
    }

    // A statement block.
    fn statement_list(&mut self) -> Vec<Node> {
        self.lexer.eat(Token::Operator(Op::BlockStart));
        let mut t = vec![
            self.statement()
        ];

        while match self.get_curr().get_val() {
            Token::Operator(Op::BlockEnd) => {
                self.lexer.eat(Token::Operator(Op::BlockEnd));
                false
            }
            _ => true,
        } {
            t.push(self.statement());
        }

        t
    }

    // A statement.
    fn statement(&mut self) -> Node {
        let mut t = Node::new();

        match self.get_curr().get_val() {
            // If it's a nested block.
            Token::Operator(Op::BlockStart) => {
                t = t.children(self.statement_list()).type_(NodeType::Block);
            }
            // If it is a variable or number, it's an expression.
            Token::Var(_) | Token::Number(_) | Token::Operator(Op::LParens) => t = self.expr(),

            // Various keywords.
            Token::KeyWord(x) => if x == "if".to_owned() {
                t = t.children(self.conditional_statement())
                    .type_(NodeType::Cond);
            } else if x == "return".to_owned() {
                t = self.return_statement();
            } else if x == "fn".to_owned() {
                t = self.function();
            },
            // This node is added to the AST so we can also handle intentionally empty statements.
            Token::Operator(Op::LineEnd) => {
                t = Node::new().val(self.lexer.eat(Token::Operator(Op::LineEnd)));
            }
            // An empty block.
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

    // Disambigutes the various node types which hold a Var token
    fn var_disambiguate(&mut self) -> Node {
        let m = self.lexer.peek_token();
        let t: Node;

        match m.get_val() {
            // If the next token is a left parenthesis, it can only be a function call.
            Token::Operator(Op::LParens) => t = self.fn_call(),

            // If the next token is a right parenthesis or a comma,
            //   the variable is probably part of an arg list.
            // This will probably be depreciated.
            Token::Operator(Op::RParens) | Token::Operator(Op::Comma) => t = self.id(),

            // If the next token is assign, we return an assign statement node.
            Token::Operator(Op::Assign) => t = self.assign_statement(),

            // Otherwise, it is a variable name.
            _ => t = self.id(),
        }

        t
    }

    // A statement is an assign statement if it contains the assign operator.
    fn assign_statement(&mut self) -> Node {
        let t = Node::new()
            // The variable we assign to.
            .add_child(self.id().type_(NodeType::Assignment))
            .val(self.lexer.eat(Token::Operator(Op::Assign)))
            // The expression we want to assign.
            .add_child(self.expr())
            .type_(NodeType::Assignment);

        if self.get_curr().get_val() == Token::Operator(Op::LineEnd) {
            self.lexer.eat(Token::Operator(Op::LineEnd));
        }
        
        t
    }

    // A return statement. It returns the value of the nested statement.
    fn return_statement(&mut self) -> Node {
        let t = Node::new()
            .val(self.lexer.eat(KEYWORD_TABLE[&"return".to_owned()].clone()))
            .type_(NodeType::Return)
            .add_child(self.statement());

        if self.get_curr().get_val() == Token::Operator(Op::LineEnd) {
            self.lexer.eat(Token::Operator(Op::LineEnd));
        }

        t
    }

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
        let t: Node;
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
                t = self.var_disambiguate();
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
        let mut t = Node::new()
            .val(self.lexer.eat(Token::Var("".to_owned())))
            .type_(NodeType::FnCall);
        self.lexer.eat(Token::Operator(Op::LParens));
        loop {
            match self.get_curr().get_val() {
                Token::Operator(Op::LParens) => t = t.add_child(self.expr().type_(NodeType::FnArg)),
                Token::Var(_) => t = t.add_child(self.var_disambiguate().type_(NodeType::FnArg)),
                Token::Operator(Op::RParens) => break,
                Token::Number(_) => t = t.add_child(self.number().type_(NodeType::FnArg)),
                _ => panic!(
                    "fn_call: Did not expect {} at position {} in function call args!",
                    self.get_curr().get_val(),
                    self.get_curr().get_pos()
                ),
            };
            match self.get_curr().get_val() {
                Token::Operator(Op::Comma) => {
                    self.lexer.eat(Token::Operator(Op::Comma));
                }
                Token::Operator(Op::RParens) => break,
                _ => panic!("herpaderp!"),
            }
        }
        self.lexer.eat(Token::Operator(Op::RParens));

        t
    }

    pub fn program(&mut self) -> Node {
        let mut t = Node::new();

        t = t.add_child(self.function()).type_(NodeType::Program);

        while match self.get_curr().get_val() {
            Token::None | Token::Operator(Op::LineEnd) => false,
            _ => true
        } {
            t = t.add_child(self.function());
        }

        t
    }
}

/* 

    function: function_name LPARENS args RPARENS scope

    function_name: VARIABLE

    scope: BLOCK_START statement_list BLOCK_END

    statement_list: statement LINE_END statement_list

    statement: expr | fn_call | scope | assign_statement | return_statement | conditional_statement

    conditional_statement: IF bool_expr scope (ELSE IF bool_expr scope)* (ELSE scope)?

    return_statement: RETURN expr ()

    assign_statement: VARIABLE ASSIGN (expr | bool_expr)

    fn_call: function_name LPARENS expr RPARENS

    args: arg (SEPARATOR arg)*

    arg: (QUALIFIER)? VARIABLE

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
