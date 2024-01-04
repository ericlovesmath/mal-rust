use lazy_static::lazy_static;
use regex::Regex;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]

/** Reads tokens from given string */
pub struct Tokenizer(VecDeque<String>);

lazy_static! {
    static ref TOKENIZER_RE: Regex =
        Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"#)
            .unwrap();
}

impl Tokenizer {
    /** Turns `s` into tokens to be parsed */
    pub fn new(s: String) -> Self {
        Self(
            TOKENIZER_RE
                .captures_iter(&s)
                .map(|c| {
                    let (_, [token]) = c.extract();
                    token.to_string()
                })
                .collect(),
        )
    }

    pub fn peek(&mut self) -> Option<&str> {
        self.0.front().map(|s| s.as_str())
    }
}

impl Iterator for Tokenizer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(test: &str, expect: &str) {
        assert_eq!(
            Tokenizer::new(String::from(test)).0,
            String::from(expect)
                .split(' ')
                .map(String::from)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_tokenizer() {
        test("123", "123");
        test(" a b   c d    ", "a b c d");
        test("( 123   456)", "( 123 456 )");
        test(" (  + 2   (*  3  4)  )", "( + 2 ( * 3 4 ) )");
        test(
            "(def! nil?   (fn* [x] (= x nil  )))",
            "( def! nil? ( fn* [ x ] ( = x nil ) ) )",
        );
        test("(+ 1 2) ;;Comment", "( + 1 2 ) ;;Comment");
        test(
            "(defmacro! let*B (fn* [binds form]
                (let* [f (fn* [key val acc]
                    `((fn* [~key] ~acc) ~val))]
                        (_foldr_pairs f form binds))))",
            concat!(
                r"( defmacro! let*B ( fn* [ binds form ] ( let* [ f ( fn* ",
                r"[ key val acc ] ` ( ( fn* [ ~ key ] ~ acc ) ~ val ) ) ] ",
                r"( _foldr_pairs f form binds ) ) ) )",
            ),
        );

        assert!(Tokenizer::new(String::new()).0.is_empty());
    }

    #[test]
    fn test_iterator() {
        let mut tk = Tokenizer::new(String::from(" a b (c   d)   "));
        assert_eq!(tk.next(), Some(String::from("a")));
        assert_eq!(tk.next(), Some(String::from("b")));
        assert_eq!(tk.next(), Some(String::from("(")));
        assert_eq!(tk.next(), Some(String::from("c")));
        assert_eq!(tk.next(), Some(String::from("d")));
        assert_eq!(tk.next(), Some(String::from(")")));
        assert_eq!(tk.next(), None);
        assert_eq!(tk.next(), None);
    }
}
