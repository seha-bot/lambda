use runner::Expr;
use thiserror::Error;

mod parser_arg;
mod parser_blc;
mod parser_lc;
mod runner;

#[derive(Debug, Clone, Copy)]
pub enum InputFmt {
    Binary,
    Standard,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFmt {
    Binary,
    DeBruijn,
    Standard,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum RunError {
    #[error("failed parsing blc input: {0}")]
    Binary(#[from] parser_blc::ParseError),
    #[error("failed parsing input: {0}")]
    Standard(#[from] parser_lc::ParseError),
    #[error("failed parsing output: {0}")]
    Argument(#[from] parser_arg::ParseError),
}

pub fn run(
    prog: &str,
    arg: Option<&str>,
    input_fmt: InputFmt,
    output_fmt: OutputFmt,
) -> Result<String, RunError> {
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

    let expr = prog.eval_lazy().eval_full();

    Ok(match output_fmt {
        OutputFmt::Binary => expr.fmt_blc(),
        OutputFmt::DeBruijn => expr.fmt_bruijn(),
        OutputFmt::Standard => parser_arg::blc_to_bytes(&expr.fmt_blc())?
            .iter()
            .map(|&x| x as char)
            .collect::<String>(),
    })
}
