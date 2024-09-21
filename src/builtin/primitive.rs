use std::rc::Rc;

use crate::{
    env::Env,
    eval::{eval, EvalResult},
    expr::{Expr, ExprKind, NIL},
    list::List,
    proc::Proc,
};

use super::utils::{get_exact_1_arg, get_exact_2_args, make_formal_args, make_syntax_error};

pub fn atom(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    Ok(eval(expr, env)?.is_atom().into())
}

pub fn car(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    if let ExprKind::List(List::Cons(cons)) = eval(expr, env)?.kind {
        Ok(cons.car.as_ref().clone())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cdr(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    if let ExprKind::List(List::Cons(cons)) = eval(expr, env)?.kind {
        Ok(cons.cdr.as_ref().clone().into())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cons(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (car, cdr) = get_exact_2_args(proc_name, args)?;

    let car = eval(car, env)?;
    let ExprKind::List(cdr) = eval(cdr, env)?.kind else {
        return Err(format!("{proc_name}: {cdr} does not evaluate to a list."));
    };

    Ok(crate::list::cons(car, cdr).into())
}

pub fn cond(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut iter = args.iter();
    loop {
        match iter.next().map(|e| &e.kind) {
            None => {
                return Ok(NIL);
            }
            Some(ExprKind::List(List::Cons(cons))) => {
                let car = &cons.car;
                if eval(car, env)?.is_truthy() {
                    if let Some(expr) = cons.cdar() {
                        return eval(expr, env);
                    } else {
                        break;
                    }
                }
            }
            _ => break,
        }
    }

    Err(make_syntax_error(proc_name, args))
}

pub fn define(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut iter = args.iter();
    match iter.next().map(|e| &e.kind) {
        Some(ExprKind::Sym(name)) => {
            if let Some(expr) = iter.next() {
                env.define(name, eval(expr, env)?);
                Ok(NIL)
            } else {
                Err(format!(
                    "{proc_name}: define expects a expression after symbol"
                ))
            }
        }
        Some(ExprKind::List(List::Cons(cons))) => {
            let ExprKind::Sym(name) = &cons.car.kind else {
                return Err(format!("{proc_name}: expects a list of symbols"));
            };

            env.define(
                name,
                Expr::new(
                    ExprKind::Proc(Proc::Closure {
                        name: Some(name.to_owned()),
                        formal_args: make_formal_args(&cons.cdr)?,
                        body: Box::new(iter.into()),
                        outer_env: env.clone(),
                    }),
                    None,
                ),
            );
            Ok(NIL)
        }
        _ => Err(make_syntax_error(proc_name, args)),
    }
}

pub fn defmacro(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut iter = args.iter();

    let (macro_name, formal_args) = match iter.next().map(|e| &e.kind) {
        // (defmacro name (args) body)
        Some(ExprKind::Sym(macro_name)) => {
            let Some(ExprKind::List(list)) = iter.next().map(|e| &e.kind) else {
                return Err(make_syntax_error(proc_name, args));
            };

            (macro_name, make_formal_args(list)?)
        }
        // (defmacro (name args) body)
        Some(ExprKind::List(List::Cons(cons))) => {
            let ExprKind::Sym(macro_name) = &cons.car.kind else {
                return Err(make_syntax_error(proc_name, args));
            };

            (macro_name, make_formal_args(&cons.cdr)?)
        }
        _ => return Err(make_syntax_error(proc_name, args)),
    };

    env.define(
        macro_name,
        Expr::new(
            ExprKind::Proc(Proc::Macro {
                name: Some(macro_name.clone()),
                formal_args,
                body: Box::new(iter.into()),
            }),
            None,
        ),
    );

    Ok(NIL)
}

pub fn eq(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (left, right) = get_exact_2_args(proc_name, args)?;

    Ok((eval(left, env)? == eval(right, env)?).into())
}

pub fn eval_(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    eval(&eval(expr, env)?, env)
}

pub fn lambda(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut iter = args.iter();

    let Some(ExprKind::List(list)) = iter.next().map(|e| &e.kind) else {
        return Err(make_syntax_error(proc_name, args));
    };

    Ok(Expr::new(
        ExprKind::Proc(Proc::Closure {
            name: None,
            formal_args: make_formal_args(list)?,
            body: Box::new(iter.into()),
            outer_env: env.clone(),
        }),
        None,
    ))
}

pub fn set(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (name_expr, value_expr) = get_exact_2_args(proc_name, args)?;

    let ExprKind::Sym(name) = &name_expr.kind else {
        return Err("".to_owned());
    };

    env.update(name, eval(value_expr, &env)?);

    Ok(NIL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::expr::intern;
    use crate::list::list;

    #[test]
    fn test_define() {
        let evaluator = Evaluator::new();
        let env = evaluator.root_env();

        // (define name "value")
        let ret = define("", &list!(intern("name"), "value"), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.lookup("name"), Some("value".into()));
    }

    #[test]
    fn test_eq() {
        let evaluator = Evaluator::new();
        let env = evaluator.root_env();

        // (eq 1 1) => #t
        assert_ne!(eq("", &list!(1, 1), &env).unwrap(), NIL);
        // (eq 1 2) => ()
        assert_eq!(eq("", &list!(1, 2), &env).unwrap(), NIL);
        // (eq "str" "str") => #t
        assert_ne!(eq("", &list!("str", "str"), &env).unwrap(), NIL);
        // (eq 1 "1") => ()
        assert_eq!(eq("", &list!(1, "1"), &env).unwrap(), NIL);
    }
}
