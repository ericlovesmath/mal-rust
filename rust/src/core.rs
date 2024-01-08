use crate::env::{env_new, Env};
use crate::types::{tokens_to_string as to_str, Sexp};

pub fn env_core() -> Env {
    let env = env_new(None);
    {
        let mut env = env.borrow_mut();
        let mut set = |sym, func| env.set(sym, Sexp::Func(func));
        set("+", add);
        set("-", subtract);
        set("*", multiply);
        set("/", divide);
        set("prn", prn);
        set("println", println);
        set("pr-str", pr_str);
        set("str", str);
        set("list", list);
        set("list?", is_list);
        set("empty?", is_empty);
        set("count", count);
        set("=", eq);
        set("<", lt);
        set(">", gt);
        set("<=", le);
        set(">=", ge);
    }
    env
}

macro_rules! arithmetic_op {
    ($func:ident, $op:tt) => {
        fn $func(args: &[Sexp]) -> Result<Sexp, String> {
            match args {
                [Sexp::Integer(x), Sexp::Integer(y)] => Ok(Sexp::Integer(x $op y)),
                _ => Err(format!("{}() received unexpected inputs: [{}]", stringify!($func), to_str(args)))
            }
        }
    };
}

arithmetic_op!(add, +);
arithmetic_op!(subtract, -);
arithmetic_op!(multiply, *);
arithmetic_op!(divide, /);

macro_rules! cmp {
    ($func:ident, $op:tt) => {
        fn $func(args: &[Sexp]) -> Result<Sexp, String> {
            match args {
                [sexp_l, sexp_r] => Ok(Sexp::Bool(sexp_l $op sexp_r)),
                _ => Err(format!("{} expects 2 args, received {}", stringify!($op), to_str(args))),
            }
        }
    };
}

cmp!(eq, ==);
cmp!(lt, <);
cmp!(le, <=);
cmp!(gt, >);
cmp!(ge, >=);

fn prn(args: &[Sexp]) -> Result<Sexp, String> {
    let Sexp::String(s) = pr_str(args)? else {
        return Err("prn failed unexpectedly".to_string());
    };
    println!("{}", s);
    Ok(Sexp::Nil)
}

fn println(args: &[Sexp]) -> Result<Sexp, String> {
    let Sexp::String(s) = str(args)? else {
        return Err("println failed unexpectedly".to_string());
    };
    println!("{}", s);
    Ok(Sexp::Nil)
}

fn pr_str(args: &[Sexp]) -> Result<Sexp, String> {
    Ok(Sexp::String(to_str(args)))
}

fn str(args: &[Sexp]) -> Result<Sexp, String> {
    Ok(Sexp::String(
        args.iter()
            .map(|s| match s {
                Sexp::String(s) => s.to_string(),
                _ => s.to_string(),
            })
            .collect::<Vec<String>>()
            .join(""),
    ))
}

fn list(args: &[Sexp]) -> Result<Sexp, String> {
    Ok(Sexp::List(args.to_vec()))
}

fn is_list(args: &[Sexp]) -> Result<Sexp, String> {
    Ok(Sexp::Bool(matches!(args, [Sexp::List(_)])))
}

fn is_empty(args: &[Sexp]) -> Result<Sexp, String> {
    Ok(Sexp::Bool(matches!(args, [Sexp::List(l)] if l.is_empty())))
}

fn count(args: &[Sexp]) -> Result<Sexp, String> {
    match args {
        [Sexp::List(list)] => Ok(Sexp::Integer(list.len() as i64)),
        _ => Err(format!("count expected 1 List, recieved {}", to_str(args))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::evaluate;
    use crate::tokenizer::Tokenizer;
    use crate::types::Sexp;

    fn test_eq(test: &str, expect: &str) {
        let ast = Sexp::read_from(&mut Tokenizer::new(test.to_string())).unwrap();
        let new_ast = evaluate(ast, env_core()).unwrap();
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
        assert!(evaluate(ast, env_core()).is_err());
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
