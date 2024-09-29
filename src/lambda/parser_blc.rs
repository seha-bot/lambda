// x0 - x ones and trailing zero => de bruijn index
// 00 - two zeros => abstraction
// 01ab - a and b are bit sequences => apply b to a (a b)

use alloc::rc::Rc;
use core::cell::RefCell;

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
    let mut refs = Vec::new();
    Ok(parse_impl(prog.as_bytes(), &mut refs, 0)?.0)
}

fn parse_impl<'a>(
    prog: &'a [u8],
    refs: &mut Vec<Rc<RefCell<Option<Term>>>>,
    depth: usize,
) -> Result<(Term, &'a [u8]), ParseError> {
    match prog {
        [b'0', b'0', tail @ ..] => {
            let x = Rc::new(RefCell::new(None));
            refs.push(Rc::clone(&x));
            let (body, tail) = parse_impl(tail, refs, depth + 1)?;
            refs.pop();
            Ok((Term::Lam(x, Box::new(body)), tail))
        }
        [b'0', b'1', tail @ ..] => {
            let (a, tail) = parse_impl(tail, refs, depth)?;
            let (b, tail) = parse_impl(tail, refs, depth)?;
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

                match refs.get(depth.wrapping_sub(cnt)) {
                    Some(x) => Ok((Term::Var(Rc::clone(x)), prog)),
                    None => Err(ParseError::BruijnIndexOutOfBounds),
                }
            } else {
                Err(ParseError::IncompleteStatement)
            }
        }
    }
}
