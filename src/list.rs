use crate::expr::Expr;
use std::fmt;
use std::iter::Iterator;

#[derive(Clone, Debug, PartialEq)]
pub struct Cons {
    pub car: Box<Expr>,
    pub cdr: Box<List>,
}

impl Cons {
    pub fn new<T>(car: T, cdr: List) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            car: Box::new(car.into()),
            cdr: Box::new(cdr),
        }
    }

    pub fn cdar(&self) -> Option<&Expr> {
        if let List::Cons(cons) = self.cdr.as_ref() {
            Some(cons.car.as_ref())
        } else {
            None
        }
    }
}

pub fn cons<T>(car: T, cdr: List) -> List
where
    T: Into<Expr>,
{
    List::Cons(Cons::new(car, cdr))
}

#[derive(Clone, Debug, PartialEq)]
pub enum List {
    Cons(Cons),
    Nil,
}

impl List {
    pub fn iter(&self) -> ListIter {
        ListIter::new(self)
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn is_nil(&self) -> bool {
        if let List::Nil = self {
            true
        } else {
            false
        }
    }

    pub fn cdr(&self) -> Option<&List> {
        if let List::Cons(cons) = &self {
            Some(cons.cdr.as_ref())
        } else {
            None
        }
    }
}

impl Into<Expr> for List {
    fn into(self) -> Expr {
        Expr::List(self)
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_list(f, self, true)
    }
}

fn write_list(f: &mut fmt::Formatter<'_>, list: &List, is_top_level: bool) -> fmt::Result {
    if is_top_level {
        write!(f, "(")?;
    }
    if let List::Cons(cons) = list {
        if is_top_level {
            write!(f, "{}", cons.car)?;
        } else {
            write!(f, " {}", cons.car)?;
        }

        write_list(f, cons.cdr.as_ref(), false)?
    }
    if is_top_level {
        write!(f, ")")?;
    }
    Ok(())
}

pub struct ListIter<'a> {
    list: &'a List,
}

impl<'a> ListIter<'a> {
    pub fn new(list: &'a List) -> Self {
        Self { list }
    }
}

impl<'a> Iterator for ListIter<'a> {
    type Item = &'a Expr;

    fn next(&mut self) -> Option<Self::Item> {
        if let List::Cons(cons) = self.list {
            let car = &cons.car;
            self.list = &cons.cdr;
            Some(car)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::{num, str, sym};
    use crate::macros::list;

    #[test]
    fn test_display() {
        let list = list!(num(1), num(2), list!(num(3), sym("sym"), str("str")));
        assert_eq!(format!("{}", list), "(1 2 (3 sym \"str\"))");
    }

    #[test]
    fn test_iter() {
        let list = list!(num(1), num(2), num(3));
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&num(1)));
        assert_eq!(iter.next(), Some(&num(2)));
        assert_eq!(iter.next(), Some(&num(3)));
        assert_eq!(iter.next(), None);
    }
}
