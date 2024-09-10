// x0 - x ones and trailing zero => de bruijn index
// 00 - two zeros => abstraction
// 01ab - a and b are bit sequences => apply b to a (a b)

use thiserror::Error;

use super::new_runner::{Expression, Term};

#[derive(Error, Debug, Clone, Copy)]
pub enum ParseError {
    #[error("de-brujin indexes must start from 1")]
    ZeroBruijnIndex,
    #[error("trying to reference variable out of bounds")]
    BruijnIndexOutOfBounds,
    #[error("part of the program is missing")]
    IncompleteStatement,
}

fn flatten_memory(
    slice: (usize, usize),
    mut prog_mem: Vec<Term>,
    expr_prog_mem: &Vec<Term>,
) -> Vec<Term> {
    for i in slice.0..=slice.1 {
        if let Term::Expression(Expression(s, e)) = prog_mem[i] {
            let d = e - s;
            let new_s = prog_mem.len();

            prog_mem.extend_from_slice(&expr_prog_mem[s..=e]);

            let Term::Expression(Expression(s, e)) = &mut prog_mem[i] else {
                unreachable!();
            };
            *s = new_s;
            *e = new_s + d;

            prog_mem = flatten_memory((*s, *e), prog_mem, expr_prog_mem);
        }
    }

    prog_mem
}

pub fn parse(prog: &str) -> Result<(usize, Vec<Term>), ParseError> {
    let mut prog_mem = Vec::new();
    let mut expr_prog_mem = Vec::new();

    parse_impl(prog.as_bytes(), 0, &mut prog_mem, &mut expr_prog_mem)?;

    println!("");
    println!("     MEM BEFORE: {:?}", prog_mem);
    println!("EXPR MEM BEFORE: {:?}", expr_prog_mem);
    println!("");

    Ok((
        prog_mem.len() - 1, // the top term
        flatten_memory((0, prog_mem.len() - 1), prog_mem, &expr_prog_mem),
    ))
}

fn parse_impl<'a>(
    prog: &'a [u8],
    depth: usize,
    prog_mem: &mut Vec<Term>,
    expr_prog_mem: &mut Vec<Term>,
) -> Result<&'a [u8], ParseError> {
    match prog {
        [b'0', b'0', tail @ ..] => {
            let tail = parse_impl(tail, depth + 1, prog_mem, expr_prog_mem)?;
            prog_mem.push(Term::Lambda);
            Ok(tail)
        }
        [b'0', b'1', tail @ ..] => {
            let len_before = prog_mem.len();
            println!("START ID({len_before}): {prog_mem:?}");
            let tail = parse_impl(tail, depth, prog_mem, expr_prog_mem)?;
            let terms_added_a = prog_mem.len() - len_before;
            println!("END ID({len_before}) RES({terms_added_a}): {prog_mem:?}");

            let len_before = prog_mem.len();
            let tail = parse_impl(tail, depth, prog_mem, expr_prog_mem)?;
            let terms_added_b = prog_mem.len() - len_before;

            let potential_expr_index = prog_mem.len() - 1 - terms_added_b;

            if let Term::Expression(Expression(_, e)) = &mut prog_mem[potential_expr_index] {
                *e += terms_added_b;
                let drain = prog_mem.drain(prog_mem.len() - terms_added_b..);
                expr_prog_mem.extend(drain);
            } else {
                let b = prog_mem
                    .drain(prog_mem.len() - terms_added_b..)
                    .collect::<Vec<_>>();

                let a = prog_mem.drain(prog_mem.len() - terms_added_a..);

                expr_prog_mem.extend(a);
                expr_prog_mem.extend(b);

                let terms_added = terms_added_a + terms_added_b;
                prog_mem.push(Term::Expression(Expression(
                    expr_prog_mem.len() - terms_added,
                    expr_prog_mem.len() - 1,
                )));
            }

            Ok(tail)
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
                    Some(_) => {
                        prog_mem.push(Term::Var(cnt - 1));
                        Ok(prog)
                    }
                    None => Err(ParseError::BruijnIndexOutOfBounds),
                }
            } else {
                Err(ParseError::IncompleteStatement)
            }
        }
    }
}
