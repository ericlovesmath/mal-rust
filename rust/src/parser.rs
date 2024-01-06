use crate::tokenizer::Tokenizer;
use crate::types::Sexp;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref INTEGER_RE: Regex = Regex::new(r"^-?\d+$").unwrap();
    static ref COMMENT_RE: Regex = Regex::new(r"^;.*$").unwrap();
    static ref KEYWORD_RE: Regex = Regex::new(r"^:.+$").unwrap();
}

fn read_seq(tokenizer: &mut Tokenizer, closer: &str) -> Result<Vec<Sexp>, String> {
    let mut acc = Vec::new();
    loop {
        match tokenizer.peek() {
            Some(token) if token == closer => {
                tokenizer.next();
                return acc.into_iter().collect::<Result<Vec<Sexp>, _>>();
            }
            Some(_) => acc.push(Sexp::read_from(tokenizer)),
            None => return Err(format!("Expected {}, recieved EOF", closer)),
        }
    }
}

fn read_quote(tokenizer: &mut Tokenizer, repr: &str) -> Result<Sexp, String> {
    let quote = Ok(Sexp::Symbol(String::from(repr)));
    let contents = Sexp::read_from(tokenizer);
    let sexp = vec![quote, contents];
    sexp.into_iter()
        .collect::<Result<Vec<Sexp>, _>>()
        .map(Sexp::List)
}

impl Sexp {
    /** Parses tokens to AST */
    pub fn read_from(tokenizer: &mut Tokenizer) -> Result<Sexp, String> {
        match tokenizer.next() {
            Some(token) => match token.as_str() {
                "(" => read_seq(tokenizer, ")").map(Sexp::List),
                "[" => read_seq(tokenizer, "]").map(Sexp::Vec),
                "{" => read_seq(tokenizer, "}").map(Sexp::Map),
                "'" => read_quote(tokenizer, "quote"),
                "`" => read_quote(tokenizer, "quasiquote"),
                "~" => read_quote(tokenizer, "unquote"),
                "@" => read_quote(tokenizer, "deref"),
                "~@" => read_quote(tokenizer, "splice-unquote"),
                "true" => Ok(Sexp::Bool(true)),
                "false" => Ok(Sexp::Bool(false)),
                "nil" => Ok(Sexp::Nil),
                ")" | "]" | "}" => Err(format!("Unexpected token '{}'", token)),
                "^" => {
                    let quote = Ok(Sexp::Symbol(String::from("with-meta")));
                    let meta = Sexp::read_from(tokenizer);
                    let symbol = Sexp::read_from(tokenizer);
                    [quote, symbol, meta]
                        .into_iter()
                        .collect::<Result<Vec<Sexp>, _>>()
                        .map(Sexp::List)
                }
                comment if COMMENT_RE.is_match(comment) => Ok(Sexp::Nil),
                int if INTEGER_RE.is_match(int) => Ok(Sexp::Integer(
                    int.parse::<i64>()
                        .expect("Error: Failed to parse to i64, but matched INTEGER_RE"),
                )),
                keyword if KEYWORD_RE.is_match(keyword) => {
                    Ok(Sexp::Keyword(keyword[1..].to_string()))
                }
                _ => Ok(Sexp::Symbol(token)),
            },
            None => Err("Unexpected EOF".to_string()),
        }
    }
}
