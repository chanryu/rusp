use rusche::{
    eval::{eval, EvalContext, EvalResult},
    expr::{Expr, NIL},
    list::List,
};
use std::io::Write;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("print", print);
    context.env.define_native_proc("println", println);
    context.env.define_native_proc("read", read);
    context.env.define_native_proc("read-num", read_num);
}

fn print(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

fn println(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    println!();
    Ok(NIL)
}

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(format!("Error reading input: {}", error));
    }
    Ok(Expr::from(input.trim()))
}

fn read_num(proc_name: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(format!("Error reading input: {}", error));
    }

    let text = input.trim();
    match text.parse::<f64>() {
        Ok(num) => Ok(Expr::from(num)),
        Err(err) => Err(format!("{}: {}", proc_name, err)),
    }
}