// x0 - x ones and trailing zero => de bruijn index
// 00 - two zeros => abstraction
// 01ab - a and b are bit sequences => apply b to a (a b)

use thiserror::Error;

use super::evaluator::Term;

#[derive(Error, Debug, Clone, Copy)]
pub enum ParseError {
    #[error("de-brujin indexes must start from 1")]
    ZeroBruijnIndex,
    #[error("trying to reference variable out of bounds")]
    BruijnIndexOutOfBounds,
    #[error("part of the program is missing")]
    IncompleteStatement,
}

pub fn parse(prog: &str) -> Result<Term, ParseError> {
    Ok(parse_impl(prog.as_bytes(), 0)?.0)
}

fn parse_impl(prog: &[u8], depth: usize) -> Result<(Term, &[u8]), ParseError> {
    match prog {
        [b'0', b'0', tail @ ..] => {
            let (body, tail) = parse_impl(tail, depth + 1)?;
            Ok((
                Term::Lam(u32::try_from(depth).expect("failed cast"), Box::new(body)),
                tail,
            ))
        }
        [b'0', b'1', tail @ ..] => {
            let (a, tail) = parse_impl(tail, depth)?;
            let (b, tail) = parse_impl(tail, depth)?;
            Ok((Term::App(Box::new((a, b))), tail))
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

                match depth.checked_sub(cnt) {
                    Some(x) => Ok((Term::Var(u32::try_from(x).expect("failed cast")), prog)),
                    None => Err(ParseError::BruijnIndexOutOfBounds),
                }
            } else {
                Err(ParseError::IncompleteStatement)
            }
        }
    }
}
