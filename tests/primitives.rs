mod common;

use common::{cons, num, test_eval};
use rusp::{expr::Expr, list::NIL};

#[test]
fn test_cond() {
    assert_eq!(test_eval("(cond (t 0) (f 1))"), Ok(num(0)));
    assert_eq!(test_eval("(cond (f 0) (t 1))"), Ok(num(1)));
}

#[test]
fn test_quote() {
    assert_eq!(test_eval("'1"), Ok(num(1)));
    assert_eq!(test_eval("'(1)"), Ok(Expr::List(cons(num(1), NIL))));
    assert_eq!(
        test_eval("'(1 2)"),
        Ok(Expr::List(cons(num(1), cons(num(2), NIL))))
    );
}