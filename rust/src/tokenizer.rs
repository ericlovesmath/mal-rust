use lazy_static::lazy_static;
use regex::Regex;

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

        assert!(Tokenizer::new(String::new()).tokens.is_empty());
    }
}
