use std::{cell::RefCell, collections::HashMap, fmt::Display, ptr::null_mut, rc::Rc};

#[derive(Clone)]
pub struct Expression {
    pub terms: Vec<Term>,
}

#[derive(Debug)]
pub enum ReductionError {
    EmptyExpressionError,
}

impl Expression {
    fn change_substitution(
        &mut self,
        old: &Rc<RefCell<Substitution>>,
        new: &Rc<RefCell<Substitution>>,
    ) {
        for term in self.terms.iter_mut() {
            term.change_substitution(old, new);
        }
    }

    pub fn reduce(mut self) -> Result<Term, ReductionError> {
        if self.terms.is_empty() {
            return Err(ReductionError::EmptyExpressionError);
        }

        let mut it = self.terms.drain(..);
        let mut accum = Vec::new();
        let mut potentially_reducible_result = false;

        let mut current = it.next();
        while let Some(term) = current.take() {
            match term {
                Term::Expression(expression) => {
                    if let Some(next) = it.next() {
                        potentially_reducible_result = true;
                        accum.push(expression.reduce()?);
                        current = Some(next);
                        continue;
                    }

                    if accum.is_empty() {
                        drop(it);
                        self = expression;
                        it = self.terms.drain(..);
                        current = it.next();
                    } else {
                        accum.push(expression.reduce()?);
                        break;
                    }
                }
                Term::Lambda(lambda) => {
                    if let Some(next) = it.next() {
                        if accum.is_empty() {
                            potentially_reducible_result = true;
                            lambda.substitution.borrow_mut().value = Some(next.reduce()?);
                            current = Some(lambda.body);
                        } else {
                            accum.push(lambda.reduce()?);
                            current = Some(next);
                        }
                    } else {
                        // else if?
                        if accum.is_empty() {
                            return lambda.reduce(); // is it possible to unwind here?
                        } else {
                            accum.push(lambda.reduce()?);
                            break;
                        }
                    }
                }
                Term::Substitution(substitution) => {
                    if let Some(x) = &substitution.borrow().value {
                        current = Some(x.clone());
                        continue;
                    }

                    if let Some(next) = it.next() {
                        accum.push(Term::Substitution(substitution));
                        current = Some(next);
                    } else {
                        // else if?
                        if accum.is_empty() {
                            return Ok(Term::Substitution(substitution));
                        } else {
                            accum.push(Term::Substitution(substitution));
                            break;
                        }
                    }
                }
            }
        }

        if potentially_reducible_result {
            Expression { terms: accum }.reduce() // TODO: unwind the stack here
        } else {
            Ok(Term::Expression(Expression { terms: accum }))
        }
    }

