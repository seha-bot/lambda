use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    rc::Rc,
};

#[derive(Clone)]
pub enum Expr {
    Var(u32, Rc<RefCell<Option<Expr>>>),
    Lam(Rc<RefCell<Option<Expr>>>, Box<Expr>),
    App(Box<(Expr, Expr)>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Var(i, _) => write!(f, "{i}"),
            Expr::Lam(_, body) => write!(f, "λ {}", body),
            Expr::App(app) => {
                let (a, b) = &**app;

                if a.is_var() || a.is_app() {
                    write!(f, "{} ", a)?;
                } else {
                    write!(f, "({}) ", a)?;
                }

                if b.is_var() {
                    write!(f, "{}", b)
                } else {
                    write!(f, "({})", b)
                }
            }
        }
    }
}

impl Expr {
    fn is_app(&self) -> bool {
        if let Expr::App(_) = self {
            return true;
        }
        false
    }

    fn is_var(&self) -> bool {
        if let Expr::Var(_, _) = self {
            return true;
        }
        false
    }

    pub fn eval(mut self) -> Expr {
        loop {
            match self {
                Expr::Var(i, x) => {
                    if let Some(expr) = &*(*x).borrow() {
                        return expr.clone();
                    }
                    return Expr::Var(i, x);
                }
                Expr::Lam(var, body) => return Expr::Lam(var, Box::new(body.eval())),
                Expr::App(app) => {
                    let f = app.0.eval();
                    let x = app.1.eval();
                    if let Expr::Lam(var, body) = f {
                        *(*var).borrow_mut() = Some(x);
                        self = *body;
                    } else {
                        return Expr::App(Box::new((f, x)));
                    }
                }
            }
        }
    }
}

// x0 - x ones and trailing zero => de brujin index
// 00 - two zeros => abstraction
// 01ab - a and b are bit sequences => apply b to a (a b)

fn parse<'a>(
    prog: &'a [u8],
    refs: &mut HashMap<u32, Rc<RefCell<Option<Expr>>>>,
    depth: u32,
) -> Result<(Expr, &'a [u8]), ()> {
    match prog {
        [b'0', b'0', tail @ ..] => {
            let x = Rc::new(RefCell::new(None));
            refs.insert(depth, x.clone());
            let (body, tail) = parse(tail, refs, depth + 1)?;
            refs.remove(&depth);
            Ok((Expr::Lam(x, Box::new(body)), tail))
        }
        [b'0', b'1', tail @ ..] => {
            let (a, tail) = parse(tail, refs, depth)?;
            let (b, tail) = parse(tail, refs, depth)?;
            Ok((Expr::App(Box::new((a, b))), tail))
        }
        mut prog => {
            if *prog.first().unwrap_or(&b'0') == b'0' {
                return Err(());
            }

            let mut cnt = 0;
            while let Some(b'1') = prog.first() {
                cnt += 1;
                prog = &prog[1..];
            }

            if let None = prog.first() {
                return Err(());
            }
            prog = &prog[1..];

            if cnt == 0 {
                return Err(());
            }

            Ok((
                Expr::Var(cnt, refs.get(&(depth - cnt)).unwrap().clone()),
                prog,
            ))
        }
    }
}

pub fn temp_full_parse(prog: &str) -> Expr {
    let mut refs = HashMap::new();
    parse(prog.as_bytes(), &mut refs, 0).unwrap().0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> Expr {
        temp_full_parse("0010")
    }

    fn zero() -> Expr {
        temp_full_parse("000010")
    }

    fn inc() -> Expr {
        temp_full_parse("000000011100101111011010")
    }

    fn reduce(root: Expr, expected: &'static str) {
        let eval = root.eval();
        assert_eq!(eval.to_string(), expected);
    }

    #[test]
    fn test_id() {
        let id = id();
        let root = Expr::App(Box::new((id.clone(), id)));
        reduce(root, "λ 1");
    }

    #[test]
    fn test_inc() {
        let zero = zero();
        let inc = inc();

        let root = Expr::App(Box::new((inc.clone(), Expr::App(Box::new((inc, zero))))));

        reduce(root, "λ λ 2 (2 1)");
    }
}
