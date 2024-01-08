use crate::types::{tokens_to_string, Sexp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// TODO: Stop copying the Sexps so much
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

    fn get(&self, symbol: &String) -> Option<Sexp> {
        match self.data.get(symbol) {
            None => match &self.outer {
                None => None,
                Some(env) => env.borrow().get(symbol),
            },
            Some(sexp) => Some(sexp.clone()),
        }
    }
}

pub fn evaluate(ast: Sexp, env: Env) -> Result<Sexp, String> {
    match ast {
        Sexp::List(list) if list.is_empty() => Ok(Sexp::List(list)),
        Sexp::List(list) if matches!(&list[0], Sexp::Symbol(sym) if sym == "def!") => {
            if let [_, Sexp::Symbol(key), val] = list.as_slice() {
                let eval = evaluate(val.clone(), env.clone())?;
                env.borrow_mut().set(key.clone(), &eval);
                Ok(eval)
            } else {
                Err(format!(
                    "def! expected [Key, Val], got {}",
                    tokens_to_string(&list)
                ))
            }
        }
        Sexp::List(list) if matches!(&list[0], Sexp::Symbol(sym) if sym == "let*") => {
            if let [_, Sexp::List(list), val] = list.as_slice() {
                let env = env_new(Some(env.clone()));
                for chunk in list.chunks_exact(2) {
                    if let [Sexp::Symbol(sym), expr] = chunk {
                        let eval = evaluate(expr.clone(), env.clone())?;
                        env.borrow_mut().set(sym.clone(), &eval);
                    } else {
                        return Err("let* did not recieve Sexp::Symbol".to_string());
                    }
                }
                evaluate(val.clone(), env.clone())
            } else {
                Err(format!(
                    "let* expected [(Keys), Val], got {}",
                    tokens_to_string(&list)
                ))
            }
        }
        Sexp::List(list) => match apply(Sexp::List(list), env) {
            Ok(Sexp::List(eval_list)) => {
                if let Some(Sexp::Func(func)) = eval_list.first() {
                    func(&eval_list[1..])
                } else {
                    Err("eval_list missing Sexp::Func".to_string())
                }
            }
            Ok(_) => Err("apply() didn't return Sexp::List".to_string()),
            Err(s) => Err(s),
        },
        _ => apply(ast, env),
    }
}

fn apply(ast: Sexp, env: Env) -> Result<Sexp, String> {
    match ast {
        Sexp::Symbol(sym) => env
            .borrow()
            .get(&sym)
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
