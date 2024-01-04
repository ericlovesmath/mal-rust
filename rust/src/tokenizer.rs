use lazy_static::lazy_static;
use regex::Regex;

/** Reads tokens from given string */
struct Tokenizer {
    tokens: Vec<String>,
    pos: usize,
}

lazy_static! {
    static ref TOKENIZER_RE: Regex =
        Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"#)
            .unwrap();
}

impl Tokenizer {
    /** Turns `s` into tokens to be parsed */
    pub fn new(s: String) -> Self {
        let tokens = TOKENIZER_RE
            .captures_iter(&s)
            .map(|c| {
                let (_, [token]) = c.extract();
                token.to_string()
            })
            .collect();
        Self { tokens, pos: 0 }
    }

    /** Returns current `token`, advances `Tokenizer`.
    Returns `None` if Tokenizer is completely consumed. */
    pub fn next(&mut self) -> Option<&str> {
        if self.pos >= self.tokens.len() {
            return None;
        };
        let token = &self.tokens[self.pos];
        self.pos += 1;
        Some(token)
    }

    /** Returns current token, doesn't advance Tokenizer
    Returns `None` if Tokenizer is completely consumed. */
    pub fn peek(&mut self) -> Option<&str> {
        if self.pos >= self.tokens.len() {
            return None;
        };
        Some(&self.tokens[self.pos])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(test: &str, expect: &str) {
        assert_eq!(
            Tokenizer::new(String::from(test)).tokens,
            String::from(expect)
                .split(' ')
                .map(|s| s.to_string())
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

        assert!(Tokenizer::new(String::new()).tokens.is_empty());
    }

    #[test]
    fn test_next_peek() {
        let mut tk = Tokenizer::new(String::from(" a b   c d     "));
        assert_eq!(tk.peek(), Some("a"));
        assert_eq!(tk.next(), Some("a"));
        assert_eq!(tk.peek(), Some("b"));
        assert_eq!(tk.next(), Some("b"));
        assert_eq!(tk.next(), Some("c"));
        assert_eq!(tk.peek(), Some("d"));
        assert_eq!(tk.next(), Some("d"));
        assert_eq!(tk.peek(), None);
        assert_eq!(tk.next(), None);
        assert_eq!(tk.next(), None);
        assert_eq!(tk.peek(), None);
    }
}
