use crate::types::{tokens_to_string, Sexp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Env = Rc<RefCell<EnvStruct>>;

pub struct EnvStruct {
    outer: Option<Env>,
    data: HashMap<String, Sexp>,
}

fn env_new(outer: Option<Env>) -> Env {
    Rc::new(RefCell::new(EnvStruct {
        outer,
        data: HashMap::new(),
    }))
}

impl EnvStruct {
    fn set(&mut self, symbol: &str, sexp: Sexp) {
        self.data.insert(symbol.to_string(), sexp);
    }

    fn get(&self, symbol: &str) -> Option<Sexp> {
        self.data
            .get(symbol)
            .cloned()
            .or(self.outer.as_ref().and_then(|env| env.borrow().get(symbol)))
    }
}

fn first_equal(list: &[Sexp], str: &str) -> bool {
    matches!(&list[0], Sexp::Symbol(sym) if sym == str)
}

pub fn evaluate(ast: Sexp, env: Env) -> Result<Sexp, String> {
    match ast {
        Sexp::List(list) if list.is_empty() => Ok(Sexp::List(list)),
        Sexp::List(list) if first_equal(&list, "def!") => {
            let [_, Sexp::Symbol(key), val] = list.as_slice() else {
                return Err(format!(
                    "def! expected [Key, Val], got {}",
                    tokens_to_string(&list)
                ));
            };
            let eval = evaluate(val.clone(), env.clone())?;
            env.borrow_mut().set(key, eval.clone());
            Ok(eval)
        }
        Sexp::List(list) | Sexp::Vec(list) if first_equal(&list, "let*") => {
            let ([Sexp::List(list), val] | [Sexp::Vec(list), val]) = &list[1..] else {
                return Err(format!(
                    "let* expected [Keys, Val], got {}",
                    tokens_to_string(&list)
                ));
            };
            if list.len() % 2 == 1 {
                return Err(format!(
                    "let* recieved an odd number of atoms on LHS: [{}]",
                    tokens_to_string(list)
                ));
            }
            let env = env_new(Some(env.clone()));
            for chunk in list.chunks_exact(2) {
                let [Sexp::Symbol(sym), expr] = chunk else {
                    return Err("let* did not recieve Sexp::Symbol".to_string());
                };
                let eval = evaluate(expr.clone(), env.clone())?;
                env.borrow_mut().set(sym, eval);
            }
            evaluate(val.clone(), env.clone())
        }
        Sexp::List(list) => {
            let Sexp::List(list) = apply(Sexp::List(list), env)? else {
                return Err("apply() didn't return Sexp::List".to_string());
            };
            let Some(Sexp::Func(func)) = list.first() else {
                return Err("Evaluated List missing Sexp::Func".to_string());
            };
            func(&list[1..])
        }
        _ => apply(ast, env),
    }
}

fn eval_sexps(sexps: Vec<Sexp>, env: Env) -> Result<Vec<Sexp>, String> {
    sexps
        .into_iter()
        .map(|s| evaluate(s, env.clone()))
        .collect::<Result<Vec<Sexp>, _>>()
}

fn apply(ast: Sexp, env: Env) -> Result<Sexp, String> {
    match ast {
        Sexp::Symbol(sym) => env
            .borrow()
            .get(sym.as_ref())
            .ok_or(format!("Unknown symbol '{}' found", sym)),
        Sexp::List(list) => eval_sexps(list, env).map(Sexp::List),
        Sexp::Vec(list) => eval_sexps(list, env).map(Sexp::Vec),
        ast => Ok(ast),
    }
}

macro_rules! arithmetic_op {
    ($func_name:ident, $operator:tt) => {
        fn $func_name(args: &[Sexp]) -> Result<Sexp, String> {
            let [Sexp::Integer(x), Sexp::Integer(y)] = args else {
                return Err(format!(
                    "{}() received unexpected inputs: [{}]",
                    stringify!($func_name), tokens_to_string(args)
                ));
            };
            Ok(Sexp::Integer(x $operator y))
        }
    };
}

arithmetic_op!(add, +);
arithmetic_op!(subtract, -);
arithmetic_op!(multiply, *);
arithmetic_op!(divide, /);

pub fn env_default() -> Env {
    let env = env_new(None);
    {
        let mut env = env.borrow_mut();
        env.set("+", Sexp::Func(add));
        env.set("-", Sexp::Func(subtract));
        env.set("*", Sexp::Func(multiply));
        env.set("/", Sexp::Func(divide));
    }
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;
    use crate::types::Sexp;

    fn test_eq(test: &str, expect: &str) {
        let ast = Sexp::read_from(&mut Tokenizer::new(test.to_string())).unwrap();
        let new_ast = evaluate(ast, env_default()).unwrap();
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
        assert!(evaluate(ast, env_default()).is_err());
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
