use crate::expr::Expr;
use crate::span::Span;
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
            Some(&cons.car)
        } else {
            None
        }
    }
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
            Some(&cons.cdr)
        } else {
            None
        }
    }

    pub fn span(&self) -> Option<Span> {
        let mut iter = self.iter();

        match (iter.next(), iter.last()) {
            (Some(first), Some(last)) => match (first.span(), last.span()) {
                (Some(first_span), Some(last_span)) => {
                    Some(Span::new(first_span.begin, last_span.end))
                }
                _ => None,
            },
            (Some(first), None) => first.span(),
            _ => None,
        }
    }
}

impl<'a> From<ListIter<'a>> for List {
    fn from(val: ListIter<'a>) -> Self {
        val.list.clone()
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

        write_list(f, &cons.cdr, false)?
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

pub fn cons<T>(car: T, cdr: List) -> List
where
    T: Into<Expr>,
{
    List::Cons(Cons::new(car, cdr))
}

#[macro_export]
macro_rules! list {
    () => {
        $crate::list::List::Nil
    };

    ($car:literal $(, $cdr:expr)*) => {
        $crate::list::cons($crate::expr::Expr::from($car), list!($($cdr),*))
    };

    ($car:expr $(, $cdr:expr)*) => {
        $crate::list::cons($car, list!($($cdr),*))
    };
}

pub(crate) use list;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::span::Loc;

    #[test]
    fn test_display() {
        let list = list!(1, 2, list!(3, "str", intern("sym")));
        assert_eq!(format!("{}", list), "(1 2 (3 \"str\" sym))");
    }

    #[test]
    fn test_list_span() {
        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)))),
            Expr::Num(2.0, Some(Span::new(Loc::new(1, 3), Loc::new(1, 4)))),
            Expr::Num(3.0, Some(Span::new(Loc::new(1, 5), Loc::new(1, 6))))
        );
        assert_eq!(args.span(), Some(Span::new(Loc::new(1, 1), Loc::new(1, 6))));

        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, None),
            Expr::Num(2.0, Some(Span::new(Loc::new(1, 3), Loc::new(1, 4)))),
            Expr::Num(3.0, Some(Span::new(Loc::new(1, 5), Loc::new(1, 6))))
        );
        assert_eq!(args.span(), None);

        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)))),
            Expr::Num(2.0, Some(Span::new(Loc::new(1, 3), Loc::new(1, 4)))),
            Expr::Num(3.0, None)
        );
        assert_eq!(args.span(), None);

        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)))),
            Expr::Num(2.0, None),
            Expr::Num(3.0, Some(Span::new(Loc::new(1, 5), Loc::new(1, 6))))
        );
        assert_eq!(args.span(), Some(Span::new(Loc::new(1, 1), Loc::new(1, 6))));
    }

    #[test]
    fn test_iter() {
        let list = list!(1, 2, 3);
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&num(1)));
        assert_eq!(iter.next(), Some(&num(2)));
        assert_eq!(iter.next(), Some(&num(3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_list_macro() {
        // (cons 0 nil) => (list 0)
        assert_eq!(cons(0, List::Nil), list!(0));

        // (cons 0 (cons 1 nil)) => (list 0 1)
        assert_eq!(cons(0, cons(1, List::Nil)), list!(0, 1));

        // (cons 0 (cons (cons 1 nil) (cons 2 nil))) => (list 0 (list 1) 2)
        assert_eq!(
            cons(0, cons(cons(1, List::Nil), cons(2, List::Nil))),
            list!(0, list!(1), 2)
        );
    }
}
