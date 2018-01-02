use parser::Token;

#[derive(Debug, Clone)]
pub struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    val: Token,
}

impl Node {
    pub fn new() -> Node {
        Node {
            left: None,
            right: None,
            val: Token::None,
        }
    }

    pub fn left(mut self, l: Option<Box<Node>>) -> Node {
        self.left = l;
        self
    }

    pub fn right(mut self, r: Option<Box<Node>>) -> Node {
        self.right = r;
        self
    }

    pub fn val(mut self, v: Token) -> Node {
        self.val = v;
        self
    }
}
