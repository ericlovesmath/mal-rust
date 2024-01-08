use crate::types::Sexp;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Env {
    outer: Option<Box<Env>>,
    data: HashMap<String, Sexp>,
}

impl Env {
    pub fn new(outer: Option<Env>) -> Self {
        Self {
            outer: outer.map(Box::new),
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, symbol: String, sexp: &Sexp) {
        self.data.insert(symbol, sexp.clone());
    }

    fn get(&self, symbol: &String) -> Option<Sexp> {
        match self.data.get(symbol) {
            None => match &self.outer {
                None => None,
                Some(env) => env.get(symbol),
            },
            Some(sexp) => Some(sexp.clone()),
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        let mut env = Self::new(None);
        env.set("+".to_string(), &Sexp::Func(add));
        env.set("-".to_string(), &Sexp::Func(subtract));
        env.set("*".to_string(), &Sexp::Func(multiply));
        env.set("/".to_string(), &Sexp::Func(divide));
        env
    }
}

pub fn evaluate(ast: Sexp, env: &mut Env) -> Result<Sexp, String> {
    match ast {
        Sexp::List(list) if list.is_empty() => Ok(Sexp::List(list)),
        Sexp::List(list) => {
            // TODO: Improve
            match list.first().unwrap() {
                Sexp::Symbol(sym) if sym == "def!" => {
                    assert_eq!(list.len(), 3);
                    let eval = evaluate(list.get(2).unwrap().clone(), env)?;
                    if let Sexp::Symbol(sym) = list.get(1).unwrap().clone() {
                        env.set(sym, &eval);
                        Ok(eval)
                    } else {
                        panic!("def! failed");
                    }
                }
                Sexp::Symbol(sym) if sym == "let*" => {
                    assert_eq!(list.len(), 3);
                    let mut new_env = Env::new(Some(env.clone())); // TODO: RC instead
                    if let Sexp::List(sub_list) = list.get(1).unwrap().clone() {

                        // Sub eval, same code copy + pasted from def!
                        // TODO: Move to own function?
                        for i in 0..(sub_list.len() / 2) {
                            let sub_eval = evaluate(sub_list.get(2 * i + 1).unwrap().clone(), &mut new_env)?;
                            if let Sexp::Symbol(sym) = sub_list.get(2 * i).unwrap().clone() {
                                new_env.set(sym, &sub_eval);
                            } else {
                                panic!("let* inner failed");
                            }
                        }

                        evaluate(list.get(2).unwrap().clone(), &mut new_env)
                    } else {
                        panic!("let* failed");
                    }
                }
                _ => match apply(Sexp::List(list), env) {
                    Ok(Sexp::List(eval_list)) => match eval_list.first() {
                        Some(Sexp::Func(func)) => func(&eval_list[1..]),
                        Some(_) => Err("eval_list missing Sexp::Func".to_string()),
                        None => unreachable!(),
                    },
                    Ok(_) => Err("apply() didn't return Sexp::List".to_string()),
                    Err(s) => Err(s),
                },
            }
        }
        _ => apply(ast, env),
    }
}

fn apply(ast: Sexp, env: &mut Env) -> Result<Sexp, String> {
    match ast {
        Sexp::Symbol(sym) => match env.get(&sym) {
            None => Err(format!("Unknown symbol '{}' found", sym)),
            Some(sexp) => Ok(sexp), // TODO: Make RC Cell?
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;
    use crate::types::Sexp;

    fn test_eq(test: &str, expect: &str) {
        let ast = Sexp::read_from(&mut Tokenizer::new(test.to_string())).unwrap();
        let new_ast = evaluate(ast, &mut Env::default()).unwrap();

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
        assert!(evaluate(ast, &mut Env::default()).is_err());
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
