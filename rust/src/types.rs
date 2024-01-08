use std::fmt;

// TODO: Convert to Error Enum instead of String?
pub type Func = fn(&[Sexp]) -> Result<Sexp, String>;

// TODO: No clone?
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sexp {
    Integer(i64),
    Bool(bool),
    Symbol(String),
    List(Vec<Sexp>),
    Vec(Vec<Sexp>),
    Map(Vec<Sexp>),
    Keyword(String),
    String(String),
    Func(Func),
    Nil,
}

pub fn tokens_to_string(tokens: &[Sexp]) -> String {
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
            Sexp::String(s) => write!(
                f,
                r#""{}""#,
                s.replace(r#"\n"#, "\n")
                    .replace(r#"\""#, "\"")
                    .replace(r#"\\"#, "\\")
            ),
            Sexp::Nil => write!(f, "nil"),
            Sexp::List(tokens) => write!(f, "({})", tokens_to_string(tokens)),
            Sexp::Map(tokens) => write!(f, "{{{}}}", tokens_to_string(tokens)),
            Sexp::Vec(tokens) => write!(f, "[{}]", tokens_to_string(tokens)),
            Sexp::Func(_) => write!(f, "<func>"),
        }
    }
}
