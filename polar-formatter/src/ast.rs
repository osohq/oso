use polar_core::terms;
use terms::Symbol;

pub type Comment = String;

#[derive(Debug, Clone)]
pub struct Field(pub Box<Node>, pub Box<Node>);

pub type Fields = Vec<Field>;

#[derive(Debug, Clone)]
pub struct Dictionary(pub Fields);

#[derive(Debug, Clone)]
pub struct Call {
    pub name: Box<Node>,
    pub args: Vec<Node>,
    pub kwargs: Option<Vec<Field>>,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub operator: terms::Operator,
    pub args: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct InstanceLiteral {
    pub tag: Box<Node>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Dictionary(Dictionary),
    Instance(InstanceLiteral),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub parameter: Box<Node>,
    pub specializer: Option<Box<Node>>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: Box<Node>,
    pub parameters: Vec<Parameter>,
    pub body: Option<Box<Node>>,
}

// #[derive(Debug, Clone)]
// pub struct ShorthandRule {
//     /// `Term` is a `String`. E.g., `"member"` in `"member" if "owner";`.
//     pub head: Term,
//     /// The first `Term` is the 'implier' `String`, e.g., `"owner"` in `"member" if "owner";`. The
//     /// `Option` is the optional 'relation' `Symbol` and `String`, e.g., `on "parent"` in `"member"
//     /// if "owner" on "parent";`.
//     pub body: (Term, Option<(Term, Term)>),
// }

// #[derive(Debug, Clone)]
// pub enum ResourceBlockLine {
//     Declaration(Box<Node>, Box<Node>),
//     Rule(Box<Node>, Box<Node>),
// }

// #[derive(Debug, Clone)]
// pub struct ResourceBlock {
//     pub lines: Vec<ResourceBlockLine>,
// }

#[derive(Debug, Clone)]
pub struct File(pub Vec<Node>);

#[derive(Debug, Clone)]
pub enum Value {
    Number(terms::Numeric),
    Symbol(String),
    String(String),
    Boolean(bool),
    Dictionary(Dictionary),
    Pattern(Pattern),
    Call(Call),
    List(Vec<Node>),
    Variable(Symbol),
    RestVariable(Symbol),
    Expression(Operation),
    Rule(Rule),
    File(File),
    // ResourceBlock(ResourceBlock),
    // ResourceBlockLine(ResourceBlockLine),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub value: Value,
    pub start: usize,
    pub end: usize,
    pub lines_before: usize,
    // pub comments_before: Vec<Comment>,
    // pub comments_after: Vec<Comment>,
}

impl Node {
    pub fn from_parser(value: Value, start: usize, end: usize) -> Node {
        Node {
            value,
            start,
            end,
            lines_before: 0,
        }
    }
}
