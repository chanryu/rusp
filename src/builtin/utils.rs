use crate::eval::{eval, eval_error, EvalContext, EvalError};
use crate::expr::{intern, Expr};
use crate::list::{cons, List};

pub fn syntax_error(proc_name: &str, args: &List) -> EvalError {
    EvalError::SyntaxError(format!(
        "Ill-formed syntax: {}",
        cons(intern(proc_name), args.clone())
    ))
}

pub fn get_exact_1_arg<'a>(proc_name: &str, args: &'a List) -> Result<&'a Expr, EvalError> {
    let mut iter = args.iter();
    let Some(arg) = iter.next() else {
        return Err(eval_error!(ArityError, "{proc_name} needs an argument."));
    };
    if iter.next().is_none() {
        Ok(arg)
    } else {
        Err(eval_error!(
            ArityError,
            "{proc_name} expects only 1 argument."
        ))
    }
}

pub fn get_exact_2_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr), EvalError> {
    let mut iter = args.iter();
    let Some(arg1) = iter.next() else {
        return Err(eval_error!(
            ArityError,
            "{}: requres two arguments",
            proc_name
        ));
    };
    let Some(arg2) = iter.next() else {
        return Err(eval_error!(
            ArityError,
            "{}: requres two arguments",
            proc_name
        ));
    };
    if iter.next().is_none() {
        Ok((arg1, arg2))
    } else {
        Err(eval_error!(
            ArityError,
            "{}: takes only two arguments",
            proc_name
        ))
    }
}

#[allow(dead_code)]
pub fn get_exact_3_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr, &'a Expr), EvalError> {
    let mut iter = args.iter();
    let Some(arg1) = iter.next() else {
        return Err(eval_error!(
            ArityError,
            "{}: requres 3 arguments",
            proc_name
        ));
    };
    let Some(arg2) = iter.next() else {
        return Err(eval_error!(
            ArityError,
            "{}: requres 3 arguments",
            proc_name
        ));
    };
    let Some(arg3) = iter.next() else {
        return Err(eval_error!(
            ArityError,
            "{}: requres 3 arguments",
            proc_name
        ));
    };
    if iter.next().is_none() {
        Ok((arg1, arg2, arg3))
    } else {
        Err(eval_error!(
            ArityError,
            "{}: takes only 3 arguments",
            proc_name
        ))
    }
}

pub fn get_2_or_3_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr, Option<&'a Expr>), EvalError> {
    let mut iter = args.iter();
    let Some(arg1) = iter.next() else {
        return Err(eval_error!(
            ArityError,
            "{}: requres at least 2 arguments",
            proc_name
        ));
    };
    let Some(arg2) = iter.next() else {
        return Err(eval_error!(
            ArityError,
            "{}: requres at least 2 arguments",
            proc_name
        ));
    };
    let Some(arg3) = iter.next() else {
        return Ok((arg1, arg2, None));
    };
    if iter.next().is_none() {
        Ok((arg1, arg2, Some(arg3)))
    } else {
        Err(eval_error!(
            ArityError,
            "{}: takes up to 3 arguments",
            proc_name
        ))
    }
}

pub fn make_formal_args(list: &List) -> Result<Vec<String>, EvalError> {
    let mut formal_args = Vec::new();
    for item in list.iter() {
        let Expr::Sym(formal_arg, _) = item else {
            return Err(eval_error!(TypeError, "{item} is not a symbol."));
        };
        formal_args.push(formal_arg.clone());
    }

    Ok(formal_args)
}

pub fn eval_to_str(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<String, EvalError> {
    match eval(expr, context)? {
        Expr::Str(text, _) => Ok(text),
        _ => Err(eval_error!(
            TypeError,
            "{proc_name}: {expr} does not evaluate to a string."
        )),
    }
}

pub fn eval_to_num(proc_name: &str, expr: &Expr, context: &EvalContext) -> Result<f64, EvalError> {
    match eval(expr, context)? {
        Expr::Num(value, _) => Ok(value),
        _ => Err(eval_error!(
            TypeError,
            "{proc_name}: {expr} does not evaluate to a number."
        )),
    }
}
