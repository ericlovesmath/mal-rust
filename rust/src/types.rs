use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Sexp {
    Integer(i64),
    Bool(bool),
    Symbol(String),
    List(Vec<Sexp>),
    Vec(Vec<Sexp>),
    Map(Vec<Sexp>),
    Keyword(String),
    Func(Func),
    Nil,
}

fn list_to_string(tokens: &[Sexp]) -> String {
    tokens
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sexp::Integer(num) => write!(f, "{}", num),
            Sexp::Bool(boolean) => write!(f, "{}", boolean),
            Sexp::Symbol(sym) => write!(f, "{}", sym),
            Sexp::Keyword(sym) => write!(f, ":{}", sym),
            Sexp::Nil => write!(f, "nil"),
            Sexp::List(tokens) => write!(f, "({})", list_to_string(tokens)),
            Sexp::Map(tokens) => write!(f, "{{{}}}", list_to_string(tokens)),
            Sexp::Vec(tokens) => write!(f, "[{}]", list_to_string(tokens)),
            Sexp::Func(func) => write!(f, "{}", func),
        }
    }
}

pub type Env = HashMap<String, Func>;

// TODO: Convert to Error Enum instead of String
// TODO: nargs is not necessary since I'm validating inputs on `run` anyways
#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub nargs: u8,
    pub func: fn(&[Sexp]) -> Result<Sexp, String>,
}

impl Func {
    pub fn run(&self, args: &[Sexp]) -> Result<Sexp, String> {
        // TODO: Remove assert?
        assert_eq!(self.nargs as usize, args.len());
        (self.func)(args)
    }
}

// TODO: Make this slightly better
impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fn with {} args", self.nargs)
    }
}
