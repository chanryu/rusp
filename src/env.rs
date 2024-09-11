use crate::expr::Expr;
use crate::prelude::load_prelude;
use crate::proc::Proc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Env {
    base: Option<Box<Env>>,
    vars: Rc<RefCell<HashMap<String, Expr>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            base: None,
            vars: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn new_root_env() -> Self {
        use crate::built_in;

        let env = Env::new();

        let set_native_func = |name, func| {
            env.define(
                name,
                Expr::Proc(Proc::Native {
                    name: name.to_owned(),
                    func,
                }),
            );
        };

        // lisp primitives
        set_native_func("atom?", built_in::atom);
        set_native_func("car", built_in::car);
        set_native_func("cdr", built_in::cdr);
        set_native_func("cons", built_in::cons_);
        set_native_func("cond", built_in::cond);
        set_native_func("define", built_in::define);
        set_native_func("defmacro", built_in::defmacro);
        set_native_func("display", built_in::display);
        set_native_func("eq?", built_in::eq);
        set_native_func("eval", built_in::eval_);
        set_native_func("lambda", built_in::lambda);
        set_native_func("set!", built_in::set);

        // quote
        set_native_func("quote", built_in::quote::quote);
        set_native_func("quasiquote", built_in::quote::quasiquote);

        // num
        set_native_func("+", built_in::num::add);
        set_native_func("-", built_in::num::minus);
        set_native_func("*", built_in::num::multiply);
        set_native_func("/", built_in::num::divide);
        set_native_func("num?", built_in::num::is_num);

        // prelude
        load_prelude(&env);

        env
    }

    pub fn define<IntoExpr>(&self, name: &str, expr: IntoExpr)
    where
        IntoExpr: Into<Expr>,
    {
        self.vars.borrow_mut().insert(name.into(), expr.into());
    }

    pub fn update<IntoExpr>(&self, name: &str, expr: IntoExpr) -> bool
    where
        IntoExpr: Into<Expr>,
    {
        let mut env = self;
        loop {
            if let Some(value) = env.vars.borrow_mut().get_mut(name) {
                *value = expr.into();
                return true;
            }
            if let Some(base) = &env.base {
                env = base;
            } else {
                return false;
            }
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Expr> {
        let mut env = self;
        loop {
            if let Some(value) = env.vars.borrow().get(name) {
                return Some(value.clone());
            }
            if let Some(base) = &env.base {
                env = base;
            } else {
                break;
            }
        }
        None
    }

    pub fn derive(&self) -> Env {
        let mut derived_env = Env::new();
        derived_env.base = Some(Box::new(self.clone()));
        derived_env
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::num;

    #[test]
    fn test_set() {
        let env = Env::new();
        assert_eq!(env.vars.borrow().len(), 0);
        env.define("one", 1);
        assert_eq!(env.vars.borrow().get("one"), Some(&num(1)));
    }

    #[test]
    fn test_update() {
        let env = Env::new();
        assert_eq!(env.update("name", 1), false);

        env.define("name", 0);
        assert_eq!(env.update("name", 1), true);
    }

    #[test]
    fn test_lookup() {
        let env = Env::new();
        assert_eq!(env.lookup("one"), None);
        env.define("one", num(1));
        assert_eq!(env.lookup("one"), Some(num(1)));
    }

    #[test]
    fn test_derive_update() {
        let base = Env::new();
        let derived = base.derive();

        base.define("one", 1);
        derived.define("two", 2);

        assert_eq!(derived.update("one", "uno"), true);
        assert_eq!(derived.update("two", "dos"), true);

        assert_eq!(base.vars.borrow().get("one"), Some(&"uno".into()));
        assert_eq!(derived.vars.borrow().get("one"), None);
        assert_eq!(derived.vars.borrow().get("two"), Some(&"dos".into()));
    }

    #[test]
    fn test_derive_lookup() {
        let base = Env::new();
        let derived = base.derive();

        assert_eq!(derived.lookup("two"), None);
        base.define("two", 2);
        assert_eq!(derived.lookup("two"), Some(num(2)));

        derived.define("three", 3);
        assert_eq!(base.lookup("three"), None);
        assert_eq!(derived.lookup("three"), Some(num(3)));
    }

    #[test]
    fn test_clone() {
        let original = Env::new();
        let cloned = original.clone();

        original.define("one", 1);
        assert_eq!(cloned.lookup("one"), Some(num(1)));
    }
}
