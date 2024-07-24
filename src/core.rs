use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

#[derive(Clone)]
enum Expr {
    Var(Rc<RefCell<Option<Expr>>>),
    Lam(Rc<RefCell<Option<Expr>>>, Box<Expr>),
    App(Box<(Expr, Expr)>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ctx = HashMap::new();
        write!(f, "{}", self.fmt_impl(&mut ctx, 1))
    }
}

impl Expr {
    fn fmt_impl(&self, ctx: &mut HashMap<*mut Option<Expr>, u32>, depth: u32) -> String {
        match self {
            Expr::Var(x) => (depth - ctx.get(&x.as_ptr()).unwrap()).to_string(),
            Expr::Lam(x, body) => {
                let ptr = x.as_ptr();
                ctx.insert(ptr, depth);
                let res = format!("λ {}", body.fmt_impl(ctx, depth + 1));
                ctx.remove(&ptr);
                res
            }
            Expr::App(app) => {
                let (a, b) = &**app;

                let out = if a.is_var() || a.is_app() {
                    format!("{} ", a.fmt_impl(ctx, depth))
                } else {
                    format!("({}) ", a.fmt_impl(ctx, depth))
                };

                if b.is_var() {
                    out + &b.fmt_impl(ctx, depth)
                } else {
                    out + &format!("({})", b.fmt_impl(ctx, depth))
                }
            }
        }
    }

    fn is_app(&self) -> bool {
        if let Expr::App(_) = self {
            return true;
        }
        false
    }

    fn is_var(&self) -> bool {
        if let Expr::Var(_) = self {
            return true;
        }
        false
    }

    pub fn eval(mut self) -> Expr {
        loop {
            match self {
                Expr::Var(x) => {
                    if let Some(expr) = &*(*x).borrow() {
                        return expr.clone();
                    }
                    return Expr::Var(x);
                }
                Expr::Lam(var, body) => {
                    *(*var).borrow_mut() = None;
                    return Expr::Lam(var, Box::new(body.eval()));
                }
                Expr::App(app) => {
                    let f = app.0.eval(); // you only need to reduce f to WHNF
                    let x = app.1.eval(); // f and x get evaluated multiple times. avoid this.
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

// TODO: write errors for this and don't unwrap
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

            Ok((Expr::Var(refs.get(&(depth - cnt)).unwrap().clone()), prog))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "0010";
    const ZERO: &str = "000010";
    const INC: &str = "000000011100101111011010";
    const TRUE: &str = "0000110";
    const FALSE: &str = ZERO;

    // TODO: remove this
    fn temp_full_parse(prog: &str) -> Expr {
        let mut refs = HashMap::new();
        parse(prog.as_bytes(), &mut refs, 0).unwrap().0
    }

    fn reduce(prog: &str, expected_prog: &str, expected: &str) {
        let prog_ast = temp_full_parse(prog);
        assert_eq!(prog_ast.to_string(), expected_prog);
        assert_eq!(prog_ast.eval().to_string(), expected);
    }

    #[test]
    fn test_id() {
        let prog = format!("01{ID}{ID}");
        reduce(&prog, "(λ 1) (λ 1)", "λ 1");
    }

    #[test]
    fn test_inc() {
        let prog = format!("01{INC}01{INC}{ZERO}");
        reduce(
            &prog,
            "(λ λ λ 2 (3 2 1)) ((λ λ λ 2 (3 2 1)) (λ λ 1))",
            "λ λ 2 (2 1)",
        );
    }

    #[test]
    fn test_pow() {
        let two = "0000011100111010";
        let three = "000001110011100111010";

        let prog = format!("01{two}{two}");
        reduce(&prog, "(λ λ 2 (2 1)) (λ λ 2 (2 1))", "λ λ 2 (2 (2 (2 1)))");

        let prog = format!("01{three}{two}");
        reduce(
            &prog,
            "(λ λ 2 (2 (2 1))) (λ λ 2 (2 1))",
            "λ λ 2 (2 (2 (2 (2 (2 (2 (2 1)))))))",
        );
    }

    #[test]
    fn test_even() {
        let not = format!("00010110{FALSE}{TRUE}");
        let even = format!("00010110{not}{TRUE}");
        let even_ast = temp_full_parse(&even);
        let expected = ["λ λ 2", "λ λ 1"];

        let mut x = temp_full_parse(ZERO);
        for i in 0..=100 {
            let query = Expr::App(Box::new((even_ast.clone(), x.clone())));
            assert_eq!(query.eval().to_string(), expected[i % 2]);
            x = Expr::App(Box::new((temp_full_parse(INC), x)));
        }
    }
}
