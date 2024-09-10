// x0 - x ones and trailing zero => de bruijn index
// 00 - two zeros => abstraction
// 01ab - a and b are bit sequences => apply b to a (a b)

use thiserror::Error;

use super::new_runner::Term;

#[derive(Error, Debug, Clone, Copy)]
pub enum ParseError {
    #[error("de-brujin indexes must start from 1")]
    ZeroBruijnIndex,
    #[error("trying to reference variable out of bounds")]
    BruijnIndexOutOfBounds,
    #[error("part of the program is missing")]
    IncompleteStatement,
}

fn flatten_expr_mem(
    // TODO: this can probably be just from like in flatten_lam_mem
    slice: (usize, usize),
    mut prog_mem: Vec<Term>,
    expr_mem: &Vec<Term>,
) -> Vec<Term> {
    for i in slice.0..=slice.1 {
        if let Term::Expression(s, e) = prog_mem[i] {
            let d = e - s;
            let new_s = prog_mem.len();

            prog_mem.extend_from_slice(&expr_mem[s..=e]);

            let Term::Expression(s, e) = &mut prog_mem[i] else {
                unreachable!();
            };
            *s = new_s;
            *e = new_s + d;

            prog_mem = flatten_expr_mem((*s, *e), prog_mem, expr_mem);
        }
    }

    prog_mem
}

fn flatten_lam_mem(from: usize, mut prog_mem: Vec<Term>, lam_mem: &Vec<Term>) -> Vec<Term> {
    let len = prog_mem.len();
    for i in from..len {
        if let Term::Lambda(x) = prog_mem[i] {
            let new_x = prog_mem.len();

            prog_mem.push(lam_mem[x]);

            let Term::Lambda(x) = &mut prog_mem[i] else {
                unreachable!();
            };
            *x = new_x;

            prog_mem = flatten_lam_mem(*x, prog_mem, lam_mem);
        }
    }

    prog_mem
}

pub fn parse(prog: &str) -> Result<(usize, Vec<Term>), ParseError> {
    let mut prog_mem = Vec::new();
    let mut expr_mem = Vec::new();
    let mut lam_mem = Vec::new();

    parse_impl(
        prog.as_bytes(),
        0,
        &mut prog_mem,
        &mut expr_mem,
        &mut lam_mem,
    )?;

    println!("");
    println!("     MEM BEFORE: {:?}", prog_mem);
    println!("EXPR MEM BEFORE: {:?}", expr_mem);
    println!(" LAM MEM BEFORE: {:?}", lam_mem);
    println!("");

    let top_term = prog_mem.len() - 1;
    let prog_mem = flatten_expr_mem((0, prog_mem.len() - 1), prog_mem, &expr_mem);
    let last_valid_index_before_flatten_lam = prog_mem.len() - 1;
    let prog_mem = flatten_lam_mem(0, prog_mem, &lam_mem);
    let prog_mem = flatten_expr_mem(
        (last_valid_index_before_flatten_lam, prog_mem.len() - 1),
        prog_mem,
        &expr_mem,
    );

    Ok((top_term, prog_mem))
}

fn parse_impl<'a>(
    prog: &'a [u8],
    depth: usize,
    prog_mem: &mut Vec<Term>,
    expr_mem: &mut Vec<Term>,
    lam_mem: &mut Vec<Term>,
) -> Result<&'a [u8], ParseError> {
    match prog {
        [b'0', b'0', tail @ ..] => {
            let tail = parse_impl(tail, depth + 1, prog_mem, expr_mem, lam_mem)?;
            lam_mem.push(prog_mem.pop().unwrap());
            prog_mem.push(Term::Lambda(lam_mem.len() - 1));
            Ok(tail)
        }
        [b'0', b'1', tail @ ..] => {
            let len_before = prog_mem.len();
            // println!("START ID({len_before}): {prog_mem:?}");
            let tail = parse_impl(tail, depth, prog_mem, expr_mem, lam_mem)?;
            let terms_added_a = prog_mem.len() - len_before;
            // println!("END ID({len_before}) RES({terms_added_a}): {prog_mem:?}");

            let len_before = prog_mem.len();
            let tail = parse_impl(tail, depth, prog_mem, expr_mem, lam_mem)?;
            let terms_added_b = prog_mem.len() - len_before;

            let potential_expr_index = prog_mem.len() - 1 - terms_added_b;

            if let Term::Expression(_, e) = &mut prog_mem[potential_expr_index] {
                *e += terms_added_b;
                let drain = prog_mem.drain(prog_mem.len() - terms_added_b..);
                expr_mem.extend(drain);
            } else {
                let b = prog_mem
                    .drain(prog_mem.len() - terms_added_b..)
                    .collect::<Vec<_>>();

                let a = prog_mem.drain(prog_mem.len() - terms_added_a..);

                expr_mem.extend(a);
                expr_mem.extend(b);

                let terms_added = terms_added_a + terms_added_b;
                prog_mem.push(Term::Expression(
                    expr_mem.len() - terms_added,
                    expr_mem.len() - 1,
                ));
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
