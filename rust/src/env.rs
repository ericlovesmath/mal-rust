use crate::types::{tokens_to_string, Sexp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Env = Rc<RefCell<EnvStruct>>;

pub struct EnvStruct {
    outer: Option<Env>,
    data: HashMap<String, Sexp>,
}

pub fn env_new(outer: Option<Env>) -> Env {
    Rc::new(RefCell::new(EnvStruct {
        outer,
        data: HashMap::new(),
    }))
}

pub fn env_default() -> Env {
    let env = env_new(None);
    env.borrow_mut().set("+".to_string(), &Sexp::Func(add));
    env.borrow_mut().set("-".to_string(), &Sexp::Func(subtract));
    env.borrow_mut().set("*".to_string(), &Sexp::Func(multiply));
    env.borrow_mut().set("/".to_string(), &Sexp::Func(divide));
    env
}

impl EnvStruct {
    pub fn set(&mut self, symbol: String, sexp: &Sexp) {
        self.data.insert(symbol, sexp.clone());
    }

    fn get(&self, symbol: &str) -> Option<Sexp> {
        match self.data.get(symbol) {
            None => match &self.outer {
                None => None,
                Some(env) => env.borrow().get(symbol),
            },
            Some(sexp) => Some(sexp.clone()),
        }
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
            env.borrow_mut().set(key.clone(), &eval);
            Ok(eval)
        }
        Sexp::List(list) if first_equal(&list, "let*") => {
            let [_, Sexp::List(list), val] = list.as_slice() else {
                return Err(format!(
                    "let* expected [Keys, Val], got {}",
                    tokens_to_string(&list)
                ));
            };
            let env = env_new(Some(env.clone()));
            for chunk in list.chunks_exact(2) {
                let [Sexp::Symbol(sym), expr] = chunk else {
                    return Err("let* did not recieve Sexp::Symbol".to_string());
                };
                let eval = evaluate(expr.clone(), env.clone())?;
                env.borrow_mut().set(sym.clone(), &eval);
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

fn apply(ast: Sexp, env: Env) -> Result<Sexp, String> {
    match ast {
        Sexp::Symbol(sym) => env
            .borrow()
            .get(sym.as_ref())
            .ok_or(format!("Unknown symbol '{}' found", sym)),
        Sexp::List(list) => list
            .into_iter()
            .map(|s| evaluate(s, env.clone()))
            .collect::<Result<Vec<Sexp>, _>>()
            .map(Sexp::List),
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
