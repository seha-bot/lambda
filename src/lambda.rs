use alloc::rc::Rc;
use std::io::{self, Write};

use runner::Expr;
use thiserror::Error;

mod parser_arg;
mod parser_blc;
mod new_parser_blc;
mod parser_lc;
mod runner;
pub mod new_runner;

#[derive(Debug, Clone, Copy)]
pub enum InputFmt {
    Binary,
    Standard,
}

// #[derive(Debug, Clone, Copy)]
// pub enum OutputFmt {
//     Binary,
//     DeBruijn,
//     Standard,
// }

#[derive(Error, Debug)]
pub enum RunError {
    #[error("failed parsing blc input: {0}")]
    Binary(#[from] parser_blc::ParseError),
    #[error("failed parsing input: {0}")]
    Standard(#[from] parser_lc::ParseError),
    #[error("failed parsing output: {0}")]
    Argument(#[from] parser_arg::ParseError),
    #[error("io error: {0}")]
    IO(#[from] io::Error),
    #[error("WHNF did not result in a list")]
    NotReducedToList,
}

pub fn run(prog: &str, arg: Option<&str>, input_fmt: InputFmt) -> Result<(), RunError> {
    let prog = match input_fmt {
        InputFmt::Binary => parser_blc::parse(prog)?,
        InputFmt::Standard => parser_lc::parse(prog)?,
    };

    let prog = if let Some(arg) = arg {
        Expr::App(Box::new((
            prog,
            parser_blc::parse(&parser_arg::bytes_to_blc(arg.as_bytes()))?,
        )))
    } else {
        prog
    };

    let mut expr = prog;
    while let Some((head, tail)) = uncons(expr.eval_lazy())? {
        expr = tail;
        // TODO: figure out if this evaluation strategy for head is ok
        let c = parser_arg::blc_to_byte(&head.eval_lazy().eval_full().fmt_blc())?.0;
        io::stdout().write_all(&[c])?;
    }

    Ok(())
}

fn uncons(expr: Expr) -> Result<Option<(Expr, Expr)>, RunError> {
    if let Expr::Lam(x, body) = expr {
        if let Expr::App(app) = *body {
            let r = app.1;
            if let Expr::App(app) = app.0 {
                let l = app.1;
                if let Expr::Var(var) = app.0 {
                    if Rc::ptr_eq(&var, &x) {
                        return Ok(Some((l, r)));
                    }
                }
            }
        } else if let Expr::Lam(y, body) = *body {
            if let Expr::Var(var) = *body {
                if Rc::ptr_eq(&var, &y) {
                    return Ok(None);
                }
            }
        }
    }

    Err(RunError::NotReducedToList)
}
