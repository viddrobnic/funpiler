#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Number(i64),
    Id(String),
    Not(Box<Node>),
    Equal(Box<Node>, Box<Node>),
    NotEqual(Box<Node>, Box<Node>),
    Add(Box<Node>, Box<Node>),
    Subtract(Box<Node>, Box<Node>),
    Multiply(Box<Node>, Box<Node>),
    Divide(Box<Node>, Box<Node>),
    Call { callee: String, args: Vec<Node> },
    Return(Box<Node>),
    Block(Vec<Node>),
    If(If),
    Function(Function),
    Var(String, Box<Node>),
    Assignment(String, Box<Node>),
    While(While),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub condition: Box<Node>,
    pub consequence: Box<Node>,
    pub alternative: Box<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct While {
    pub condition: Box<Node>,
    pub body: Box<Node>,
}
