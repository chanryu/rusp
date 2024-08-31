pub mod num;

use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::{cons, List};
use crate::proc::Proc;

pub fn atom(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        // TODO: error if cdr is not NIL
        if eval(car, env)?.is_atom() {
            Ok(Expr::new_sym("#t"))
        } else {
            Ok(NIL)
        }
    } else {
        Err(make_syntax_error("atom", args))
    }
}

pub fn car(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        Ok(eval(car, env)?)
    } else {
        Err(make_syntax_error("car", args))
    }
}

pub fn cdr(args: &List, env: &Env) -> EvalResult {
    if let Some(cdr) = args.cdr() {
        if let Some(cdar) = cdr.car() {
            return Ok(eval(cdar, env)?);
        }
    }

    Err(make_syntax_error("cdr", args))
}

pub fn cond(args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    loop {
        match iter.next() {
            None => {
                return Ok(NIL);
            }
            Some(Expr::List(List::Cons(cons))) => {
                let car = cons.car.as_ref();
                if eval(car, env)? != NIL {
                    if let Some(cdar) = cons.cdr.car() {
                        return eval(cdar, env);
                    } else {
                        break;
                    }
                }
            }
            _ => break,
        }
    }

    Err(make_syntax_error("cond", args))
}

pub fn define(args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = iter.next() {
                env.set(name, eval(expr, env)?.clone());
                Ok(NIL)
            } else {
                Err("define expects a expression after symbol".into())
            }
        }
        _ => Err("define expects a symbol".into()),
    }
}

pub fn eq(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        if let Some(cdr) = args.cdr() {
            if let Some(cdar) = cdr.car() {
                let arg1 = eval(car, env)?;
                let arg2 = eval(cdar, env)?;
                return if arg1 == arg2 {
                    Ok(Expr::new_sym("#t"))
                } else {
                    Ok(NIL)
                };
            }
        }
    }

    Err(make_syntax_error("eq", args))
}

pub fn lambda(args: &List, env: &Env) -> EvalResult {
    if let List::Cons(cons) = args {
        if let Expr::List(List::Cons(formal_args)) = cons.car.as_ref() {
            // TODO: check if formal_args are list of symbols.

            let lambda_body = cons.cdr.as_ref();

            return Ok(Expr::Proc(Proc::Closure {
                formal_args: List::Cons(formal_args.clone()),
                lambda_body: lambda_body.clone(),
                outer_env: env.clone(),
            }));
        }
    }
    Err(make_syntax_error("lambda", args))
}

pub fn quasiquote(args: &List, env: &Env) -> EvalResult {
    let mut exprs = Vec::new();
    let mut iter = args.iter();
    while let Some(expr) = iter.next() {
        let Expr::List(list) = expr else {
            exprs.push(expr.clone());
            continue;
        };

        let List::Cons(cons) = list else {
            exprs.push(List::Nil.into());
            continue;
        };

        let Expr::Sym(name) = cons.car.as_ref() else {
            exprs.push(quasiquote(list, env)?);
            continue;
        };

        match name.as_str() {
            "quote" => {
                exprs.push(expr.clone());
            }
            "unquote" => {
                exprs.push(eval(expr, env)?);
            }
            "unquote-splicing" => {
                let result = eval(expr, env)?;
                if let Expr::List(List::Cons(cons)) = result {
                    exprs.push(cons.car.as_ref().clone());
                    let mut l = cons.cdr.as_ref();
                    while let List::Cons(cons) = l {
                        exprs.push(cons.car.as_ref().clone());
                        l = cons.cdr.as_ref();
                    }
                } else {
                    exprs.push(result);
                }
            }
            _ => {
                exprs.push(quasiquote(list, env)?);
            }
        }
    }

    Ok(exprs.into())
}

pub fn quote(args: &List, _env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        // TODO: error if cdr is not NIL
        Ok(car.clone())
    } else {
        Err(make_syntax_error("quote", args))
    }
}

pub fn unquote(args: &List, env: &Env) -> EvalResult {
    let mut exprs = Vec::new();
    for expr in args.iter() {
        exprs.push(eval(expr, env)?);
    }
    Ok(exprs.into())
}

pub fn unquote_splicing(args: &List, env: &Env) -> EvalResult {
    unquote(args, env)
}

fn make_syntax_error(func_name: &str, args: &List) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        cons(Expr::new_sym(func_name), args.clone())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::{num, str, sym};
    use crate::list::cons;
    use crate::macros::list;

    #[test]
    fn test_car() {
        let env = Env::new();
        // (car '(1 2)) => 1
        let ret = car(&list!(num(1), num(2)), &env);
        assert_eq!(ret, Ok(num(1)));
    }

    #[test]
    fn test_cdr() {
        let env = Env::new();
        // (cdr '(1 2)) => 2
        let ret = cdr(&list!(num(1), num(2)), &env);
        assert_eq!(ret, Ok(num(2)));
    }

    #[test]
    fn test_define() {
        let env = Env::new();
        // (define name "value")
        let ret = define(&list!(sym("name"), str("value")), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.lookup("name"), Some(str("value")));
    }

    #[test]
    fn test_eq() {
        let env = Env::new();
        // (eq 1 1) => #t
        assert_eq!(eq(&list!(num(1), num(1)), &env), Ok(sym("#t")));
        // (eq 1 2) => ()
        assert_eq!(eq(&list!(num(1), num(2)), &env), Ok(NIL));
        // (eq "str" "str") => #t
        assert_eq!(eq(&list!(str("str"), str("str")), &env), Ok(sym("#t")));
        // (eq 1 "1") => ()
        assert_eq!(eq(&list!(num(1), str("1")), &env), Ok(NIL));
    }

    #[test]
    fn test_quote() {
        let env = Env::new();
        // (quote (1 2)) => (1 2)
        let ret = quote(&list!(list!(num(1), num(2))), &env);
        assert_eq!(ret, Ok(list!(num(1), num(2)).into()));
    }
}