    pub fn fix_names(&mut self, seen: &mut HashMap<String, u32>) {
        for term in self.terms.iter_mut() {
            term.fix_names(seen);
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut it = self.terms.iter();
        if let Some(head) = it.next() {
            write!(f, "({})", head)?;
            for term in it {
                write!(f, " ({})", term)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Substitution {
    name: String,
    value: Option<Term>,
}

impl Substitution {
    pub fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Substitution {
            name: String::from(name),
            value: None,
        }))
    }
}

pub struct Lambda {
    pub substitution: Rc<RefCell<Substitution>>,
    pub body: Term,
}

impl Lambda {
    pub fn reduce(mut self) -> Result<Term, ReductionError> {
        self.body = self.body.reduce()?;
        // TODO: this is too expensive for the little result it returns.
        // maybe remove reduce from Lambda and Term
        Ok(Term::Lambda(Box::new(self)))
    }
}

impl Clone for Lambda {
    fn clone(&self) -> Self {
        let substitution = Rc::new(RefCell::new(self.substitution.borrow().clone()));
        let mut body = self.body.clone();
        body.change_substitution(&self.substitution, &substitution);
        Lambda { substitution, body }
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\\{}.{}", self.substitution.borrow().name, self.body)
    }
}

#[derive(Clone)]
pub enum Term {
    Expression(Expression),
    Lambda(Box<Lambda>),
    Substitution(Rc<RefCell<Substitution>>),
}

impl Term {
    fn change_substitution(
        &mut self,
        old: &Rc<RefCell<Substitution>>,
        new: &Rc<RefCell<Substitution>>,
    ) {
        match self {
            Term::Expression(x) => x.change_substitution(old, new),
            Term::Lambda(x) => x.body.change_substitution(old, new),
            Term::Substitution(x) => {
                if Rc::ptr_eq(&x, &old) {
                    *x = Rc::clone(&new);
                }
            }
        }
    }

    fn reduce(self) -> Result<Term, ReductionError> {
        match self {
            Term::Expression(x) => x.reduce(),
            Term::Lambda(x) => x.reduce(),
            Term::Substitution(x) => {
                if let Some(x) = &x.borrow().value {
                    return Ok(x.clone());
                }

                Ok(Term::Substitution(x))
            }
        }
    }

    pub fn fix_names(&mut self, seen: &mut HashMap<String, u32>) {
        match self {
            Term::Expression(x) => x.fix_names(seen),
            Term::Lambda(x) => {
                let suffix_ptr = seen
                    .get_mut(&x.substitution.borrow().name)
                    .map(|x| x as *mut u32)
                    .unwrap_or(null_mut());
                let mut diff = 0u32;

                if !suffix_ptr.is_null() {
                    let mut new_key = x.substitution.borrow().name.clone();
                    let len = new_key.len();

                    let mut incr = |x: *mut u32| unsafe {
                        let res = (*x).to_string();
                        *x += 1;
                        diff += 1;
                        res
                    };

                    new_key += &incr(suffix_ptr);
                    while seen.contains_key(&new_key) {
                        new_key.replace_range(len.., &incr(suffix_ptr));
                    }
                    x.substitution.borrow_mut().name = new_key;
                }

                let key = &x.substitution.borrow().name;
                if seen.insert(key.clone(), 1).is_some() {
                    panic!("fix_names inner logic error. Please report.");
                }
                x.body.fix_names(seen);
                seen.remove(key)
                    .expect("fix_names inner logic error. Please report.");
                if !suffix_ptr.is_null() {
                    unsafe { *suffix_ptr -= diff };
                }
            }
            Term::Substitution(_) => {}
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Expression(x) => write!(f, "{x}"),
            Term::Lambda(x) => write!(f, "{}", x),
            Term::Substitution(x) => match &x.borrow().value {
                Some(x) => write!(f, "{}", x),
                None => write!(f, "{}", x.borrow().name),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sll;

    use super::*;

    struct TestResult {
        got: Result<(String, String), ReductionError>,
        expected: (&'static str, &'static str),
    }

    fn reduce(
        root: Expression,
        expected_root_str: &'static str,
        expected_reduced: &'static str,
    ) -> Result<(), ReductionError> {
        let mut seen = HashMap::new();

        let root_str = {
            let mut root_clone = root.clone();
            root_clone.fix_names(&mut seen);
            root_clone.to_string()
        };
        assert_eq!(expected_root_str, root_str);

        let mut reduced = root.reduce()?;
        reduced.fix_names(&mut seen);

        assert_eq!(expected_reduced, reduced.to_string());
        Ok(())
    }

    #[test]
    fn test_id() -> Result<(), ReductionError> {
        let id = sll::id();

        let root = Expression {
            terms: vec![
                Term::Lambda(Box::new(id.clone())),
                Term::Lambda(Box::new(id)),
            ],
        };

        reduce(root, "(\\x.x) (\\x.x)", "\\x.x")
    }

    #[test]
    fn test_inc() -> Result<(), ReductionError> {
        let zero = sll::zero();
        let inc = sll::inc();

        let root = Expression {
            terms: vec![
                Term::Lambda(Box::new(inc.clone())),
                Term::Expression(Expression {
                    terms: vec![Term::Lambda(Box::new(inc)), Term::Lambda(Box::new(zero))],
                }),
            ],
        };

        reduce(
            root,
            "(\\n.\\f.\\x.(f) ((n) (f) (x))) ((\\n.\\f.\\x.(f) ((n) (f) (x))) (\\f.\\x.x))",
            "\\f.\\x.(f) ((f) (x))",
        )
    }

    #[test]
    fn test_first_reduction() -> Result<(), ReductionError> {
        let id = sll::id();
        let root = Expression {
            terms: vec![
                Term::Expression(Expression {
                    terms: vec![Term::Lambda(Box::new(id.clone()))],
                }),
                Term::Lambda(Box::new(id)),
            ],
        };

        reduce(root, "((\\x.x)) (\\x.x)", "\\x.x")
    }

    #[test]
    fn test_name_resolution() -> Result<(), ReductionError> {
        let flip = sll::flip();
        let id = sll::id();

        let root = Expression {
            terms: vec![
                Term::Lambda(Box::new(flip.clone())),
                Term::Expression(Expression {
                    terms: vec![Term::Lambda(Box::new(flip)), Term::Lambda(Box::new(id))],
                }),
            ],
        };

        reduce(
            root,
            "(\\y.\\x.(x) (y)) ((\\y.\\x.(x) (y)) (\\x.x))",
            "\\x.(x) (\\x1.(x1) (\\x2.x2))",
        )
    }

    #[test]
    fn test_double_name_resolution() -> Result<(), ReductionError> {
        let flip = sll::flip();
        let id = sll::id();

        let y = Substitution::new("y");
        let x1 = Substitution::new("x1");
        let flip_x1 = Lambda {
            substitution: y.clone(),
            body: Term::Lambda(Box::new(Lambda {
                substitution: x1.clone(),
                body: Term::Expression(Expression {
                    terms: vec![Term::Substitution(x1), Term::Substitution(y)],
                }),
            })),
        };

        let root = Expression {
            terms: vec![
                Term::Lambda(Box::new(flip)),
                Term::Expression(Expression {
                    terms: vec![Term::Lambda(Box::new(flip_x1)), Term::Lambda(Box::new(id))],
                }),
            ],
        };

        reduce(
            root,
            "(\\y.\\x.(x) (y)) ((\\y.\\x1.(x1) (y)) (\\x.x))",
            "\\x.(x) (\\x1.(x1) (\\x2.x2))",
        )
    }

    #[test]
    fn test_reverse_double_name_resolution() -> Result<(), ReductionError> {
        let flip = sll::flip();

        let x1 = Substitution::new("x1");
        let id_x1 = Lambda {
            substitution: x1.clone(),
            body: Term::Substitution(x1),
        };

        let root = Expression {
            terms: vec![
                Term::Lambda(Box::new(flip.clone())),
                Term::Expression(Expression {
                    terms: vec![Term::Lambda(Box::new(flip)), Term::Lambda(Box::new(id_x1))],
                }),
            ],
        };

        reduce(
            root,
            "(\\y.\\x.(x) (y)) ((\\y.\\x.(x) (y)) (\\x1.x1))",
            "\\x.(x) (\\x1.(x1) (\\x11.x11))",
        )
    }

    #[test]
    fn test_associativity() -> Result<(), ReductionError> {
        let id = sll::id();

        let x = Substitution::new("x");
        let f = Lambda {
            substitution: x.clone(),
            body: Term::Expression(Expression {
                terms: vec![
                    Term::Substitution(x),
                    Term::Lambda(Box::new(id.clone())),
                    Term::Lambda(Box::new(id)),
                ],
            }),
        };

        let root = Expression {
            terms: vec![Term::Lambda(Box::new(f))],
        };

        reduce(
            root,
            "(\\x.(x) (\\x1.x1) (\\x1.x1))",
            "\\x.(x) (\\x1.x1) (\\x1.x1)",
        )
    }

    #[test]
    fn test_name_triple() -> Result<(), ReductionError> {
        let id = sll::id();

        let f = Substitution::new("f");
        let g = Substitution::new("g");
        let x = Substitution::new("x");
        let flip = Lambda {
            substitution: f.clone(),
            body: Term::Lambda(Box::new(Lambda {
                substitution: g.clone(),
                body: Term::Lambda(Box::new(Lambda {
                    substitution: x.clone(),
                    body: Term::Expression(Expression {
                        terms: vec![
                            Term::Substitution(x),
                            Term::Substitution(g),
                            Term::Substitution(f),
                        ],
                    }),
                })),
            })),
        };

        let root = Expression {
            terms: vec![
                Term::Lambda(Box::new(flip)),
                Term::Lambda(Box::new(id.clone())),
                Term::Lambda(Box::new(id)),
            ],
        };

        reduce(
            root,
            "(\\f.\\g.\\x.(x) (g) (f)) (\\x.x) (\\x.x)",
            "\\x.(x) (\\x1.x1) (\\x1.x1)",
        )
    }
}
