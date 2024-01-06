use crate::types::{Env, Func, Sexp};
use std::collections::HashMap;

pub fn evaluate(ast: Sexp, env: &Env) -> Result<Sexp, String> {
    match ast {
        Sexp::List(list) => {
            // TODO: Has to be a better way than this-
            if list.is_empty() {
                Ok(Sexp::List(list))
            } else {
                match apply(Sexp::List(list), env) {
                    Ok(Sexp::List(eval_list)) => match eval_list.first() {
                        Some(Sexp::Func(func)) => func.run(&eval_list[1..]),
                        Some(_) => Err("eval_list missing Sexp::Func".to_string()),
                        None => unreachable!(),
                    },
                    Ok(_) => Err("apply() didn't return Sexp::List".to_string()),
                    Err(s) => Err(s),
                }
            }
        }
        _ => apply(ast, env),
    }
}

// TODO: Remove all String errors, return enum instead
fn apply(ast: Sexp, env: &Env) -> Result<Sexp, String> {
    match ast {
        Sexp::Symbol(sym) => match env.get(&sym) {
            None => Err(format!("Unknown symbol '{}' found", sym)),
            Some(func) => Ok(Sexp::Func(func.clone())), // TODO: Make RC Cell
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
                Err(format!("{}() received unexpected input: {:?}", stringify!($func_name), args))
            }
        }
    };
}

macro_rules! get_env {
    ($($name:expr => $function:expr, $args:expr;)+) => {
        {
            let mut env: Env = HashMap::new();
            $(
                env.insert(
                    $name.to_string(),
                    Func { nargs: $args, func: $function },
                );
            )+
            env
        }
    };
}

arithmetic_op!(add, +);
arithmetic_op!(subtract, -);
arithmetic_op!(multiply, *);

// TODO: Div by 0 Error
arithmetic_op!(divide, /);

pub fn get_repl_env() -> Env {
    get_env!(
        "+" => add, 2;
        "-" => subtract, 2;
        "*" => multiply, 2;
        "/" => divide, 2;
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;
    use crate::types::Sexp;

    fn test_eq(test: &str, expect: &str) {
        let ast = Sexp::read_from(&mut Tokenizer::new(test.to_string())).unwrap();
        let new_ast = evaluate(ast, &get_repl_env()).unwrap();

        assert_eq!(expect, new_ast.to_string());
    }

    #[test]
    fn test_add() {
        test_eq("(+ (+ 1 (- 7 9)) (/ 3 4))", "-1");
        test_eq("()", "()");
    }
}
