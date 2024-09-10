use std::fmt::Debug;

use super::new_parser_blc::{self};

#[derive(Debug, Clone, Copy)]
pub enum Term {
    Lambda(usize),
    Var(usize),
    Expression(usize, usize),
}

impl Term {
    fn is_potentially_reducible(&self) -> bool {
        match self {
            Term::Lambda(_) => false,
            Term::Var(_) | Term::Expression(_, _) => true,
        }
    }
}

fn fmt(i: usize, prog_mem: &Vec<Term>, heap: &Vec<BoundTerm>, stack: &Vec<usize>) -> String {
    match prog_mem[i] {
        Term::Lambda(i) => format!("λ {}", fmt(i, prog_mem, heap, stack)),
        Term::Var(x) => {
            if let Some(ptr) = stack.get(x) {
                format!("{}", heap[*ptr].fmt(prog_mem, heap))
            } else {
                (x + 1).to_string()
            }
        }
        Term::Expression(s, e) => {
            format!(
                "{}",
                (s..=e).fold(String::new(), |mut acc, i| {
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

#[derive(Debug, Clone)]
struct BoundTerm {
    stack: Vec<usize>,
    term_index: usize,
}

impl BoundTerm {
    fn fmt(&self, prog_mem: &Vec<Term>, heap: &Vec<BoundTerm>) -> String {
        fmt(self.term_index, prog_mem, heap, &self.stack)
    }
}

struct TopExpression(Vec<BoundTerm>);

impl TopExpression {
    fn eval_lazy(self, prog_mem: &Vec<Term>, heap: &mut Vec<BoundTerm>) -> BoundTerm {
        let TopExpression(mut terms) = self;
        assert!(!terms.is_empty());

        while terms.len() > 1 || prog_mem[terms[0].term_index].is_potentially_reducible() {
            Self::eval_one(prog_mem, &mut terms, heap);
            // TODO: remove these asserts once the runner is correct
            assert!(!terms.is_empty());

            println!("{terms:?}");
        }

        let Some(first) = terms.drain(..).next() else {
            unreachable!()
        };

        first
    }

    fn eval_one(prog_mem: &Vec<Term>, terms: &mut Vec<BoundTerm>, heap: &mut Vec<BoundTerm>) {
        let mut top = terms.pop().unwrap();

        match prog_mem[top.term_index] {
            Term::Lambda(i) => {
                let to_be_applied = terms.pop().unwrap();
                top.stack.push(heap.len());
                heap.push(to_be_applied);
                terms.push(BoundTerm {
                    stack: top.stack,
                    term_index: i,
                });
            }
            Term::Expression(s, e) => {
                terms.extend((s..=e).rev().map(|i| BoundTerm {
                    stack: top.stack.clone(),
                    term_index: i,
                }));
            }
            Term::Var(stack_i) => {
                if let Some(heap_i) = top.stack.get(stack_i) {
                    terms.push(heap[*heap_i].clone());
                } else {
                    panic!("invalid AST: found dangling variable.")
                }
            }
        }
    }
}

pub fn test() {
    // (\x.x) (\x.x) (\x.x)
    // let (top_index, prog_mem) = new_parser_blc::parse("0101001000100010").unwrap();
    // (\x.x) ((\x.x) (\x.x))
    // let (top_index, prog_mem) = new_parser_blc::parse("0100100100100010").unwrap();
    // \x.x x
    // let (top_index, prog_mem) = new_parser_blc::parse("00011010").unwrap();
    // (\x.x x) (\x.x x)
    let (top_index, prog_mem) = new_parser_blc::parse("010001101000011010").unwrap();

    println!("PROG MEM: {:?}", prog_mem);
    println!("TOP PROG: {:?}", prog_mem[top_index]);
    println!("");

    let mut heap = vec![];

    let expr = TopExpression(vec![BoundTerm {
        stack: Vec::new(),
        term_index: top_index,
    }]);

    let after = expr.eval_lazy(&prog_mem, &mut heap);

    let before = BoundTerm {
        stack: Vec::new(),
        term_index: top_index,
    };
    println!("BEFORE: {}", before.fmt(&prog_mem, &heap));
    println!("AFTER : {}", after.fmt(&prog_mem, &heap));
}
