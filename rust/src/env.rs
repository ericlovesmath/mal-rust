use crate::types::{Env, Sexp};
use std::collections::HashMap;

pub fn evaluate(ast: Sexp, env: &Env) -> Result<Sexp, String> {
    match ast {
        Sexp::List(list) if list.is_empty() => Ok(Sexp::List(list)),
        Sexp::List(list) => match apply(Sexp::List(list), env) {
            Ok(Sexp::List(eval_list)) => match eval_list.first() {
                Some(Sexp::Func(func)) => func(&eval_list[1..]),
                Some(_) => Err("eval_list missing Sexp::Func".to_string()),
                None => unreachable!(),
            },
            Ok(_) => Err("apply() didn't return Sexp::List".to_string()),
            Err(s) => Err(s),
        },
        _ => apply(ast, env),
    }
}

fn apply(ast: Sexp, env: &Env) -> Result<Sexp, String> {
    match ast {
        Sexp::Symbol(sym) => match env.get(&sym) {
            None => Err(format!("Unknown symbol '{}' found", sym)),
            Some(func) => Ok(Sexp::Func(*func)), // TODO: Make RC Cell?
        },
        Sexp::List(list) => list
            .into_iter()
            .map(|s| evaluate(s, env))
            .collect::<Result<Vec<Sexp>, _>>()
            .map(Sexp::List),
        ast => Ok(ast),
    }
}

macro_rules! arithmetic_op {
    ($func_name:ident, $operator:tt) => {
        fn $func_name(args: &[Sexp]) -> Result<Sexp, String> {
            if let [Sexp::Integer(x), Sexp::Integer(y)] = args {
                Ok(Sexp::Integer(x $operator y))
            } else {
                Err(
                    format!("{}() received unexpected inputs: [{}]",
                    stringify!($func_name),
                    args.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "))
                )
            }
        }
    };
}

arithmetic_op!(add, +);
arithmetic_op!(subtract, -);
arithmetic_op!(multiply, *);
arithmetic_op!(divide, /);

pub fn get_repl_env() -> Env {
    let mut env: Env = HashMap::new();
    env.insert("+".to_string(), add);
    env.insert("-".to_string(), subtract);
    env.insert("*".to_string(), multiply);
    env.insert("/".to_string(), divide);
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;
    use crate::types::Sexp;

    fn test_eq(test: &str, expect: &str) {
        let ast = Sexp::read_from(&mut Tokenizer::new(test.to_string())).unwrap();
        let new_ast = evaluate(ast, &get_repl_env()).unwrap();

        assert_eq!(new_ast.to_string(), expect);
    }

    #[test]
    fn test_basic_repl_env() {
        test_eq("10", "10");
        test_eq("(+ 9 3)", "12");
        test_eq("(- 9 3)", "6");
        test_eq("(* 9 3)", "27");
        test_eq("(/ 9 3)", "3");
        test_eq("(/ (* -4 (- 3 9)) (+ 4 2))", "4");
        test_eq("()", "()");
    }

    fn test_fail(test: &str) {
        let ast = Sexp::read_from(&mut Tokenizer::new(test.to_string())).unwrap();
        assert!(evaluate(ast, &get_repl_env()).is_err());
    }

    #[test]
    fn test_repl_env_expect_fail() {
        test_fail("(+)");
        test_fail("(+ 1)");
        test_fail("(+ 1 2 3)");
        test_fail("(+ + +)");
        test_fail("(+ + 1 2)");
    }
}
