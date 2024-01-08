use std::fmt;

// TODO: Convert to Error Enum instead of String?
pub type Func = fn(&[Sexp]) -> Result<Sexp, String>;

#[derive(PartialEq, Clone)]
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
            Sexp::Func(_) => write!(f, "<func>"),
        }
    }
}
