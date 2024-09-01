use alloc::rc::Rc;
use const_format::formatcp;
use core::{cell::RefCell, fmt::Display};

#[derive(Clone)]
enum Expr {
    Var(Rc<RefCell<Option<Expr>>>),
    Lam(Rc<RefCell<Option<Expr>>>, Box<Expr>),
    App(Box<(Expr, Expr)>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ctx = Vec::new();
        write!(f, "{}", self.fmt_impl(&mut ctx, 1))
    }
}

impl Expr {
    fn fmt_impl(&self, ctx: &mut Vec<*mut Option<Expr>>, depth: usize) -> String {
        match self {
            Expr::Var(x) => {
                let i = ctx.iter().rev().position(|&e| e == x.as_ptr());
                let i = ctx.len() - i.expect("incorrect AST");
                assert!(i < depth, "incorrect AST");
                (depth - i).to_string()
            }
            Expr::Lam(x, body) => {
                ctx.push(x.as_ptr());
                let res = format!("λ {}", body.fmt_impl(ctx, depth + 1));
                ctx.pop();
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

    fn fmt_blc(&self, ctx: &mut Vec<*mut Option<Expr>>, depth: usize) -> String {
        match self {
            Expr::Var(x) => {
                let i = ctx.iter().rev().position(|&e| e == x.as_ptr());
                let i = ctx.len() - i.expect("incorrect AST");
                assert!(i < depth, "incorrect AST");

                (0..depth - i).map(|_| '1').collect::<String>() + "0"
            }
            Expr::Lam(x, body) => {
                ctx.push(x.as_ptr());
                let res = format!("00{}", body.fmt_blc(ctx, depth + 1));
                ctx.pop();
                res
            }
            Expr::App(app) => {
                let (a, b) = &**app;
                format!("01{}{}", a.fmt_blc(ctx, depth), b.fmt_blc(ctx, depth))
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

    // I'm pretty sure this can be written without recursion
    fn eval_var_refs(&mut self) {
        match self {
            Expr::Var(x) => {
                let expr = (*x).borrow().clone();
                if let Some(expr) = expr {
                    *self = expr;
                }
            }
            Expr::Lam(var, body) => {
                let x = (*var).borrow_mut().take();
                body.eval_var_refs();
                *(*var).borrow_mut() = x;
            }
            Expr::App(app) => {
                app.0.eval_var_refs();
                app.1.eval_var_refs();
            }
        };
    }

    fn eval_full(mut self) -> Expr {
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
                    return Expr::Lam(var, Box::new(body.eval_full()));
                }
                Expr::App(app) => {
                    let f = app.0.eval_full();
                    let x = app.1.eval_full();
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

    fn eval_one(self) -> (Expr, bool) {
        match self {
            var @ Expr::Var(_) => (var, false),
            lam @ Expr::Lam(_, _) => (lam, false),
            Expr::App(app) => {
                let (f, has_changed) = app.0.eval_one();
                let x = app.1;
                if has_changed {
                    return (Expr::App(Box::new((f, x))), true);
                }
                if let Expr::Lam(var, mut body) = f {
                    *(*var).borrow_mut() = Some(x);
                    body.eval_var_refs();
                    (*body, true)
                } else {
                    (Expr::App(Box::new((f, x))), false)
                }
            }
        }
    }

    fn eval_lazy(mut self) -> Expr {
        let mut has_changed = true;
        while has_changed {
            (self, has_changed) = self.eval_one();
        }
        self
    }
}

// x0 - x ones and trailing zero => de bruijn index
// 00 - two zeros => abstraction
// 01ab - a and b are bit sequences => apply b to a (a b)

#[derive(Debug)]
pub enum ParseError {
    ZeroBruijnIndex,
    BruijnIndexOutOfBounds,
    IncompleteStatement,
}

fn parse_impl<'a>(
    prog: &'a [u8],
    refs: &mut Vec<Rc<RefCell<Option<Expr>>>>,
    depth: usize,
) -> Result<(Expr, &'a [u8]), ParseError> {
    match prog {
        [b'0', b'0', tail @ ..] => {
            let x = Rc::new(RefCell::new(None));
            refs.push(Rc::clone(&x));
            let (body, tail) = parse_impl(tail, refs, depth + 1)?;
            refs.pop();
            Ok((Expr::Lam(x, Box::new(body)), tail))
        }
        [b'0', b'1', tail @ ..] => {
            let (a, tail) = parse_impl(tail, refs, depth)?;
            let (b, tail) = parse_impl(tail, refs, depth)?;
            Ok((Expr::App(Box::new((a, b))), tail))
        }
        mut prog => {
            let mut cnt = 0;
            while let Some(b'1') = prog.first() {
                prog = &prog[1..];
                cnt += 1;
            }

            if let Some(b'0') = prog.first() {
                prog = &prog[1..];

                if cnt == 0 {
                    return Err(ParseError::ZeroBruijnIndex);
                }

                match refs.get(depth.wrapping_sub(cnt)) {
                    Some(x) => Ok((Expr::Var(Rc::clone(x)), prog)),
                    None => Err(ParseError::BruijnIndexOutOfBounds),
                }
            } else {
                Err(ParseError::IncompleteStatement)
            }
        }
    }
}

fn parse(prog: &str) -> Result<Expr, ParseError> {
    let mut refs = Vec::new();
    Ok(parse_impl(prog.as_bytes(), &mut refs, 0)?.0)
}

#[derive(Clone, Copy)]
pub enum OutputFmt {
    AsciiBinary,
    HumanReadable,
    Parsed,
}

const PAIR_BLANK: &str = "00010110";
const PAIR_END: &str = "000010";
const TRUE: &str = "0000110";
const FALSE: &str = "000010";
const PAIR_TRUE: &str = formatcp!("{PAIR_BLANK}{TRUE}");
const PAIR_FALSE: &str = formatcp!("{PAIR_BLANK}{FALSE}");

fn byte_to_blc(mut x: u8) -> String {
    let mut data = [false; 8];

    for i in 0..8 {
        data[7 - i] = x % 2 == 1;
        x /= 2;
    }

    let data = data.iter().fold(String::new(), |mut acc, &x| {
        acc.push_str(if x { PAIR_TRUE } else { PAIR_FALSE });
        acc
    });

    format!("{data}{PAIR_END}")
}

fn blc_to_byte(mut data: &str) -> (u8, usize) {
    let mut x = 0;
    let mut bytes_read = 0;

    for _ in 0..8 {
        x <<= 1;
        if data.starts_with(PAIR_TRUE) {
            x |= 1;
            data = &data[PAIR_TRUE.len()..];
            bytes_read += PAIR_TRUE.len();
        } else if data.starts_with(PAIR_FALSE) {
            data = &data[PAIR_FALSE.len()..];
            bytes_read += PAIR_FALSE.len();
        } else {
            panic!("bad blc in byte loop");
        }
    }

    if data.starts_with(PAIR_END) {
        (x, bytes_read + PAIR_END.len())
    } else {
        panic!("bad blc in byte end")
    }
}

fn bytes_to_blc(buf: &[u8]) -> String {
    let data = buf.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(PAIR_BLANK);
        acc.push_str(&byte_to_blc(*x));
        acc
    });

    format!("{data}{PAIR_END}")
}

fn blc_to_bytes(mut data: &str) -> Vec<u8> {
    let mut out = Vec::new();

    loop {
        if data.starts_with(PAIR_BLANK) {
            data = &data[PAIR_BLANK.len()..];
            let (x, bytes_read) = blc_to_byte(data);
            out.push(x);
            data = &data[bytes_read..];
        } else if data == PAIR_END {
            break;
        } else {
            panic!("bad blc in bytes loop");
        }
    }

    out
}

pub fn run(prog: &str, args: Option<&str>, output_fmt: OutputFmt) -> Result<String, ParseError> {
    let prog = if let Some(args) = args {
        Expr::App(Box::new((
            parse(prog)?,
            parse(&bytes_to_blc(args.as_bytes()))?,
        )))
    } else {
        parse(prog)?
    };

    let expr = prog.eval_lazy().eval_full();

    let mut ctx = Vec::new();
    Ok(match output_fmt {
        OutputFmt::AsciiBinary => expr.fmt_blc(&mut ctx, 1),
        OutputFmt::HumanReadable => expr.to_string(),
        OutputFmt::Parsed => blc_to_bytes(&expr.fmt_blc(&mut ctx, 1))
            .iter()
            .map(|&x| x as char)
            .collect::<String>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: use formatcp here
    const ID: &str = "0010";
    const ZERO: &str = "000010";
    const ONE: &str = "00000111010";
    const TWO: &str = "0000011100111010";
    const THREE: &str = "000001110011100111010";
    const INC: &str = "000000011100101111011010";
    const TRUE: &str = "0000110";
    const FALSE: &str = ZERO;
    const PAIR: &str = "0000000101101110110";
    const FST: &str = "0001100000110";
    const SND: &str = "000110000010";
    const Y: &str = "000100011100110100001110011010";

    type Result = core::result::Result<(), ParseError>;

    fn reduce(prog: &str, expected_prog: &str, expected: &str) -> Result {
        let prog_ast = parse(prog)?;
        assert_eq!(prog_ast.to_string(), expected_prog);
        assert_eq!(prog_ast.eval_lazy().eval_full().to_string(), expected);
        Ok(())
    }

    #[test]
    fn test_id() -> Result {
        let prog = format!("01{ID}{ID}");
        reduce(&prog, "(λ 1) (λ 1)", "λ 1")
    }

    #[test]
    fn test_inc() -> Result {
        let prog = format!("01{INC}01{INC}{ZERO}");
        reduce(
            &prog,
            "(λ λ λ 2 (3 2 1)) ((λ λ λ 2 (3 2 1)) (λ λ 1))",
            "λ λ 2 (2 1)",
        )
    }

    #[test]
    fn test_plus() -> Result {
        let plus = format!("00000101110{INC}10");
        let prog = format!("0101{plus}{THREE}{TWO}");
        reduce(
            &prog,
            "(λ λ 2 (λ λ λ 2 (3 2 1)) 1) (λ λ 2 (2 (2 1))) (λ λ 2 (2 1))",
            "λ λ 2 (2 (2 (2 (2 1))))",
        )
    }

    #[test]
    fn test_pow() -> Result {
        let prog = format!("01{TWO}{TWO}");
        reduce(&prog, "(λ λ 2 (2 1)) (λ λ 2 (2 1))", "λ λ 2 (2 (2 (2 1)))")?;

        let prog = format!("01{THREE}{TWO}");
        reduce(
            &prog,
            "(λ λ 2 (2 (2 1))) (λ λ 2 (2 1))",
            "λ λ 2 (2 (2 (2 (2 (2 (2 (2 1)))))))",
        )
    }

    #[test]
    fn test_even() -> Result {
        let not = format!("00010110{FALSE}{TRUE}");
        let even = format!("00010110{not}{TRUE}");
        let inc_ast = parse(INC)?;
        let even_ast = parse(&even)?;
        let expected = ["λ λ 2", "λ λ 1"];

        let mut x = parse(ZERO)?;
        for i in 0..=100 {
            let query = Expr::App(Box::new((even_ast.clone(), x.clone())));
            assert_eq!(query.eval_full().to_string(), expected[i % 2]);
            x = Expr::App(Box::new((inc_ast.clone(), x)));
        }
        Ok(())
    }

    #[test]
    fn test_pair() -> Result {
        let zero_and_one = format!("0101{PAIR}{ZERO}01{INC}{ZERO}");

        let prog = format!("01{SND}{zero_and_one}");
        reduce(
            &prog,
            "(λ 1 (λ λ 1)) ((λ λ λ 1 3 2) (λ λ 1) ((λ λ λ 2 (3 2 1)) (λ λ 1)))",
            "λ λ 2 1",
        )?;

        let prog = format!("01{FST}{zero_and_one}");
        reduce(
            &prog,
            "(λ 1 (λ λ 2)) ((λ λ λ 1 3 2) (λ λ 1) ((λ λ λ 2 (3 2 1)) (λ λ 1)))",
            "λ λ 1",
        )
    }

    #[test]
    fn test_y() -> Result {
        let nums = format!("0101{PAIR}{ONE}0101{PAIR}{TWO}0101{PAIR}{THREE}{ZERO}");
        let rev = format!("000000000101110111100101{PAIR}111010");

        let prog = format!("0101{nums}01{Y}{rev}{ZERO}");
        reduce(
            &prog,
            "(λ λ λ 1 3 2) (λ λ 2 1) ((λ λ λ 1 3 2) (λ λ 2 (2 1)) ((λ λ λ 1 3 2) (λ λ 2 (2 (2 1))) (λ λ 1))) ((λ (λ 2 (1 1)) (λ 2 (1 1))) (λ λ λ λ 2 4 ((λ λ λ 1 3 2) 3 1))) (λ λ 1)",
            "λ 1 (λ λ 2 (2 (2 1))) (λ 1 (λ λ 2 (2 1)) (λ 1 (λ λ 2 1) (λ λ 1)))",
        )
    }

    #[test]
    fn test_sum() -> Result {
        let nums = format!("0101{PAIR}{ONE}0101{PAIR}{TWO}0101{PAIR}{THREE}{ZERO}");
        let plus = format!("00000101110{INC}10");
        let sum = format!("000000000101110111100101{plus}101110");

        let prog = format!("0101{nums}01{Y}{sum}{ZERO}");
        reduce(
            &prog,
            "(λ λ λ 1 3 2) (λ λ 2 1) ((λ λ λ 1 3 2) (λ λ 2 (2 1)) ((λ λ λ 1 3 2) (λ λ 2 (2 (2 1))) (λ λ 1))) ((λ (λ 2 (1 1)) (λ 2 (1 1))) (λ λ λ λ 2 4 ((λ λ 2 (λ λ λ 2 (3 2 1)) 1) 1 3))) (λ λ 1)",
            "λ λ 2 (2 (2 (2 (2 (2 1)))))",
        )
    }

    #[test]
    fn test_args() -> Result {
        let reverse = format!("000000000101110111100101{PAIR}111010");
        let prog = format!("0001011001{Y}{reverse}{ZERO}");
        let out = run(&prog, Some("Hello World!"), OutputFmt::Parsed)?;
        assert_eq!(out, "!dlroW olleH");
        Ok(())
    }
}
