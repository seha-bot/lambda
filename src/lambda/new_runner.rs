use std::fmt::Debug;

use super::new_parser_blc::{self};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Expression(pub usize, pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Term {
    Lambda,
    Var(usize),
    Expression(Expression),
}

fn fmt(i: usize, prog_mem: &Vec<Term>, heap: &Vec<BoundTerm>, stack: &Vec<usize>) -> String {
    match prog_mem[i] {
        Term::Lambda => format!("λ {}", fmt(i - 1, prog_mem, heap, stack)),
        Term::Var(x) => {
            if let Some(ptr) = stack.get(x) {
                format!("{}", heap[*ptr].fmt(prog_mem, heap))
            } else {
                (x + 1).to_string()
            }
        }
        Term::Expression(Expression(s, e)) => {
            format!(
                "{}",
                (s..=e).fold(String::new(), |mut acc, i| {
                    if i != e && prog_mem[i + 1] == Term::Lambda {
                        return acc;
                    }

                    let sub = fmt(i, prog_mem, heap, stack);
                    acc.push_str(&if let Term::Var(_) = prog_mem[i] {
                        sub
                    } else {
                        format!("({sub})")
                    });

                    if i != e {
                        acc.push(' ');
                    }
                    acc
                })
            )
        }
    }
}

#[derive(Debug)]
struct BoundTerm {
    stack: Vec<usize>,
    term_index: usize,
}

impl BoundTerm {
    // fn is_expr(&self) -> bool {
    //     if let Term::Expression(_) = self.term {
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn fmt(&self, prog_mem: &Vec<Term>, heap: &Vec<BoundTerm>) -> String {
        fmt(self.term_index, prog_mem, heap, &self.stack)
    }
}

// struct TopExpression(Vec<BoundTerm>);

// impl TopExpression {
//     fn eval_lazy(self, heap: &mut Vec<BoundTerm>) -> BoundTerm {
//         let TopExpression(mut terms) = self;
//         assert!(!terms.is_empty());

//         while terms.len() > 1 || terms[0].is_expr() {
//             Self::eval_one(&mut terms, heap);
//             assert!(!terms.is_empty());
//         }

//         let Some(first) = terms.drain(..).next() else {
//             unreachable!()
//         };

//         first
//     }

//     fn eval_one(terms: &mut Vec<BoundTerm>, heap: &mut Vec<BoundTerm>) {
//         todo!()
//         // let mut top = terms.pop().unwrap();

//         // match top.term {
//         //     Term::Lambda(Lambda(body)) => {
//         //         let to_be_applied = terms.pop().unwrap();
//         //         top.stack.push(heap.len());
//         //         heap.push(to_be_applied);
//         //         terms.push(BoundTerm {
//         //             stack: top.stack,
//         //             term: *body,
//         //         });
//         //     }
//         //     Term::Expression(Expression(subterms)) => {
//         //         terms.extend(subterms.iter().rev().map(|subterm| BoundTerm {
//         //             stack: top.stack.clone(),
//         //             term: *subterm,
//         //         }));
//         //     }
//         //     Term::Var(_) => panic!("invalid AST: found dangling variable."),
//         // }
//     }
// }

pub fn test() {
    // (\x.x) (\x.x) (\x.x)
    let (top_index, prog_mem) = new_parser_blc::parse("0101001000100010").unwrap();
    // // (\x.x) ((\x.x) (\x.x))
    // let (top_index, prog_mem) = new_parser_blc::parse("0100100100100010").unwrap();
    // // \x.x x
    // let (top_index, prog_mem) = new_parser_blc::parse("00011010").unwrap();
    // // (\x.x x) (\x.x x)
    // let (top_index, prog_mem) = new_parser_blc::parse("010001101000011010").unwrap();

    println!("PROG MEM: {:?}", prog_mem);
    println!("TOP PROG: {:?}", prog_mem[top_index]);

    let mut heap = vec![];

    let expr = BoundTerm {
        stack: vec![],
        term_index: top_index,
    };
    println!("{}", expr.fmt(&prog_mem, &heap));

    // let term = expr.eval_lazy(&mut heap);

    // println!("{}", term.fmt(&mem));
}
