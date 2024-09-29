use thiserror::Error;

mod byte_encoder;
mod evaluator;
mod parser_ast;
mod parser_blc;
mod parser_lc;

#[derive(Debug, Clone, Copy)]
pub enum InputFmt {
    Binary,
    Standard,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFmt {
    Bytes,
    Bits,
}

#[derive(Error, Debug)]
pub enum RunError {
    #[error("failed parsing blc input: {0}")]
    Binary(#[from] parser_blc::ParseError),
    #[error("failed parsing input: {0}")]
    Standard(#[from] parser_lc::ParseError),
    #[error("runtime error: {0}")]
    RuntimeError(#[from] parser_ast::ParseError),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}

pub fn run(
    prog_raw: &str,
    args_raw: Option<&str>,
    input_fmt: InputFmt,
    output_fmt: OutputFmt,
) -> Result<(), RunError> {
    let prog = match input_fmt {
        InputFmt::Binary => parser_blc::parse(prog_raw)?,
        InputFmt::Standard => parser_lc::parse(prog_raw)?,
    };

    let prog = if let Some(arg) = args_raw {
        evaluator::Term::App(Box::new((
            prog,
            parser_blc::parse(&byte_encoder::bytes_to_blc(arg.as_bytes()))?,
        )))
    } else {
        prog
    };

    let mut prog = evaluator::eval(Vec::new(), prog);
    while let Some((head, tail)) = parser_ast::uncons(prog)? {
        let head = head.eval();

        let c = match output_fmt {
            OutputFmt::Bytes => parser_ast::ast_to_byte(head)?,
            OutputFmt::Bits if parser_ast::ast_to_bool(head)? => b'1',
            OutputFmt::Bits => b'0',
        };
        std::io::Write::write_all(&mut std::io::stdout(), &[c])?;
        prog = tail.eval();
    }

    Ok(())
}
