use core::{Expression, Term};
use std::collections::HashMap;

mod core;
mod sll;

fn reduce_full(x: Expression) -> Term {
    let mut x = x.reduce().unwrap();
    let mut seen = HashMap::new();
    x.fix_names(&mut seen);
    x
}

fn main() {
    let a = sll::num(3);
    let b = sll::num(5);

    let root = Expression {
        terms: vec![
            Term::Lambda(Box::new(sll::plus())),
            Term::Lambda(Box::new(a)),
            Term::Lambda(Box::new(b)),
        ],
    };

    println!("{}", reduce_full(root));
}
